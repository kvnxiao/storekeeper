//! Generic HoYoLab daily reward client.
//!
//! Provides a config-driven `DailyRewardClient` implementation that works for
//! all HoYoLab games (Genshin Impact, Honkai: Star Rail, Zenless Zone Zero).

use crate::client::HoyolabClient;
use crate::error::ClientError;
use crate::error::Error;
use crate::error::Result;
use crate::retcode;
use crate::retcode::RetcodeKind;
use reqwest::Method;
use serde::Deserialize;
use storekeeper_core::ClaimResult;
use storekeeper_core::DailyReward;
use storekeeper_core::DailyRewardClient;
use storekeeper_core::DailyRewardInfo;
use storekeeper_core::DailyRewardStatus;
use storekeeper_core::GameId;

/// Configuration for a HoYoLab daily reward endpoint.
#[derive(Debug, Clone, Copy)]
pub struct HoyolabDailyRewardConfig {
    /// Base URL for the reward API (e.g., `https://sg-hk4e-api.hoyolab.com/event/sol`).
    pub reward_url: &'static str,
    /// Act ID for the daily reward event.
    pub act_id: &'static str,
    /// Value for the `x-rpc-signgame` header.
    pub sign_game: &'static str,
    /// Game identifier.
    pub game_id: GameId,
}

/// Genshin Impact daily reward configuration.
pub const GENSHIN_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-hk4e-api.hoyolab.com/event/sol",
    act_id: "e202102251931481",
    sign_game: "hk4e",
    game_id: GameId::GenshinImpact,
};

/// Honkai: Star Rail daily reward configuration.
pub const HSR_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-public-api.hoyolab.com/event/luna/hkrpg/os",
    act_id: "e202303301540311",
    sign_game: "hkrpg",
    game_id: GameId::HonkaiStarRail,
};

/// Zenless Zone Zero daily reward configuration.
pub const ZZZ_DAILY_REWARD: HoyolabDailyRewardConfig = HoyolabDailyRewardConfig {
    reward_url: "https://sg-public-api.hoyolab.com/event/luna/zzz/os",
    act_id: "e202406031448091",
    sign_game: "zzz",
    game_id: GameId::ZenlessZoneZero,
};

/// API response for daily reward info (`/info` endpoint).
#[derive(Debug, Deserialize)]
struct RewardInfoResponse {
    is_sign: bool,
    total_sign_day: u32,
}

/// API response for monthly rewards list (`/home` endpoint).
#[derive(Debug, Deserialize)]
struct RewardHomeResponse {
    awards: Vec<RewardItem>,
}

/// Individual reward item in the monthly rewards list.
#[derive(Debug, Deserialize)]
struct RewardItem {
    name: String,
    #[serde(alias = "cnt")]
    count: u32,
    icon: String,
}

/// API response fields used to diagnose an unregistered sign attempt.
#[derive(Debug, Deserialize)]
struct SignResponse {
    risk_code: Option<i32>,
    gt: Option<String>,
    challenge: Option<String>,
}

/// Generic HoYoLab daily reward client.
pub struct HoyolabDailyRewardClient {
    client: HoyolabClient,
    config: &'static HoyolabDailyRewardConfig,
}

impl HoyolabDailyRewardClient {
    /// Creates a new daily reward client with the given HoYoLab client and
    /// config.
    #[must_use]
    pub fn new(client: HoyolabClient, config: &'static HoyolabDailyRewardConfig) -> Self {
        Self { client, config }
    }

    /// Builds a daily reward URL with the given endpoint.
    fn reward_url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}?act_id={}&lang=en-us",
            self.config.reward_url, endpoint, self.config.act_id
        )
    }

    /// Returns the headers required for daily reward requests.
    fn reward_headers(&self) -> [(&'static str, &'static str); 2] {
        [
            ("x-rpc-signgame", self.config.sign_game),
            ("referer", "https://act.hoyolab.com/"),
        ]
    }
}

