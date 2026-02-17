//! HoYoLab HTTP client implementation.

use reqwest::Method;
use reqwest::header::COOKIE;
use serde::Serialize;
use serde::de::DeserializeOwned;
use storekeeper_client_core::{ApiResponse, ClientError, HoyolabApiResponse, HttpClientBuilder};

use crate::ds::generate_dynamic_secret_overseas;
use crate::error::{Error, Result};

/// HoYoLab API client.
#[derive(Debug, Clone)]
pub struct HoyolabClient {
    client: reqwest::Client,
    cookie: String,
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

        let ltuid = ltuid.into();
        let ltoken = ltoken.into();
        let cookie = format!("ltuid_v2={ltuid}; ltoken_v2={ltoken}");

        Ok(Self { client, cookie })
    }

    /// Makes an authenticated GET request to the HoYoLab API.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.request_with_headers::<T, ()>(Method::GET, url, None, &[])
            .await
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

    /// Makes an authenticated POST request to the HoYoLab API.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn post<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.request_with_headers::<T, ()>(Method::POST, url, None, &[])
            .await
    }

    /// Makes an authenticated request with custom headers to the HoYoLab API.
    ///
    /// This is useful for endpoints like daily rewards that require additional
    /// headers such as `x-rpc-signgame`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn request_with_headers<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
        extra_headers: &[(&str, &str)],
    ) -> Result<T> {
        let ds = generate_dynamic_secret_overseas();

        tracing::debug!(url = %url, method = %method, "HoYoLab API request");

        let mut request = self.client.request(method, url);
        request = request.header(COOKIE, &self.cookie).header("DS", ds);

        // Add extra headers (e.g., x-rpc-signgame for daily rewards)
        for (name, value) in extra_headers {
            request = request.header(*name, *value);
        }

        // Add body for POST requests if provided
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;

        let status = response.status();
        tracing::debug!(status = %status, url = %url, "HoYoLab API response received");

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let body_preview: String = body.chars().take(300).collect();
            let message = format!("HTTP {status} from HoYoLab API: {}", body_preview.trim());
            tracing::warn!(url = %url, status = %status, body_preview = %body_preview, "HoYoLab HTTP error");
            return Err(Error::Client(ClientError::api_error(
                i32::from(status.as_u16()),
                message,
            )));
        }

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
}
