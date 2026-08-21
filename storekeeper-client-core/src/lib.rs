//! Shared HTTP client abstractions for Storekeeper API clients.
//!
//! This crate provides common types and utilities for building game API
//! clients, including:
//!
//! - Base error types that can be extended by specific clients
//! - API response traits for handling different response formats
//! - HTTP client builder with common configuration options
//! - Retry utilities with exponential backoff and jitter
//!
//! # Examples
//!
//! ```
//! use storekeeper_client_core::HttpClientBuilder;
//!
//! let client = HttpClientBuilder::new()
//!     .header_static("x-custom-header", "value")
//!     .build()?;
//!
//! let with_retry = HttpClientBuilder::new().build_with_retry(3)?;
//! # Ok::<(), storekeeper_client_core::ClientError>(())
//! ```

mod client;
mod error;
mod response;
mod retry;

pub use client::DEFAULT_USER_AGENT;
pub use client::HttpClientBuilder;
pub use error::ClientError;
pub use error::Result;
pub use reqwest_middleware::ClientWithMiddleware;
pub use response::ApiResponse;
pub use response::HoyolabApiResponse;
pub use response::KuroApiResponse;
pub use retry::DEFAULT_BASE_DELAY_MS;
pub use retry::DEFAULT_MAX_DELAY_MS;
pub use retry::DEFAULT_MAX_RETRIES;
pub use retry::RetryConfig;
pub use retry::retry_with_backoff;
