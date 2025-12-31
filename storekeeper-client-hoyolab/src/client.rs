//! HoYoLab HTTP client implementation.

use reqwest::header::COOKIE;
use serde::de::DeserializeOwned;
use storekeeper_client_core::{ApiResponse, ClientError, HoyolabApiResponse, HttpClientBuilder};

use crate::ds::generate_ds_overseas;
use crate::error::{Error, Result};

/// HoYoLab API client.
#[derive(Debug, Clone)]
pub struct HoyolabClient {
    client: reqwest::Client,
    ltuid: String,
    ltoken: String,
}

impl HoyolabClient {
    /// Creates a new HoYoLab client with the given credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(ltuid: impl Into<String>, ltoken: impl Into<String>) -> Result<Self> {
        let client = HttpClientBuilder::new()
            .header_static("x-rpc-app_version", "1.5.0")
            .header_static("x-rpc-client_type", "5")
            .header_static("x-rpc-language", "en-us")
            .build()
            .map_err(Error::Client)?;

        Ok(Self {
            client,
            ltuid: ltuid.into(),
            ltoken: ltoken.into(),
        })
    }

    /// Makes an authenticated GET request to the HoYoLab API.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let ds = generate_ds_overseas();
        // Use v2 cookie format which is current HoYoLab standard
        let cookie = format!("ltuid_v2={}; ltoken_v2={}", self.ltuid, self.ltoken);

        tracing::debug!(url = %url, "HoYoLab API GET request");

        let response = self
            .client
            .get(url)
            .header(COOKIE, cookie)
            .header("DS", ds)
            .send()
            .await?;

        let status = response.status();
        tracing::debug!(status = %status, url = %url, "HoYoLab API response received");

        let api_response: HoyolabApiResponse<T> = response.json().await?;

        if !api_response.is_success() {
            tracing::warn!(
                retcode = api_response.retcode,
                message = %api_response.message,
                url = %url,
                "HoYoLab API error response"
            );
        }

        tracing::debug!(url = %url, "HoYoLab API request successful");

        api_response.into_result().map_err(Error::Client)
    }

    /// Checks if the client credentials are valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication check fails.
    pub async fn check_auth(&self) -> Result<bool> {
        // Try to fetch user info to verify credentials
        let url = "https://bbs-api-os.hoyolab.com/community/user/wapi/getUserFullInfo";
        match self.get::<serde_json::Value>(url).await {
            Ok(_) => Ok(true),
            Err(Error::Client(ClientError::ApiError { code: -100, .. })) => Ok(false), // Not logged in
            Err(e) => Err(e),
        }
    }
}
