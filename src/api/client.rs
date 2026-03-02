use crate::config::get_credentials;

pub(crate) struct Client {
    http: reqwest::Client,
    base_url: &'static str,
    header_name: &'static str,
    api_key: Option<String>,
}

impl Client {
    pub(crate) fn build() -> Result<Self, Box<dyn std::error::Error>> {
        let creds = get_credentials();
        Ok(Client {
            http: reqwest::Client::builder()
                .user_agent(concat!("coingecko-cli/", env!("CARGO_PKG_VERSION")))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            base_url: creds.tier.base_url(),
            header_name: creds.tier.header_key(),
            api_key: creds.api_key,
        })
    }

    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let req = self.http.get(url);
        match &self.api_key {
            Some(key) => req.header(self.header_name, key),
            None => req,
        }
    }
}
