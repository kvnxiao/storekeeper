//! Shared HTTP client abstractions for Storekeeper API clients.
//!
//! This crate provides common types and utilities for building game API clients,
//! including:
//!
//! - Base error types that can be extended by specific clients
//! - API response traits for handling different response formats
//! - HTTP client builder with common configuration options
//! - Retry utilities with exponential backoff and jitter
//!
//! # Example
//!
//! ```rust,ignore
//! use storekeeper_client_core::{ClientError, HttpClientBuilder, ApiResponse};
//!
//! // Create an HTTP client with custom headers
//! let client = HttpClientBuilder::new()
//!     .header_static("x-custom-header", "value")
//!     .build()?;
//!
//! // Or create a client with automatic HTTP-level retries
//! let client_with_retry = HttpClientBuilder::new()
//!     .build_with_retry(3)?; // Max 3 retries
//! ```

pub mod client;
pub mod error;
pub mod response;
pub mod retry;

pub use client::{DEFAULT_USER_AGENT, HttpClientBuilder};
pub use error::{ClientError, Result};
pub use reqwest_middleware::ClientWithMiddleware;
pub use response::{ApiResponse, HoyolabApiResponse, KuroApiResponse};
pub use retry::RetryConfig;
