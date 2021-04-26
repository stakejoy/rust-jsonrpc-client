use anyhow::Result;
use jsonrpc_client::SendRequest;

#[jsonrpc_client::api]
pub trait Math {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[jsonrpc_client::implement(Math)]
struct Client {
    inner: reqwest::Client,
    base_url: jsonrpc_client::Url,
}

impl Client {
    fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            base_url: base_url.parse()?,
        })
    }

    async fn multiply(
        &self,
        value: i64,
        factor: i64,
    ) -> Result<i64, jsonrpc_client::Error<jsonrpc_client::reqwest::Error>> {
        let body = jsonrpc_client::Request::new_v2("multiply")
            .with_argument(String::from("value"), value)?
            .with_argument(String::from("factor"), factor)?
            .serialize()?;

        let payload = self
            .inner
            .send_request::<i64>(self.base_url.clone(), body)
            .await
            .map_err(|e| jsonrpc_client::Error::Client {
                inner: e,
                rpc_method: "multiply",
            })?
            .payload;
        let response = Result::from(payload).map_err(|e| jsonrpc_client::Error::JsonRpc {
            inner: e,
            rpc_method: "multiply",
        })?;

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://example-jsonrpc.org/".to_owned())?;

    let _ = client.subtract(10, 5).await?;
    let _ = client.multiply(10, 5).await?;

    Ok(())
}
