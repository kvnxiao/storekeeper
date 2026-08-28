//! Error types for the HoYoLab API client.

use crate::retcode;
use crate::retcode::RetcodeKind;
pub use storekeeper_client_core::ClientError;
use thiserror::Error;

/// Error type for HoYoLab API operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Base client error (HTTP, deserialization, API errors).
    #[error(transparent)]
    Client(#[from] ClientError),

    /// Temporary throttle reported in the response body.
    #[error("HoYoLab rate limit (retcode {retcode}): {message}")]
    RateLimited {
        /// API retcode for the throttle.
        retcode: i32,
        /// API message returned with the retcode.
        message: String,
    },

    /// The sign endpoint accepted the request but the follow-up status remains
    /// unsigned.
    #[error("Sign accepted but the reward is still unclaimed (risk_code {risk_code:?})")]
    ClaimNotRegistered {
        /// Risk-control code returned by the sign endpoint, when present.
        risk_code: Option<i32>,
    },
}

impl Error {
    /// Return whether retrying the error can succeed.
    ///
    /// Return `false` for API errors classified as rate-limited, cookie,
    /// account, already-claimed, or redemption failures. Return `true` for
    /// other errors, including the `RateLimited` variant, transport and
    /// deserialization failures, geetest and request errors, and unknown
    /// API retcodes.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Client(ClientError::ApiError { code, .. }) => {
                retcode::lookup(*code).is_none_or(|entry| match entry.kind {
                    RetcodeKind::Cooldown | RetcodeKind::Geetest | RetcodeKind::Request => true,
                    RetcodeKind::RateLimited
                    | RetcodeKind::Cookie
                    | RetcodeKind::Account
                    | RetcodeKind::AlreadyClaimed
                    | RetcodeKind::Redemption => false,
                })
            }
            Self::Client(ClientError::HttpStatus { status, .. }) => {
                matches!(status, 408 | 429 | 500..=599)
            }
            _ => true,
        }
    }
}

/// Result type alias using the HoYoLab Error type.
pub type Result<T> = std::result::Result<T, Error>;
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Client(ClientError::from(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Client(ClientError::from(err))
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        Self::Client(ClientError::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_crosses_task_boundaries() {
        const fn assert_send<T: Send>() {}
        const fn assert_sync<T: Sync>() {}
        assert_send::<Error>();
        assert_sync::<Error>();
    }

    #[test]
    fn a_self_clearing_failure_is_recoverable() {
        let recoverable: [Error; 7] = [
            Error::RateLimited {
                retcode: -1004,
                message: "Too many attempts. Please try again later.".to_string(),
            },
            Error::ClaimNotRegistered { risk_code: None },
            Error::Client(ClientError::api_error(-3102, "geetest")),
            Error::Client(ClientError::api_error(-10001, "malformed")),
            Error::Client(ClientError::api_error(-1002, "unrecognized")),
            Error::Client(ClientError::http_status(503, "service unavailable")),
            Error::Client(ClientError::Middleware("connector failed".to_string())),
        ];

        for error in recoverable {
            assert!(error.is_recoverable(), "{error} is recoverable");
        }
    }

    #[test]
    fn a_failure_needing_the_user_or_the_next_day_is_not_recoverable() {
        let permanent: [Error; 6] = [
            Error::Client(ClientError::api_error(-100, "not logged in")),
            Error::Client(ClientError::api_error(-10002, "no game account")),
            Error::Client(ClientError::api_error(-5003, "already signed")),
            Error::Client(ClientError::api_error(10101, "30 accounts per day")),
            Error::Client(ClientError::api_error(-2016, "redemption cooldown")),
            Error::Client(ClientError::http_status(403, "forbidden")),
        ];

        for error in permanent {
            assert!(!error.is_recoverable(), "{error} is not recoverable");
        }
    }
}
