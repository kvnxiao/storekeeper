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
    auth_check_url: String,
}

const HOYOLAB_AUTH_CHECK_URL: &str =
    "https://bbs-api-os.hoyolab.com/community/user/wapi/getUserFullInfo";

impl HoyolabClient {
    /// Creates a new HoYoLab client with the given credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(ltuid: impl Into<String>, ltoken: impl Into<String>) -> Result<Self> {
        Self::with_auth_check_url(ltuid, ltoken, HOYOLAB_AUTH_CHECK_URL)
    }

    /// Creates a new HoYoLab client with a custom auth-check URL.
    ///
    /// This is primarily useful for testing.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_auth_check_url(
        ltuid: impl Into<String>,
        ltoken: impl Into<String>,
        auth_check_url: impl Into<String>,
    ) -> Result<Self> {
        let client = HttpClientBuilder::new()
            .header_static("x-rpc-app_version", "1.5.0")
            .header_static("x-rpc-client_type", "5")
            .header_static("x-rpc-language", "en-us")
            .build()
            .map_err(Error::Client)?;

        let ltuid = ltuid.into();
        let ltoken = ltoken.into();
        let cookie = format!("ltuid_v2={ltuid}; ltoken_v2={ltoken}");

        Ok(Self {
            client,
            cookie,
            auth_check_url: auth_check_url.into(),
        })
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
        match self.get::<serde_json::Value>(&self.auth_check_url).await {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use reqwest::Method;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::{Mutex, oneshot};

    use super::*;

    #[derive(Debug, Clone)]
    struct TestRequest {
        method: String,
        target: String,
        headers: HashMap<String, String>,
        body: String,
    }

    #[derive(Debug, Clone)]
    struct TestResponse {
        status: u16,
        body: String,
        content_type: &'static str,
    }

    struct TestServer {
        base_url: String,
        requests: Arc<Mutex<Vec<TestRequest>>>,
        shutdown_tx: Option<oneshot::Sender<()>>,
    }

    impl TestServer {
        async fn spawn(handler: Arc<dyn Fn(&TestRequest) -> TestResponse + Send + Sync>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test server");
            let addr = listener.local_addr().expect("get local addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_clone = Arc::clone(&requests);
            let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = &mut shutdown_rx => {
                            break;
                        }
                        accepted = listener.accept() => {
                            let Ok((mut stream, _)) = accepted else {
                                break;
                            };
                            let requests = Arc::clone(&requests_clone);
                            let handler = Arc::clone(&handler);
                            tokio::spawn(async move {
                                if let Some(request) = read_request(&mut stream).await {
                                    requests.lock().await.push(request.clone());
                                    let response = handler(&request);
                                    let _ = write_response(&mut stream, response).await;
                                }
                            });
                        }
                    }
                }
            });

            Self {
                base_url: format!("http://{addr}"),
                requests,
                shutdown_tx: Some(shutdown_tx),
            }
        }

        async fn requests(&self) -> Vec<TestRequest> {
            self.requests.lock().await.clone()
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }
        }
    }

    async fn read_request(stream: &mut TcpStream) -> Option<TestRequest> {
        let mut raw = Vec::new();
        let mut buf = [0_u8; 1024];

        let header_end = loop {
            let read = stream.read(&mut buf).await.ok()?;
            if read == 0 {
                return None;
            }
            raw.extend_from_slice(&buf[..read]);
            if let Some(pos) = find_header_end(&raw) {
                break pos;
            }
        };

        let head = String::from_utf8_lossy(&raw[..header_end]).to_string();
        let mut lines = head.split("\r\n");
        let request_line = lines.next()?.to_string();
        let mut request_line_parts = request_line.split_whitespace();
        let method = request_line_parts.next()?.to_string();
        let target = request_line_parts.next()?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut body = raw[header_end + 4..].to_vec();
        while body.len() < content_length {
            let read = stream.read(&mut buf).await.ok()?;
            if read == 0 {
                break;
            }
            body.extend_from_slice(&buf[..read]);
        }

        let body_len = content_length.min(body.len());
        let body = String::from_utf8_lossy(&body[..body_len]).to_string();

        Some(TestRequest {
            method,
            target,
            headers,
            body,
        })
    }

    fn find_header_end(bytes: &[u8]) -> Option<usize> {
        bytes.windows(4).position(|window| window == b"\r\n\r\n")
    }

    async fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
        let reason = match response.status {
            401 => "Unauthorized",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            _ => "OK",
        };
        let body = response.body;
        let reply = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.status,
            reason,
            response.content_type,
            body.len(),
            body
        );
        stream.write_all(reply.as_bytes()).await
    }

    fn json_response(status: u16, body: &str) -> TestResponse {
        TestResponse {
            status,
            body: body.to_string(),
            content_type: "application/json",
        }
    }

    fn text_response(status: u16, body: &str) -> TestResponse {
        TestResponse {
            status,
            body: body.to_string(),
            content_type: "text/plain",
        }
    }

    #[tokio::test]
    async fn request_with_headers_includes_cookie_ds_extra_headers_and_body() {
        let server = TestServer::spawn(Arc::new(|_| {
            json_response(200, r#"{"retcode":0,"message":"OK","data":{"ok":true}}"#)
        }))
        .await;

        let auth_url = format!("{}/auth", server.base_url);
        let client =
            HoyolabClient::with_auth_check_url("123", "456", auth_url).expect("create client");
        let url = format!("{}/test", server.base_url);
        let body = serde_json::json!({ "hello": "world" });
        let result: serde_json::Value = client
            .request_with_headers(
                Method::POST,
                &url,
                Some(&body),
                &[("x-rpc-signgame", "hk4e"), ("x-extra", "value")],
            )
            .await
            .expect("request should succeed");

        assert_eq!(result, serde_json::json!({"ok": true}));

        let requests = server.requests().await;
        assert_eq!(requests.len(), 1, "Expected a single request");
        let request = &requests[0];
        assert_eq!(request.method, "POST");
        assert_eq!(
            request.headers.get("cookie"),
            Some(&"ltuid_v2=123; ltoken_v2=456".to_string())
        );
        assert!(
            request
                .headers
                .get("ds")
                .is_some_and(|value| !value.is_empty()),
            "DS header should be present and non-empty"
        );
        assert_eq!(
            request.headers.get("x-rpc-signgame"),
            Some(&"hk4e".to_string())
        );
        assert_eq!(request.headers.get("x-extra"), Some(&"value".to_string()));
        assert!(
            request.body.contains("\"hello\":\"world\""),
            "Request body should contain serialized JSON, got: {}",
            request.body
        );
    }

    #[tokio::test]
    async fn non_success_http_status_maps_to_api_error() {
        let server = TestServer::spawn(Arc::new(|_| text_response(429, "rate limited"))).await;

        let auth_url = format!("{}/auth", server.base_url);
        let client =
            HoyolabClient::with_auth_check_url("123", "456", auth_url).expect("create client");
        let url = format!("{}/not-ok", server.base_url);

        let result = client.get::<serde_json::Value>(&url).await;
        assert!(
            matches!(
                result,
                Err(Error::Client(ClientError::ApiError { code: 429, ref message }))
                    if message.contains("HTTP 429")
            ),
            "Expected HTTP 429 to map to ApiError with status in message, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn api_retcode_failure_maps_to_api_error() {
        let server = TestServer::spawn(Arc::new(|_| {
            json_response(
                200,
                r#"{"retcode":-1002,"message":"invalid request","data":null}"#,
            )
        }))
        .await;

        let auth_url = format!("{}/auth", server.base_url);
        let client =
            HoyolabClient::with_auth_check_url("123", "456", auth_url).expect("create client");
        let url = format!("{}/retcode-error", server.base_url);

        let result = client.get::<serde_json::Value>(&url).await;
        assert!(
            matches!(
                result,
                Err(Error::Client(ClientError::ApiError { code: -1002, ref message }))
                    if message == "invalid request"
            ),
            "Expected retcode failure to map to ApiError, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn check_auth_returns_false_for_not_logged_in() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/auth") {
                json_response(
                    200,
                    r#"{"retcode":-100,"message":"not logged in","data":null}"#,
                )
            } else {
                json_response(200, r#"{"retcode":0,"message":"OK","data":{}}"#)
            }
        }))
        .await;

        let auth_url = format!("{}/auth", server.base_url);
        let client =
            HoyolabClient::with_auth_check_url("123", "456", auth_url).expect("create client");
        let result = client
            .check_auth()
            .await
            .expect("check_auth should resolve");
        assert!(
            !result,
            "retcode -100 should map to unauthenticated (false)"
        );
    }

    #[tokio::test]
    async fn check_auth_propagates_unexpected_api_errors() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/auth") {
                json_response(
                    200,
                    r#"{"retcode":-999,"message":"unexpected","data":null}"#,
                )
            } else {
                json_response(200, r#"{"retcode":0,"message":"OK","data":{}}"#)
            }
        }))
        .await;

        let auth_url = format!("{}/auth", server.base_url);
        let client =
            HoyolabClient::with_auth_check_url("123", "456", auth_url).expect("create client");
        let result = client.check_auth().await;
        assert!(
            matches!(
                result,
                Err(Error::Client(ClientError::ApiError { code: -999, .. }))
            ),
            "Unexpected API error should be returned, got: {result:?}"
        );
    }
}
