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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // Test data structure for responses
    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct TestData {
        id: u32,
        name: String,
    }

    // =========================================================================
    // HoyolabApiResponse tests
    // =========================================================================

    #[test]
    fn test_hoyolab_response_success() {
        let json = r#"{
            "retcode": 0,
            "message": "OK",
            "data": {"id": 1, "name": "test"}
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), 0);
        assert_eq!(response.message(), "OK");
        assert!(response.is_success());

        let data = response.into_data().expect("should have data");
        assert_eq!(data.id, 1);
        assert_eq!(data.name, "test");
    }

    #[test]
    fn test_hoyolab_response_error() {
        let json = r#"{
            "retcode": -1,
            "message": "Invalid token",
            "data": null
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), -1);
        assert_eq!(response.message(), "Invalid token");
        assert!(!response.is_success());
        assert!(response.into_data().is_none());
    }

    #[test]
    fn test_hoyolab_response_into_result_success() {
        let json = r#"{
            "retcode": 0,
            "message": "OK",
            "data": {"id": 42, "name": "success"}
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");
        let result = response.into_result();

        assert!(result.is_ok());
        let data = result.expect("should be ok");
        assert_eq!(data.id, 42);
        assert_eq!(data.name, "success");
    }

    #[test]
    fn test_hoyolab_response_into_result_api_error() {
        let json = r#"{
            "retcode": 10001,
            "message": "Rate limited",
            "data": null
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.expect_err("should be error");
        assert!(matches!(err, ClientError::ApiError { code: 10001, .. }));
        assert_eq!(err.api_error_code(), Some(10001));
    }

    #[test]
    fn test_hoyolab_response_into_result_null_data() {
        // Success code but null data should be an error
        let json = r#"{
            "retcode": 0,
            "message": "OK",
            "data": null
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.expect_err("should be error due to null data");
        assert!(matches!(err, ClientError::ApiError { code: 0, .. }));
    }

    // =========================================================================
    // KuroApiResponse tests
    // =========================================================================

    #[test]
    fn test_kuro_response_success_code_zero() {
        let json = r#"{
            "code": 0,
            "message": "success",
            "data": {"id": 1, "name": "test"}
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), 0);
        assert!(response.is_success(), "code 0 should be success");
    }

    #[test]
    fn test_kuro_response_success_code_200() {
        let json = r#"{
            "code": 200,
            "message": "OK",
            "data": {"id": 1, "name": "test"}
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), 200);
        assert!(response.is_success(), "code 200 should be success");
    }

    #[test]
    fn test_kuro_response_error() {
        let json = r#"{
            "code": 1005,
            "message": "Retry requested",
            "data": null
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), 1005);
        assert!(!response.is_success(), "code 1005 should not be success");
        assert_eq!(response.message(), "Retry requested");
    }

    #[test]
    fn test_kuro_response_into_result_success() {
        let json = r#"{
            "code": 200,
            "message": "OK",
            "data": {"id": 123, "name": "kuro_data"}
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");
        let result = response.into_result();

        assert!(result.is_ok());
        let data = result.expect("should be ok");
        assert_eq!(data.id, 123);
        assert_eq!(data.name, "kuro_data");
    }

    #[test]
    fn test_kuro_response_into_result_error() {
        let json = r#"{
            "code": 500,
            "message": "Internal error",
            "data": null
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");
        let result = response.into_result();

        assert!(result.is_err());
        let err = result.expect_err("should be error");
        assert!(matches!(err, ClientError::ApiError { code: 500, .. }));
    }

    // =========================================================================
    // Default is_success behavior tests
    // =========================================================================

    #[test]
    fn test_default_is_success_checks_code_zero() {
        // HoyolabApiResponse uses default is_success (code == 0)
        let response = HoyolabApiResponse {
            retcode: 0,
            message: "OK".to_string(),
            data: Some(TestData {
                id: 1,
                name: "test".to_string(),
            }),
        };
        assert!(response.is_success());

        let response = HoyolabApiResponse {
            retcode: 1,
            message: "Error".to_string(),
            data: None::<TestData>,
        };
        assert!(!response.is_success());
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_response_with_empty_message() {
        let json = r#"{
            "retcode": 0,
            "message": "",
            "data": {"id": 1, "name": "test"}
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.message(), "");
        assert!(response.is_success());
    }

    #[test]
    fn test_response_with_negative_code() {
        let json = r#"{
            "retcode": -10,
            "message": "Negative error",
            "data": null
        }"#;

        let response: HoyolabApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert_eq!(response.code(), -10);
        assert!(!response.is_success());
    }

    #[test]
    fn test_kuro_code_1_is_not_success() {
        // Kuro only accepts 0 and 200 as success
        let json = r#"{
            "code": 1,
            "message": "Unknown",
            "data": null
        }"#;

        let response: KuroApiResponse<TestData> =
            serde_json::from_str(json).expect("should deserialize");

        assert!(
            !response.is_success(),
            "code 1 should not be success for Kuro"
        );
    }

    // =========================================================================
    // Debug trait tests
    // =========================================================================

    #[test]
    fn test_hoyolab_response_debug() {
        let response = HoyolabApiResponse {
            retcode: 0,
            message: "OK".to_string(),
            data: Some(TestData {
                id: 1,
                name: "test".to_string(),
            }),
        };

        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("HoyolabApiResponse"));
        assert!(debug_str.contains("retcode"));
    }

    #[test]
    fn test_kuro_response_debug() {
        let response = KuroApiResponse {
            code: 200,
            message: "OK".to_string(),
            data: Some(TestData {
                id: 1,
                name: "test".to_string(),
            }),
        };

        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("KuroApiResponse"));
        assert!(debug_str.contains("code"));
    }
}
