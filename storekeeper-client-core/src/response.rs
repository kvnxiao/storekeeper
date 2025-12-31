//! API response handling traits and types.

use serde::de::DeserializeOwned;

use crate::error::ClientError;

/// Trait for API response wrappers.
///
/// Different APIs return responses in different formats, but they typically
/// share a common pattern: a status code, message, and optional data.
pub trait ApiResponse: Sized + DeserializeOwned {
    /// The type of the data payload.
    type Data: DeserializeOwned;

    /// Returns the response code (0 typically means success).
    fn code(&self) -> i32;

    /// Returns the response message.
    fn message(&self) -> &str;

    /// Extracts the data payload, consuming the response.
    fn into_data(self) -> Option<Self::Data>;

    /// Returns true if this response represents a success.
    fn is_success(&self) -> bool {
        self.code() == 0
    }

    /// Converts this response into a Result, returning the data on success
    /// or a `ClientError::ApiError` on failure.
    ///
    /// # Errors
    ///
    /// Returns `ClientError::ApiError` if the response code indicates failure
    /// or if the data payload is `None`.
    fn into_result(self) -> Result<Self::Data, ClientError> {
        let code = self.code();
        let is_success = self.is_success();

        if !is_success {
            return Err(ClientError::api_error(code, self.message()));
        }

        self.into_data()
            .ok_or_else(|| ClientError::api_error(code, "Response data is null"))
    }
}

/// Standard API response structure used by HoYoLab APIs.
///
/// This struct handles the `retcode`/`message`/`data` format used by HoYoLab.
#[derive(Debug, serde::Deserialize)]
pub struct HoyolabApiResponse<T> {
    /// Response code (0 = success for HoYoLab APIs).
    pub retcode: i32,
    /// Response message.
    pub message: String,
    /// Response data payload.
    pub data: Option<T>,
}

impl<T: DeserializeOwned> ApiResponse for HoyolabApiResponse<T> {
    type Data = T;

    fn code(&self) -> i32 {
        self.retcode
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn into_data(self) -> Option<Self::Data> {
        self.data
    }
}

/// API response structure used by Kuro Games APIs.
///
/// This struct handles the `code`/`message`/`data` format used by Kuro Games.
/// Note: Kuro APIs accept both 0 and 200 as success codes.
#[derive(Debug, serde::Deserialize)]
pub struct KuroApiResponse<T> {
    /// Response code (0 or 200 = success for Kuro APIs).
    pub code: i32,
    /// Response message.
    pub message: String,
    /// Response data payload.
    pub data: Option<T>,
}

impl<T: DeserializeOwned> ApiResponse for KuroApiResponse<T> {
    type Data = T;

    fn code(&self) -> i32 {
        self.code
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn into_data(self) -> Option<Self::Data> {
        self.data
    }

    fn is_success(&self) -> bool {
        self.code == 0 || self.code == 200
    }
}
