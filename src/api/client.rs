use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct HttpClient {
    pub client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(8)
            .build()?;
        Ok(Self { client })
    }
}