impl DailyRewardClient for HoyolabDailyRewardClient {
    type Error = Error;

    fn game_id(&self) -> GameId {
        self.config.game_id
    }

    async fn get_reward_info(&self) -> Result<DailyRewardInfo> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching daily reward info");

        let url = self.reward_url("info");
        let headers = self.reward_headers();

        let response: RewardInfoResponse = self
            .client
            .request_with_headers::<RewardInfoResponse, ()>(Method::GET, &url, None, &headers)
            .await?;

        Ok(DailyRewardInfo::new(
            response.is_sign,
            response.total_sign_day,
        ))
    }

    async fn get_monthly_rewards(&self) -> Result<Vec<DailyReward>> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching monthly rewards");

        let url = self.reward_url("home");
        let headers = self.reward_headers();

        let response: RewardHomeResponse = self
            .client
            .request_with_headers::<RewardHomeResponse, ()>(Method::GET, &url, None, &headers)
            .await?;

        let rewards = response
            .awards
            .into_iter()
            .map(|item| DailyReward::new(item.name, item.count, item.icon))
            .collect();

        Ok(rewards)
    }

    async fn get_reward_status(&self) -> Result<DailyRewardStatus> {
        let game = self.config.game_id.display_name();
        tracing::debug!(game = game, "Fetching daily reward status");

        // HoYoLab throttles parallel daily-reward reads on one account with
        // retcode -1004.
        let info = self.get_reward_info().await?;
        let rewards = self.get_monthly_rewards().await?;

        let today_index = if info.is_signed {
            info.total_sign_day.saturating_sub(1) as usize
        } else {
            info.total_sign_day as usize
        };

        let today_reward = rewards.get(today_index).cloned();

        Ok(DailyRewardStatus::new(info, today_reward, rewards))
    }

    async fn claim_daily_reward(&self) -> Result<ClaimResult> {
        let game = self.config.game_id.display_name();
        tracing::info!(game = game, "Claiming daily reward");

        let pre_info = self.get_reward_info().await?;
        if pre_info.is_signed {
            tracing::debug!(game = game, "Daily reward already claimed");
            let status = self.get_reward_status().await?;
            return Ok(ClaimResult::already_claimed(
                status.today_reward,
                status.info,
            ));
        }

        let url = self.reward_url("sign");
        let headers = self.reward_headers();

        let sign = match self
            .client
            .request_with_headers::<SignResponse, ()>(Method::POST, &url, None, &headers)
            .await
        {
            Ok(sign) => sign,
            Err(Error::Client(ClientError::ApiError { code, .. }))
                if retcode::has_kind(code, RetcodeKind::AlreadyClaimed) =>
            {
                tracing::debug!(game = game, "Sign endpoint reports today already signed");
                let status = self.get_reward_status().await?;
                return Ok(ClaimResult::already_claimed(
                    status.today_reward,
                    status.info,
                ));
            }
            Err(e) => return Err(e),
        };

        let status = self.get_reward_status().await?;

        if !status.info.is_signed {
            tracing::warn!(
                game = game,
                risk_code = ?sign.risk_code,
                geetest = sign.gt.is_some() || sign.challenge.is_some(),
                "Sign returned success but the reward is still unclaimed"
            );
            return Err(Error::ClaimNotRegistered {
                risk_code: sign.risk_code,
            });
        }

        tracing::info!(
            game = game,
            reward_name = ?status.today_reward.as_ref().map_or("Unknown", |r| r.name.as_str()),
            "Daily reward claimed successfully"
        );

        match status.today_reward {
            Some(reward) => Ok(ClaimResult::success(reward, status.info)),
            None => Ok(ClaimResult::error(
                "Claim succeeded but reward details unavailable",
                status.info,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;
    use tokio::sync::Mutex;
    use tokio::sync::oneshot;

    #[derive(Debug, Clone)]
    struct TestRequest {
        method: String,
        target: String,
    }

    #[derive(Debug, Clone)]
    struct TestResponse {
        status: u16,
        body: String,
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
                                    #[expect(
                                        clippy::let_underscore_must_use,
                                        reason = "best-effort response write to a test client that may have disconnected"
                                    )]
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
                #[expect(
                    clippy::let_underscore_must_use,
                    reason = "shutdown receiver may already be gone when the server is dropped"
                )]
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
            if let Some(chunk) = buf.get(..read) {
                raw.extend_from_slice(chunk);
            }
            if let Some(pos) = find_header_end(&raw) {
                break pos;
            }
        };

        let head_bytes = raw
            .get(..header_end)
            .expect("header_end is a valid offset within raw");
        let head = String::from_utf8_lossy(head_bytes).to_string();
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

        let mut body = raw
            .get(header_end + 4..)
            .expect("header end offset is within raw")
            .to_vec();
        while body.len() < content_length {
            let read = stream.read(&mut buf).await.ok()?;
            if read == 0 {
                break;
            }
            if let Some(chunk) = buf.get(..read) {
                body.extend_from_slice(chunk);
            }
        }

        Some(TestRequest { method, target })
    }

    fn find_header_end(bytes: &[u8]) -> Option<usize> {
        bytes.windows(4).position(|window| window == b"\r\n\r\n")
    }

    async fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
        let reason = match response.status {
            500 => "Internal Server Error",
            _ => "OK",
        };
        let body = response.body;
        let reply = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.status,
            reason,
            body.len(),
            body
        );
        stream.write_all(reply.as_bytes()).await
    }

    fn ok(body: &str) -> TestResponse {
        TestResponse {
            status: 200,
            body: body.to_string(),
        }
    }

    fn leak_string(value: String) -> &'static str {
        Box::leak(value.into_boxed_str())
    }

    fn test_config(base_url: &str) -> &'static HoyolabDailyRewardConfig {
        Box::leak(Box::new(HoyolabDailyRewardConfig {
            reward_url: leak_string(format!("{base_url}/event/luna/test")),
            act_id: "act123",
            sign_game: "testgame",
            game_id: GameId::GenshinImpact,
        }))
    }

    /// Switch `/info` to signed after the POST so the successful path returns
    /// the first award.
    fn sign_flow_handler(
        awards: &'static str,
        sign_body: &'static str,
    ) -> Arc<dyn Fn(&TestRequest) -> TestResponse + Send + Sync> {
        let signed = AtomicBool::new(false);
        Arc::new(move |request| {
            if request.target.starts_with("/event/luna/test/info") {
                if signed.load(Ordering::SeqCst) {
                    ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":1}}"#)
                } else {
                    ok(
                        r#"{"retcode":0,"message":"OK","data":{"is_sign":false,"total_sign_day":0}}"#,
                    )
                }
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(awards)
            } else {
                signed.store(true, Ordering::SeqCst);
                ok(sign_body)
            }
        })
    }

    fn test_client(
        server: &TestServer,
        config: &'static HoyolabDailyRewardConfig,
    ) -> HoyolabDailyRewardClient {
        let auth_url = format!("{}/auth", server.base_url);
        let hoyolab =
            HoyolabClient::with_auth_check_url("uid", "token", auth_url).expect("create client");
        HoyolabDailyRewardClient::new(hoyolab, config)
    }

    #[tokio::test]
    async fn reward_status_handles_signed_day_zero_with_first_reward() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/event/luna/test/info") {
                ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":0}}"#)
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"A","cnt":1,"icon":"a.png"},{"name":"B","cnt":2,"icon":"b.png"}]}}"#,
                )
            } else {
                ok(r#"{"retcode":0,"message":"OK","data":{}}"#)
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let status = client
            .get_reward_status()
            .await
            .expect("status should load");

        assert!(status.info.is_signed);
        assert_eq!(status.info.total_sign_day, 0);
        assert_eq!(status.monthly_rewards.len(), 2);
        assert_eq!(
            status
                .today_reward
                .as_ref()
                .map(|reward| reward.name.as_str()),
            Some("A")
        );
    }

    #[tokio::test]
    async fn claim_daily_reward_already_claimed_skips_sign_endpoint() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/event/luna/test/info") {
                ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":1}}"#)
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Primogems","cnt":60,"icon":"primogem.png"}]}}"#,
                )
            } else {
                ok(r#"{"retcode":0,"message":"OK","data":{}}"#)
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let claim = client
            .claim_daily_reward()
            .await
            .expect("claim should return already-claimed result");

        assert!(
            !claim.success,
            "already claimed should not be marked successful"
        );
        assert_eq!(
            claim.message.as_deref(),
            Some("Already claimed today"),
            "already-claimed path should return stable message"
        );

        let requests = server.requests().await;
        let sign_calls = requests
            .iter()
            .filter(|request| {
                request.method == "POST" && request.target.starts_with("/event/luna/test/sign")
            })
            .count();
        assert_eq!(sign_calls, 0, "sign endpoint should not be called");
    }

    #[tokio::test]
    async fn claim_daily_reward_reports_missing_reward_details() {
        let server = TestServer::spawn(sign_flow_handler(
            r#"{"retcode":0,"message":"OK","data":{"awards":[]}}"#,
            r#"{"retcode":0,"message":"OK","data":{}}"#,
        ))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let claim = client
            .claim_daily_reward()
            .await
            .expect("claim should complete with fallback message");

        assert!(
            !claim.success,
            "missing reward details should be reported as non-success"
        );
        assert_eq!(
            claim.message.as_deref(),
            Some("Claim succeeded but reward details unavailable")
        );

        let requests = server.requests().await;
        let sign_calls = requests
            .iter()
            .filter(|request| {
                request.method == "POST" && request.target.starts_with("/event/luna/test/sign")
            })
            .count();
        assert_eq!(sign_calls, 1, "sign endpoint should be called exactly once");
    }

    #[tokio::test]
    async fn claim_daily_reward_rejects_sign_that_never_registers() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/event/luna/test/info") {
                ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":false,"total_sign_day":0}}"#)
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"},{"name":"Day2","cnt":1,"icon":"2.png"}]}}"#,
                )
            } else {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"risk_code":5001,"gt":"gt-token","challenge":"challenge-token"}}"#,
                )
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let error = client
            .claim_daily_reward()
            .await
            .expect_err("an unregistered sign must not report a claim");

        assert!(
            matches!(
                error,
                Error::ClaimNotRegistered {
                    risk_code: Some(5001)
                }
            ),
            "expected ClaimNotRegistered carrying the risk code, got {error:?}"
        );
    }

    #[tokio::test]
    async fn claim_daily_reward_succeeds_once_the_sign_registers() {
        let server = TestServer::spawn(sign_flow_handler(
            r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"},{"name":"Day2","cnt":1,"icon":"2.png"}]}}"#,
            r#"{"retcode":0,"message":"OK","data":{"code":"ok"}}"#,
        ))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let claim = client
            .claim_daily_reward()
            .await
            .expect("claim should succeed once the sign registers");

        assert!(claim.success);
        assert_eq!(
            claim.reward.as_ref().map(|reward| reward.name.as_str()),
            Some("Day1")
        );
        assert!(claim.info.is_signed);
    }

    #[tokio::test]
    async fn claim_daily_reward_maps_already_signed_retcode_to_already_claimed() {
        let server = TestServer::spawn(sign_flow_handler(
            r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"}]}}"#,
            r#"{"retcode":-5003,"message":"Already signed in today","data":null}"#,
        ))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let claim = client
            .claim_daily_reward()
            .await
            .expect("retcode -5003 should resolve to an already-claimed result");

        assert!(!claim.success);
        assert_eq!(claim.message.as_deref(), Some("Already claimed today"));
    }

    #[tokio::test]
    async fn a_sign_throttled_after_it_registered_resolves_to_already_claimed() {
        let signed = AtomicBool::new(false);
        let sign_attempts = AtomicUsize::new(0);
        let server = TestServer::spawn(Arc::new(move |request| {
            if request.target.starts_with("/event/luna/test/info") {
                if signed.load(Ordering::SeqCst) {
                    ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":1}}"#)
                } else {
                    ok(
                        r#"{"retcode":0,"message":"OK","data":{"is_sign":false,"total_sign_day":0}}"#,
                    )
                }
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"}]}}"#,
                )
            } else {
                // The first sign registers, then reports a throttle. The retry
                // sees the day already signed.
                signed.store(true, Ordering::SeqCst);
                if sign_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    ok(
                        r#"{"retcode":-110,"message":"Visits too frequently.","data":null}"#,
                    )
                } else {
                    ok(r#"{"retcode":-5003,"message":"Already signed in today","data":null}"#)
                }
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let claim = client
            .claim_daily_reward()
            .await
            .expect("a registered sign followed by a throttle resolves as already claimed");

        assert!(!claim.success);
        assert_eq!(claim.message.as_deref(), Some("Already claimed today"));
    }

    #[tokio::test]
    async fn reward_status_selects_signed_day_reward_by_index() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/event/luna/test/info") {
                ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":2}}"#)
            } else if request.target.starts_with("/event/luna/test/home") {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"},{"name":"Day2","cnt":1,"icon":"2.png"},{"name":"Day3","cnt":1,"icon":"3.png"}]}}"#,
                )
            } else {
                ok(r#"{"retcode":0,"message":"OK","data":{}}"#)
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let status = client
            .get_reward_status()
            .await
            .expect("status should load");

        assert_eq!(
            status
                .today_reward
                .as_ref()
                .map(|reward| reward.name.as_str()),
            Some("Day2"),
            "signed day 2 should map to index 1"
        );
    }

    #[tokio::test]
    async fn reward_status_reads_info_before_home() {
        let server = TestServer::spawn(Arc::new(|request| {
            if request.target.starts_with("/event/luna/test/info") {
                ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":1}}"#)
            } else {
                ok(r#"{"retcode":0,"message":"OK","data":{"awards":[]}}"#)
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        client
            .get_reward_status()
            .await
            .expect("fetch reward status");

        let order: Vec<&str> = server
            .requests()
            .await
            .iter()
            .map(|request| {
                if request.target.starts_with("/event/luna/test/info") {
                    "info"
                } else if request.target.starts_with("/event/luna/test/home") {
                    "home"
                } else {
                    "other"
                }
            })
            .collect();

        assert_eq!(
            order,
            vec!["info", "home"],
            "the daily-reward reads must not run concurrently"
        );
    }

    #[tokio::test]
    async fn rate_limited_info_retries_and_resolves_status() {
        let info_attempts = AtomicUsize::new(0);
        let server = TestServer::spawn(Arc::new(move |request| {
            if request.target.starts_with("/event/luna/test/info") {
                if info_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    ok(
                        r#"{"retcode":-1004,"message":"Too many attempts. Please try again later.","data":null}"#,
                    )
                } else {
                    ok(r#"{"retcode":0,"message":"OK","data":{"is_sign":true,"total_sign_day":1}}"#)
                }
            } else {
                ok(
                    r#"{"retcode":0,"message":"OK","data":{"awards":[{"name":"Day1","cnt":1,"icon":"1.png"}]}}"#,
                )
            }
        }))
        .await;

        let config = test_config(&server.base_url);
        let client = test_client(&server, config);
        let status = client
            .get_reward_status()
            .await
            .expect("a retry returns the reward status");

        assert!(status.info.is_signed);
    }
}
