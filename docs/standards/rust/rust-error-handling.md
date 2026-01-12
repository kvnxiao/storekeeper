# Error Handling

## Use `thiserror` for Library Errors

**Never use string-based errors.** Always define explicit error types using `thiserror`.

**Add to `Cargo.toml`:**
```toml
[dependencies]
thiserror = "2.0"
```

**Example Error Definition:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyLibraryError {
    #[error("Failed to parse configuration: {0}")]
    ConfigParseFailed(String),

    #[error("Database connection failed")]
    DatabaseConnectionFailed(#[from] sqlx::Error),

    #[error("Invalid input: {field} must be {constraint}")]
    ValidationError {
        field: String,
        constraint: String,
    },

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound {
        resource_type: String,
        id: String,
    },

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MyLibraryError>;
```

## Use `anyhow` for Application Errors

For application code (not libraries), `anyhow` provides convenient error handling with context.

**Add to `Cargo.toml`:**
```toml
[dependencies]
anyhow = "1.0"
```

**Example Usage:**
```rust
use anyhow::{Context, Result};

fn process_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;

    let parsed = parse_content(&content)
        .context("Failed to parse file content")?;

    Ok(parsed)
}
```

## Error Handling Best Practices

1. **Never use `unwrap()`** - Always handle errors explicitly
2. **Never use `expect()`** in production code - Use proper error propagation
3. **Use `?` operator** for error propagation
4. **Add context** to errors using `.context()` or `.with_context()`
5. **Document errors** in function documentation
6. **Create specific error variants** for different failure modes

**Bad:**
```rust
fn bad_example(data: &str) -> String {
    let parsed: u32 = data.parse().unwrap(); // NEVER DO THIS
    format!("Value: {}", parsed)
}
```

**Good:**
```rust
fn good_example(data: &str) -> Result<String, MyLibraryError> {
    let parsed: u32 = data.parse()
        .map_err(|e| MyLibraryError::ValidationError {
            field: "data".to_string(),
            constraint: format!("must be a valid u32: {}", e),
        })?;

    Ok(format!("Value: {}", parsed))
}
```
