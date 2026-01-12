# Code Quality Standards

## Rust Edition

Use the latest stable Rust edition:

```toml
[package]
edition = "2024"
rust-version = "1.85"  # Specify MSRV
```

## Code Organization

```rust
// src/lib.rs or src/main.rs
mod config;
mod error;
mod models;
mod api;
mod utils;

pub use error::{Error, Result};
pub use config::Config;
```

## Prefer Enums Over Booleans

```rust
// Bad
fn process(data: &str, is_verbose: bool, is_strict: bool) { }

// Good
#[derive(Debug, Clone, Copy)]
pub enum Verbosity { Quiet, Normal, Verbose }

#[derive(Debug, Clone, Copy)]
pub enum ValidationMode { Lenient, Strict }

fn process(data: &str, verbosity: Verbosity, validation: ValidationMode) { }
```

## Avoid Stringly-Typed Code

```rust
// Bad
fn get_user_by_type(user_type: &str) -> Result<User> {
    match user_type {
        "admin" => { /* ... */ },
        "regular" => { /* ... */ },
        _ => Err(Error::InvalidUserType),
    }
}

// Good
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserType { Admin, Regular, Guest }

fn get_user_by_type(user_type: UserType) -> Result<User> {
    match user_type {
        UserType::Admin => { /* ... */ },
        UserType::Regular => { /* ... */ },
        UserType::Guest => { /* ... */ },
    }
}
```
