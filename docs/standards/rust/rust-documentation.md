# Documentation Requirements

## Public API Documentation

Every public item must have documentation:

```rust
/// Processes the input data and returns a processed result.
///
/// # Arguments
///
/// * `input` - The input string to process
/// * `options` - Processing options
///
/// # Returns
///
/// Returns a `Result` containing the processed data or an error.
///
/// # Errors
///
/// This function will return an error if:
/// - The input is empty
/// - The input exceeds maximum length
/// - Parsing fails
///
/// # Examples
///
/// ```
/// use my_library::{process, Options};
///
/// let result = process("hello", Options::default())?;
/// assert_eq!(result.value(), "HELLO");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn process(input: &str, options: Options) -> Result<ProcessedData> {
    // Implementation
}
```

## Module Documentation

```rust
//! Configuration management for the application.
//!
//! This module provides types and functions for loading, validating,
//! and managing application configuration.
//!
//! # Examples
//!
//! ```
//! use my_library::config::Config;
//!
//! let config = Config::from_file("config.toml")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
```
