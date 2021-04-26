#![cfg(feature = "macros")]

use jsonrpc_client::{Id, Response, SendRequest, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::{cell::Cell, fmt, ops::Deref};

#[jsonrpc_client::api(version = "1.0")]
pub trait MathV1 {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[jsonrpc_client::api(version = "2.0")]
pub trait MathV2 {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[jsonrpc_client::api(version = "2.0")]
pub trait MathV2Default {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[derive(Default)]
pub struct InnerClient {
    next_response: Cell<Option<String>>,
    recorded_request: Cell<Option<String>>,
}

unsafe impl Sync for InnerClient {}

impl InnerClient {
    fn with_next_response<P>(response: Response<P>) -> Self
    where
        P: Serialize,
    {
        Self {
            next_response: Cell::new(Some(serde_json::to_string(&response).unwrap())),
            recorded_request: Cell::new(None),
        }
    }

    fn take_recorded_request(&self) -> String {
        self.recorded_request.take().unwrap()
    }
}

#[derive(Debug)]
pub struct DummyError;

impl fmt::Display for DummyError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for DummyError {}

#[async_trait::async_trait]
impl SendRequest for InnerClient {
    type Error = DummyError;

    async fn send_request<P>(&self, _: Url, request: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        self.recorded_request.set(Some(request));
        let response = self.next_response.replace(None).unwrap();

        Ok(serde_json::from_str(&response).unwrap())
    }
}

pub struct ExampleDotOrg(Url);

impl Default for ExampleDotOrg {
    fn default() -> Self {
        Self("http://example.org".parse().unwrap())
    }
}

impl Deref for ExampleDotOrg {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn assert_impls_math_v1<C: SendRequest, T: MathV1<C>>(_: T) {}
fn assert_impls_math_v2<C: SendRequest, T: MathV2<C>>(_: T) {}
fn assert_impls_math_v2_default<C: SendRequest, T: MathV2Default<C>>(_: T) {}

mod derive_on_named_inner {
    use crate::{ExampleDotOrg, InnerClient};

    #[jsonrpc_client::implement(super::MathV1)]
    #[jsonrpc_client::implement(super::MathV2)]
    #[jsonrpc_client::implement(super::MathV2Default)]
    #[derive(Default)]
    pub struct Client {
        pub inner: InnerClient,
        pub base_url: ExampleDotOrg,
    }
}

mod derive_on_named_inner_multiple_fields {
    use crate::{ExampleDotOrg, InnerClient};

    #[jsonrpc_client::implement(super::MathV2Default)]
    #[derive(Default)]
    pub struct Client {
        inner: InnerClient,
        base_url: ExampleDotOrg,
        _foobar: u64,
    }
}

// TODO: test for attr on multiple fields

#[test]
fn test_impls_math_api() {
    assert_impls_math_v1(derive_on_named_inner::Client::default());
    assert_impls_math_v2(derive_on_named_inner::Client::default());
    assert_impls_math_v2_default(derive_on_named_inner::Client::default());
    assert_impls_math_v2_default(derive_on_named_inner_multiple_fields::Client::default());
}

#[tokio::test]
async fn creates_correct_v1_request() {
    let client = derive_on_named_inner::Client {
        inner: InnerClient::with_next_response(Response::new_v1_result(Id::Number(1), 1)),
        ..derive_on_named_inner::Client::default()
    };

    let result = MathV1::subtract(&client, 5, 4).await.unwrap();

    assert_eq!(result, 1);
    assert_eq!(
        client.inner.take_recorded_request(),
        r#"{"id":0,"jsonrpc":"1.0","method":"subtract","params":[5,4]}"#
    );
}

#[tokio::test]
async fn creates_correct_v2_request() {
    let client = derive_on_named_inner::Client {
        inner: InnerClient::with_next_response(Response::new_v2_result(Id::Number(1), 1)),
        ..derive_on_named_inner::Client::default()
    };

    let result = MathV2::subtract(&client, 5, 4).await.unwrap();

    assert_eq!(result, 1);
    assert_eq!(
        client.inner.take_recorded_request(),
        r#"{"id":0,"jsonrpc":"2.0","method":"subtract","params":{"minuend":4,"subtrahend":5}}"#
    );
}

#[tokio::test]
async fn creates_correct_v2_default_request() {
    let client = derive_on_named_inner::Client {
        inner: InnerClient::with_next_response(Response::new_v2_result(Id::Number(1), 1)),
        ..derive_on_named_inner::Client::default()
    };

    let result = MathV2Default::subtract(&client, 5, 4).await.unwrap();

    assert_eq!(result, 1);
    assert_eq!(
        client.inner.take_recorded_request(),
        r#"{"id":0,"jsonrpc":"2.0","method":"subtract","params":{"minuend":4,"subtrahend":5}}"#
    );
}
