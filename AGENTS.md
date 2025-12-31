# Rust Development Guidelines

## Overview
This document contains essential best practices and guidelines for AI agents and developers working on Rust projects. Following these principles ensures safe, maintainable, and idiomatic Rust code.

---

## Table of Contents
1. [Linter and Formatter Configuration](#linter-and-formatter-configuration)
2. [Error Handling](#error-handling)
3. [Defensive Programming](#defensive-programming)
4. [Code Quality Standards](#code-quality-standards)
5. [Unsafe Rust](#unsafe-rust)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation Requirements](#documentation-requirements)
8. [Performance Considerations](#performance-considerations)
9. [Dependency Management](#dependency-management)
10. [Multi-Crate Workspaces](#multi-crate-workspaces)

---

## Linter and Formatter Configuration

### Clippy

Always use this clippy lint configuration in `Cargo.toml`:
```toml
[lints.clippy]
# Deny all warnings - treat them as errors
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
cargo = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

# Unimplemented items can be left as warnings
todo = "warn"
unimplemented = "warn"

# Correctness (deny)
cast_lossless = "deny"
cast_possible_truncation = "deny"
cast_possible_wrap = "deny"
cast_precision_loss = "deny"
cast_sign_loss = "deny"

# Performance (warn/deny)
inefficient_to_string = "deny"
large_enum_variant = "warn"
large_stack_arrays = "warn"
needless_pass_by_value = "warn"

# Style (warn)
missing_errors_doc = "warn"
missing_panics_doc = "warn"

# Allow multiple crate versions caused by transitive dependencies
multiple_crate_versions = "allow"
```

Always run Clippy with `cargo clippy --workspace --all-targets --all-features` to catch potential issues early, followed by a `cargo fmt --check` to ensure code is formatted correctly.

---

## Error Handling

### Use `thiserror` for Library Errors
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

### Use `anyhow` for Application Errors
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

### Error Handling Best Practices

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

---

## Defensive Programming

### Core Principles

1. **Validate all inputs** at API boundaries
2. **Use the type system** to enforce invariants
3. **Avoid panics** in library code
4. **Check preconditions** explicitly
5. **Handle all error cases** exhaustively

### Input Validation

```rust
pub fn process_user_input(input: &str) -> Result<ProcessedData> {
    // Validate input is not empty
    if input.is_empty() {
        return Err(MyLibraryError::ValidationError {
            field: "input".to_string(),
            constraint: "must not be empty".to_string(),
        });
    }
    
    // Validate length constraints
    if input.len() > MAX_INPUT_LENGTH {
        return Err(MyLibraryError::ValidationError {
            field: "input".to_string(),
            constraint: format!("must not exceed {} characters", MAX_INPUT_LENGTH),
        });
    }
    
    // Continue processing...
    Ok(ProcessedData::new(input))
}
```

### Use Builder Pattern for Complex Types

```rust
#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
    timeout: Duration,
}

pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            timeout: None,
        }
    }
    
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn build(self) -> Result<Config> {
        Ok(Config {
            host: self.host.ok_or_else(|| MyLibraryError::ValidationError {
                field: "host".to_string(),
                constraint: "must be specified".to_string(),
            })?,
            port: self.port.ok_or_else(|| MyLibraryError::ValidationError {
                field: "port".to_string(),
                constraint: "must be specified".to_string(),
            })?,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}
```

### Newtype Pattern for Type Safety

```rust
// Instead of using raw types that can be confused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductId(u64);

impl UserId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Now you can't accidentally mix up user IDs and product IDs
fn get_user(id: UserId) -> Result<User> {
    // Type system prevents passing ProductId here
    todo!()
}
```

### Safe Index Access

```rust
// Bad: Can panic
let item = vec[index];

// Good: Handle out of bounds
let item = vec.get(index)
    .ok_or(MyLibraryError::NotFound {
        resource_type: "item".to_string(),
        id: index.to_string(),
    })?;
```

### Safe Arithmetic

```rust
// Bad: Can overflow in debug, wraps in release
let result = a + b;

// Good: Handle overflow explicitly
let result = a.checked_add(b)
    .ok_or(MyLibraryError::ValidationError {
        field: "sum".to_string(),
        constraint: "result would overflow".to_string(),
    })?;

// Or use saturating/wrapping explicitly when appropriate
let result = a.saturating_add(b); // When overflow should cap
let result = a.wrapping_add(b);   // When overflow should wrap (make explicit)
```

---

## Code Quality Standards

### Rust Edition and Features

Use the latest stable Rust edition:

```toml
[package]
edition = "2024"
rust-version = "1.85"  # Specify MSRV
```

### Code Organization

```rust
// Module structure should be clear and logical
// src/lib.rs or src/main.rs
mod config;
mod error;
mod models;
mod api;
mod utils;

pub use error::{Error, Result};
pub use config::Config;
```

### Prefer Enums Over Booleans

```rust
// Bad
fn process(data: &str, is_verbose: bool, is_strict: bool) { }

// Good
#[derive(Debug, Clone, Copy)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Debug, Clone, Copy)]
pub enum ValidationMode {
    Lenient,
    Strict,
}

fn process(data: &str, verbosity: Verbosity, validation: ValidationMode) { }
```

### Avoid Stringly-Typed Code

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
pub enum UserType {
    Admin,
    Regular,
    Guest,
}

fn get_user_by_type(user_type: UserType) -> Result<User> {
    match user_type {
        UserType::Admin => { /* ... */ },
        UserType::Regular => { /* ... */ },
        UserType::Guest => { /* ... */ },
    }
}
```

### Use `#[must_use]` Strategically

**Core principle:** Use `#[must_use]` when ignoring a value would likely be a mistake or indicate a logic error.

**ALWAYS include a custom message** to provide context to the caller about why the value matters.

#### When to Use `#[must_use]`

**1. Results and Error Types**

Types like `Result` already have `#[must_use]` built-in, but add it to your custom error-returning functions for emphasis:

```rust
#[must_use = "errors must be handled, not silently ignored"]
pub fn validate_config(config: &Config) -> Result<(), ValidationError> {
    // ...
}
```

**2. Builder Patterns**

When methods return a modified version rather than mutating in place:

```rust
#[must_use = "builders must be used to construct the final value"]
pub struct QueryBuilder {
    filters: Vec<Filter>,
}

impl QueryBuilder {
    #[must_use = "this returns a new builder with the filter added"]
    pub fn filter(mut self, f: Filter) -> Self {
        self.filters.push(f);
        self
    }

    #[must_use = "call .execute() to run the query"]
    pub fn build(self) -> Query {
        Query { filters: self.filters }
    }
}
```

**3. Expensive Computations**

Functions that do significant work but don't have side effects:

```rust
#[must_use = "computing the hash is expensive; use the result"]
pub fn compute_hash(data: &[u8]) -> Hash {
    // CPU-intensive hashing...
}

#[must_use = "parsing is expensive; don't discard the result"]
pub fn parse_document(input: &str) -> Document {
    // Complex parsing logic...
}
```

**4. Values Representing State Changes**

When ignoring the return means losing important state information:

```rust
#[must_use = "the guard must be held to maintain the lock"]
pub fn acquire_lock(&self) -> LockGuard<'_> {
    // ...
}

#[must_use = "the handle must be stored to keep the connection alive"]
pub fn connect(addr: &str) -> ConnectionHandle {
    // ...
}

#[must_use = "the previous value may need to be processed"]
pub fn swap(&mut self, new_value: T) -> T {
    std::mem::replace(&mut self.value, new_value)
}
```

**5. Types That Should Never Be Ignored**

Apply `#[must_use]` to the type itself, not just functions:

```rust
#[must_use = "futures do nothing unless polled"]
pub struct MyFuture<T> {
    // ...
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct FilteredIterator<I> {
    // ...
}
```

#### Bad Examples (Missing Context)

```rust
// ❌ BAD: No message - caller doesn't know why it matters
#[must_use]
pub fn create_builder() -> Builder {
    Builder::new()
}

// ❌ BAD: Generic message that doesn't help
#[must_use = "don't ignore this"]
pub fn process(data: &Data) -> ProcessedData {
    // ...
}
```

#### Good Examples (Clear Context)

```rust
// ✅ GOOD: Clear explanation of consequence
#[must_use = "the builder must be used to construct the final Config"]
pub fn create_builder() -> ConfigBuilder {
    ConfigBuilder::new()
}

// ✅ GOOD: Explains what happens if ignored
#[must_use = "the processed data contains the transformation result"]
pub fn process(data: &Data) -> ProcessedData {
    // ...
}
```

#### When NOT to Use `#[must_use]`

```rust
// Don't use for side-effect functions where the return is optional info
pub fn log_event(event: &Event) -> usize {
    // Returns bytes written, but logging happened regardless
}

// Don't use for simple getters
pub fn len(&self) -> usize {
    self.items.len()
}

// Don't use when ignoring is a valid common case
pub fn try_recv(&self) -> Option<Message> {
    // Often called in a loop where None is expected
}
```

### Choosing Function Parameter Types

Follow the Rust API Guidelines for choosing between borrowed types, `impl AsRef<T>`, and `impl Into<T>`.

#### Decision Hierarchy

**1. Prefer borrowed types when you don't need ownership:**

```rust
// ✅ BEST: Accept &str when you only need to read
pub fn validate_name(name: &str) -> bool {
    !name.is_empty() && name.len() < 100
}

// ✅ BEST: Accept &Path when you only need to read
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

// Callers can pass &str, &String, or String (via deref)
validate_name("Alice");
validate_name(&my_string);
```

**2. Use `impl AsRef<T>` for maximum flexibility without ownership:**

```rust
// ✅ GOOD: Accept anything that can be borrowed as Path
pub fn read_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)?;
    // ...
}

// Callers can pass &str, String, PathBuf, &Path, etc.
read_config("config.toml");
read_config(PathBuf::from("config.toml"));
```

**3. Use `impl Into<T>` when you need ownership:**

```rust
// ✅ GOOD: Use impl Into when storing the value
pub struct User {
    name: String,
    email: String,
}

impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }
}

// Clean call site - no .to_string() needed
let user = User::new("Alice", "alice@example.com");
```

**4. Use concrete owned types when the API requires it or for performance:**

```rust
// Sometimes you need the concrete type for clarity or API constraints
pub fn set_buffer(buffer: Vec<u8>) {
    // Takes ownership explicitly
}

// Or when the function can be expensively repeated in a hot loop
pub fn hash_bytes(data: &[u8]) -> u64 {
    // ...
}
```

#### When to Use Each Approach

| Scenario                         | Recommended Type        | Example                           |
| -------------------------------- | ----------------------- | --------------------------------- |
| Read-only access                 | `&str`, `&Path`, `&[T]` | `fn print(msg: &str)`             |
| Read-only, flexible input        | `impl AsRef<T>`         | `fn read(path: impl AsRef<Path>)` |
| Need ownership, want flexibility | `impl Into<T>`          | `fn new(name: impl Into<String>)` |
| Need exact type                  | Concrete type           | `fn process(data: Vec<u8>)`       |

#### Builder Pattern

Builders that store values should use `impl Into<T>`:

```rust
impl ConfigBuilder {
    // ✅ GOOD: Builder stores the value, use impl Into
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    // ✅ GOOD: Clean chaining
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

// Clean usage
let config = ConfigBuilder::new()
    .host("localhost")  // No .to_string() needed
    .timeout(Duration::from_secs(30))
    .build()?;
```

#### Call-Site Conversions

When calling functions that require owned types, use `.into()` or specific methods:

```rust
// External API requires String
fn external_api(data: String) { /* ... */ }

// Use .into() for generic conversion
external_api("hello".into());

// Or use specific method when clearer
external_api("hello".to_string());
external_api(String::from("hello"));

// For PathBuf
let path: PathBuf = "/home/user".into();
let path = PathBuf::from("/home/user");  // Also fine
```

#### Trade-offs of `impl Into<T>`

**Pros:**
- Clean call sites
- Flexible - accepts multiple input types
- No allocation if caller already has owned value

**Cons:**
- Prevents use in trait objects (`dyn Trait`)
- Slightly more complex function signatures
- May hide allocations from the caller

#### When NOT to Use `impl Into<T>` or `impl AsRef<T>`

```rust
// Don't use when you need trait objects
trait Handler {
    // ❌ Can't use impl Trait in trait definitions
    fn handle(&self, msg: impl Into<String>);

    // ✅ Use concrete types instead
    fn handle(&self, msg: &str);
}

// Don't use for fallible conversions
fn parse_number(s: &str) -> Result<u32> {
    s.parse()  // Explicit parsing, not Into
}

// Don't use when conversion intent should be explicit
let formatted = format!("{}", number);  // Clear: formatting
let parsed: u32 = input.parse()?;       // Clear: parsing
```

#### Summary

1. **Default to borrowed types** (`&str`, `&Path`) when you don't need ownership
2. **Use `impl AsRef<T>`** for flexible borrowing across multiple types
3. **Use `impl Into<T>`** when you need ownership and want clean call sites
4. **Use concrete types** when the API requires it or for trait objects
5. **At call sites**, use `.into()` or specific conversions as needed

---

## Unsafe Rust

### Core Principle: Avoid `unsafe` at All Costs

**Unsafe Rust should be treated as a last resort.** The vast majority of Rust code should never need `unsafe` blocks.

### Why Avoid `unsafe`

1. **Defeats Rust's safety guarantees** - You lose memory safety, thread safety, and type safety
2. **Hard to audit** - Requires deep understanding of invariants and undefined behavior
3. **Maintenance burden** - Future changes can introduce subtle bugs
4. **Propagates risk** - Unsafety in one place can affect the entire codebase
5. **Better alternatives exist** - Safe wrappers and crates are usually available

### The `unsafe` Hierarchy (Prefer Earlier Options)

```
1. Pure safe Rust ✅ BEST
   ↓
2. Safe wrapper crates ✅ GOOD
   ↓
3. Well-audited unsafe in dependencies ⚠️ ACCEPTABLE
   ↓
4. Your own unsafe code ❌ AVOID
   ↓
5. Extensive unsafe code ❌❌ NEVER
```

### Use Safe Wrapper Crates Instead

Before writing `unsafe` code, **always search for a safe wrapper crate first**.

#### Common Safe Wrappers

**Windows API:**
```toml
# Bad: Using windows-sys (unsafe bindings)
[dependencies]
windows-sys = "0.52"

# Good: Using winsafe (safe wrapper)
[dependencies]
winsafe = "0.0.19"
```

```rust
// Bad: Unsafe Windows API calls
use windows_sys::Win32::System::Threading::*;

unsafe {
    let handle = CreateThread(
        std::ptr::null(),
        0,
        Some(thread_proc),
        std::ptr::null(),
        0,
        std::ptr::null_mut(),
    );
}

// Good: Safe wrapper
use winsafe::{self as w, prelude::*};

let handle = w::HTHREAD::CreateThread(
    None,
    0,
    thread_proc,
    None,
)?;  // Returns Result, no unsafe needed!
```

**POSIX/Unix APIs:**
```toml
# Instead of libc (unsafe)
[dependencies]
nix = "0.27"  # Safe POSIX wrapper
rustix = "0.38"  # Safe Unix system calls
```

**FFI Bindings:**
```toml
# Look for -sys crates with safe wrappers
libsqlite3-sys = "0.27"  # Unsafe bindings
rusqlite = "0.30"        # Safe wrapper ✅

openssl-sys = "0.9"      # Unsafe bindings  
openssl = "0.10"         # Safe wrapper ✅
```

**Memory Manipulation:**
```toml
# Instead of raw pointer manipulation
[dependencies]
bytemuck = "1.14"  # Safe transmutation
zerocopy = "0.7"   # Safe zero-copy parsing
```

#### Finding Safe Wrappers

1. **Search crates.io** for "{library}-rs" or "safe-{library}"
2. **Check "Reverse Dependencies"** of unsafe -sys crates
3. **Look for high-level crates** that wrap low-level bindings
4. **Ask the community** on Reddit, Discord, or Zulip

### When You Think You Need `unsafe`

**Ask these questions first:**

1. ❓ **Is there a safe wrapper crate?**
   - Check crates.io, lib.rs, and GitHub
   - Ask in Rust community forums

2. ❓ **Can I restructure to avoid it?**
   - Rethink the API design
   - Use safe abstractions like `Vec`, `Box`, `Rc`

3. ❓ **Can I use standard library types?**
   - `Vec` instead of raw arrays
   - `Box` instead of raw pointers
   - `Cell`/`RefCell` instead of interior mutability hacks

4. ❓ **Is this a premature optimization?**
   - Profile first
   - Safe code is usually fast enough

### If You Absolutely Must Use `unsafe`

If no safe alternative exists and you must use `unsafe`:

#### 1. Minimize the Surface Area

```rust
// Bad: Large unsafe block
pub fn process_data(data: &[u8]) -> Vec<u8> {
    unsafe {
        // 100 lines of unsafe code
        // ...
    }
}

// Good: Tiny unsafe block wrapped in safe function
pub fn process_data(data: &[u8]) -> Vec<u8> {
    // 95 lines of safe code
    // ...
    
    // Only the truly unsafe operation
    let value = unsafe {
        *ptr  // 1 line
    };
    
    // More safe code
    // ...
}
```

#### 2. Document EVERY Invariant

```rust
/// Reads a value from a raw pointer.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is non-null
/// - `ptr` is properly aligned for type `T`
/// - `ptr` points to a valid, initialized instance of `T`
/// - `ptr` is dereferenceable (points to allocated memory)
/// - The memory `ptr` points to is not accessed by any other code
///   during this function call
/// - The returned reference's lifetime doesn't outlive the pointee
unsafe fn read_from_ptr<T>(ptr: *const T) -> &T {
    &*ptr
}
```

#### 3. Wrap in Safe Abstractions

```rust
// Bad: Exposing unsafe interface
pub unsafe fn raw_operation(ptr: *mut u8) { }

// Good: Safe public API, unsafe internals
pub fn safe_operation(data: &mut [u8]) -> Result<()> {
    // Validate inputs
    if data.is_empty() {
        return Err(Error::EmptyData);
    }
    
    // Safe wrapper around unsafe operation
    unsafe {
        // SAFETY: We validated data is non-empty and properly aligned
        raw_operation_internal(data.as_mut_ptr())
    }
    
    Ok(())
}

unsafe fn raw_operation_internal(ptr: *mut u8) {
    // Actual unsafe code, not exposed
}
```

#### 4. Test Extensively

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unsafe_code_edge_cases() {
        // Test empty input
        // Test single element
        // Test boundary conditions
        // Test concurrent access (if applicable)
    }
    
    // Use Miri for undefined behavior detection
    // cargo +nightly miri test
}
```

#### 5. Run Miri

```bash
# Install Miri
rustup +nightly component add miri

# Run tests under Miri to detect undefined behavior
cargo +nightly miri test
```

### Red Flags That Should Make You Reconsider

- ❌ You're not 100% sure the unsafe code is correct
- ❌ You can't articulate all safety invariants
- ❌ The unsafe code interacts with external state
- ❌ You're using unsafe for "performance" without profiling
- ❌ The unsafe block is more than 5 lines
- ❌ You're using transmute without understanding it fully
- ❌ You're doing pointer arithmetic

### Safe Alternatives Checklist

Before writing `unsafe`, verify you've checked:

- [ ] Searched crates.io for safe wrappers
- [ ] Checked if standard library types can solve it
- [ ] Asked in Rust community (Reddit, Discord, forums)
- [ ] Profiled to confirm performance is actually a problem
- [ ] Reviewed similar crates to see their approach
- [ ] Considered restructuring the API to avoid the need

### Example: Choosing Safe Over Unsafe

```rust
// ❌ BAD: Manual unsafe memory management
pub struct Buffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Buffer {
    pub unsafe fn new(capacity: usize) -> Self {
        let layout = Layout::array::<u8>(capacity).unwrap();
        let ptr = alloc(layout);
        // ... unsafe pointer manipulation
    }
}

// ✅ GOOD: Use Vec (safe)
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
}

// ✅ ALSO GOOD: Use bytes crate (safe, optimized)
use bytes::BytesMut;

pub struct Buffer {
    data: BytesMut,
}
```

### Summary

- **Default position:** Never use `unsafe`
- **First resort:** Find a safe wrapper crate
- **Second resort:** Restructure to avoid needing it
- **Last resort:** Minimal, well-documented, tested `unsafe`
- **Remember:** Most performance problems are algorithmic, not safety-related

The Rust ecosystem has mature, safe wrappers for most use cases. Use them.

---

## Testing Guidelines

### Use `.expect()` in Tests

**In test code, prefer `.expect()` over `.unwrap()`** - the descriptive message makes test failures much easier to debug.

```rust
// ❌ BAD: No context when test fails
let result = process_input("valid").unwrap();

// ✅ GOOD: Clear message on failure
let result = process_input("valid").expect("process_input should succeed with valid input");
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let result = process_input("valid")
            .expect("process_input should succeed with valid input");
        assert_eq!(result.value(), 42);
    }

    #[test]
    fn test_invalid_input() {
        let result = process_input("");
        assert!(result.is_err(), "process_input should fail with empty input");

        let err = result.expect_err("expected an error");
        assert!(
            matches!(err, MyLibraryError::ValidationError { ref field, .. } if field == "input"),
            "expected ValidationError for 'input' field, got: {err:?}"
        );
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_unimplemented_feature() {
        unimplemented_feature();
    }
}
```

### Property-Based Testing

```toml
[dev-dependencies]
proptest = "1.0"
```

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_parse_roundtrip(s in "[a-z]{1,100}") {
            let parsed = parse(&s)?;
            let serialized = serialize(&parsed)?;
            prop_assert_eq!(s, serialized);
        }
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_test.rs
use my_library::*;

#[test]
fn test_end_to_end_workflow() {
    let config = Config::builder()
        .host("localhost")
        .port(8080)
        .build()
        .expect("Valid config");
    
    let client = Client::new(config);
    let result = client.process().expect("Process should succeed");
    
    assert!(result.is_valid());
}
```

---

## Documentation Requirements

### Public API Documentation

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

### Module Documentation

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

---

## Performance Considerations

### Prefer Borrowing Over Cloning

```rust
// Bad: Unnecessary clone
fn process(data: String) -> String {
    data.to_uppercase()
}

// Good: Borrow when possible
fn process(data: &str) -> String {
    data.to_uppercase()
}
```

### Use `Cow` for Conditional Cloning

```rust
use std::borrow::Cow;

fn process(input: &str) -> Cow<str> {
    if input.contains("special") {
        Cow::Owned(input.replace("special", "SPECIAL"))
    } else {
        Cow::Borrowed(input)
    }
}
```

### Allocations and Capacity

```rust
// Bad: Multiple reallocations
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// Good: Pre-allocate when size is known
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

### Avoid Unnecessary Copies

```rust
// Use references in iterations
for item in &collection {  // Not: for item in collection
    process(item);
}

// Use drain() when consuming is needed
for item in collection.drain(..) {
    consume(item);
}
```

---

## Dependency Management

### Specify Exact Versions for Applications

```toml
# In applications (binaries), pin exact versions
[dependencies]
serde = "=1.0.195"
tokio = "=1.35.1"
```

### Use Semantic Versioning for Libraries

```toml
# In libraries, use flexible versions
[dependencies]
serde = "1.0"
tokio = { version = "1.35", features = ["full"] }
```

### Enable Only Needed Features

```toml
[dependencies]
tokio = { version = "1.35", features = ["rt-multi-thread", "net", "macros"] }
serde = { version = "1.0", features = ["derive"] }
# Not: features = ["full"]
```

### Review Dependencies Regularly

```bash
# Check for outdated dependencies
cargo outdated

# Audit for security vulnerabilities
cargo audit

# Check for unused dependencies
cargo machete
```

---

## Summary Checklist

When writing or reviewing Rust code, ensure:

- [ ] All Clippy warnings enabled and addressed
- [ ] No `unwrap()`, `expect()`, or `panic!()` in library code
- [ ] All public APIs return `Result<T, E>` with explicit error types
- [ ] Errors defined using `thiserror` for libraries
- [ ] All inputs validated at API boundaries
- [ ] Type system used to enforce invariants
- [ ] Choose appropriate parameter types: borrowed (`&str`) > `impl AsRef<T>` > `impl Into<T>` > concrete owned
- [ ] No `unsafe` code (or minimal, well-documented if absolutely necessary)
- [ ] Searched for safe wrapper crates before using `unsafe`
- [ ] All public items documented with examples
- [ ] Tests cover success and error cases
- [ ] No unnecessary allocations or clones
- [ ] Dependencies minimized and features specified
- [ ] Code formatted with `rustfmt`
- [ ] No compiler warnings (`cargo build` clean)
- [ ] For workspaces: Crates at root level (not in `crates/` folder)
- [ ] For workspaces: Shared dependencies defined in `[workspace.dependencies]`
- [ ] For workspaces: No circular dependencies between crates

---

## Multi-Crate Workspaces

### Overview

Rust workspaces allow you to manage multiple related crates in a single repository. This is essential for larger projects that need to be split into logical components.

### Workspace Structure: Root-Level Crates

**Prefer placing workspace crates at the root level** rather than nested in a `crates/` directory.

#### ✅ Recommended Structure

```
my-project/
├── Cargo.toml          # Workspace root
├── Cargo.lock          # Shared lock file
├── my-core/            # Core library crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── my-cli/             # CLI binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── my-server/          # Server binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── my-utils/           # Utilities crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── README.md
```

#### ❌ Avoid (Nested in crates/ directory)

```
my-project/
├── Cargo.toml
├── crates/             # Unnecessary nesting
│   ├── my-core/
│   ├── my-cli/
│   ├── my-server/
│   └── my-utils/
└── README.md
```

**Why root-level is better:**
- Shorter import paths in IDEs
- Easier navigation - less nesting
- Simpler to understand project structure
- Matches common Rust ecosystem conventions
- Less typing when navigating directories

### Workspace Root `Cargo.toml`

```toml
[workspace]
members = [
    "my-core",
    "my-cli",
    "my-server",
    "my-utils",
]

# Workspace-wide settings
resolver = "2"

# Shared dependencies across all workspace members
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
anyhow = "1.0"

# Shared lints for all crates
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "deny"
pedantic = "deny"
unwrap_used = "deny"
expect_used = "deny"

# Optional: default crate (e.g., main CLI)
[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/my-project"

# Profiles apply to all workspace members
[profile.release]
opt-level = 3
lto = "thin"
strip = true

[profile.dev]
opt-level = 0
debug = true
```

### Member Crate `Cargo.toml`

```toml
# my-core/Cargo.toml
[package]
name = "my-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Use workspace dependencies
tokio.workspace = true
serde.workspace = true
thiserror.workspace = true

# Crate-specific dependencies
uuid = { version = "1.6", features = ["v4"] }

[dev-dependencies]
proptest = "1.4"

[lints]
workspace = true
```

```toml
# my-cli/Cargo.toml
[package]
name = "my-cli"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true

[[bin]]
name = "my-cli"
path = "src/main.rs"

[dependencies]
# Reference workspace crates
my-core = { path = "../my-core" }
my-utils = { path = "../my-utils" }

# Workspace dependencies
tokio.workspace = true
anyhow.workspace = true

# CLI-specific dependencies
clap = { version = "4.4", features = ["derive"] }

[lints]
workspace = true
```

### Workspace Best Practices

#### 1. Dependency Management

**Use workspace dependencies for shared crates:**

```toml
# Root Cargo.toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", default-features = false }

# Member crates can enable additional features
# my-server/Cargo.toml
[dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "net"] }
```

#### 2. Version Synchronization

**Keep related crates at the same version:**

```toml
# Use workspace.package for shared metadata
[workspace.package]
version = "0.2.0"  # Single source of truth
edition = "2024"
authors = ["Your Name <you@example.com>"]

# Member crates inherit
[package]
name = "my-core"
version.workspace = true
```

#### 3. Inter-Crate Dependencies

**Use path dependencies for workspace members:**

```toml
[dependencies]
my-core = { path = "../my-core", version = "0.2.0" }
```

**Benefits:**
- During development: uses local path
- When published: uses version from crates.io
- Ensures version compatibility

#### 4. Feature Organization

**Define features at workspace level when they affect multiple crates:**

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1.35", default-features = false }

# my-core/Cargo.toml
[features]
default = ["tokio-runtime"]
tokio-runtime = ["tokio/rt-multi-thread"]
full = ["tokio/full"]

[dependencies]
tokio = { workspace = true, optional = true }

# my-cli/Cargo.toml
[dependencies]
my-core = { path = "../my-core", features = ["tokio-runtime"] }
```

#### 5. Testing Across Crates

```bash
# Run all tests in workspace
cargo test

# Run tests for specific crate
cargo test -p my-core

# Run tests with all features
cargo test --all-features

# Run tests in specific crate with specific features
cargo test -p my-server --features "full"
```

### Workspace Commands

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p my-cli

# Check all workspace members
cargo check --workspace

# Update all dependencies
cargo update

# Publish crates (do in order of dependencies)
cd my-core && cargo publish
cd my-utils && cargo publish
cd my-cli && cargo publish

# Clean entire workspace
cargo clean
```

### Organizing Workspace Crates

**Naming conventions:**

```
project-name/
├── project-core/      # Main library
├── project-cli/       # CLI interface
├── project-server/    # Server implementation
├── project-client/    # Client library
├── project-macros/    # Procedural macros
├── project-types/     # Shared types
└── project-test-utils/ # Test utilities
```

**Guidelines:**
- Prefix crates with project name for clarity
- Use descriptive suffixes (-core, -cli, -server, etc.)
- Keep crate names lowercase with hyphens
- Binary crates: `{project}-{purpose}`
- Library crates: `{project}-{component}`

### Splitting Code into Crates

**When to create a new crate:**

✅ **Good reasons:**
- Different compilation units (lib vs bin)
- Independent versioning needed
- Reusable component for other projects
- Reduces compilation time by splitting large crates
- Clear architectural boundary
- Different sets of dependencies

❌ **Bad reasons:**
- "Organization" without clear boundary
- Premature optimization
- Creating unnecessary indirection

**Example split:**

```
# Before: Single crate
my-project/
└── src/
    ├── main.rs
    ├── db/
    ├── api/
    └── models/

# After: Workspace with clear boundaries
my-project/
├── my-db/          # Database layer (reusable)
├── my-models/      # Data models (reusable)
├── my-api/         # API server (binary)
└── my-cli/         # CLI tool (binary)
```

### Common Workspace Patterns

#### Pattern 1: Library + Multiple Binaries

```
project/
├── Cargo.toml      # Workspace root
├── project-core/   # Main library
├── cli/            # CLI binary
├── server/         # Server binary
└── admin/          # Admin tool binary
```

#### Pattern 2: Layered Architecture

```
project/
├── Cargo.toml
├── domain/         # Core business logic
├── infrastructure/ # Database, external APIs
├── application/    # Use cases, services
└── presentation/   # API, CLI, UI
```

#### Pattern 3: Plugin Architecture

```
project/
├── Cargo.toml
├── core/           # Core library
├── plugin-api/     # Plugin interface
├── plugin-a/       # Plugin implementation
├── plugin-b/       # Plugin implementation
└── cli/            # CLI that loads plugins
```

### Workspace Gotchas and Solutions

#### Problem: Dependency Version Conflicts

```toml
# Bad: Different versions in different crates
# my-core/Cargo.toml
[dependencies]
serde = "1.0.195"

# my-cli/Cargo.toml
[dependencies]
serde = "1.0.180"  # Conflict!

# Good: Use workspace dependencies
# Root Cargo.toml
[workspace.dependencies]
serde = "1.0.195"

# All member crates
[dependencies]
serde.workspace = true
```

#### Problem: Circular Dependencies

```toml
# ❌ BAD: Circular dependency
# my-core/Cargo.toml
[dependencies]
my-utils = { path = "../my-utils" }

# my-utils/Cargo.toml
[dependencies]
my-core = { path = "../my-core" }  # Circular!

# ✅ GOOD: Create third crate for shared types
# my-types/Cargo.toml (no dependencies on other workspace crates)

# my-core/Cargo.toml
[dependencies]
my-types = { path = "../my-types" }

# my-utils/Cargo.toml
[dependencies]
my-types = { path = "../my-types" }
```

#### Problem: Slow Compile Times

**Solution: Split into smaller crates and use features:**

```toml
# Disable default features to reduce compile time
[dependencies]
tokio = { workspace = true, default-features = false, features = ["rt"] }

# Use incremental compilation
[profile.dev]
incremental = true
```

### Publishing Workspace Crates

```bash
# 1. Check what will be published
cargo package -p my-core --list

# 2. Dry run
cargo publish -p my-core --dry-run

# 3. Publish in dependency order
cargo publish -p my-types
cargo publish -p my-core
cargo publish -p my-utils
cargo publish -p my-cli

# Or use cargo-release for automation
cargo install cargo-release
cargo release --workspace
```

### Workspace Checklist

When setting up a workspace:

- [ ] Use root-level crate directories (not `crates/` folder)
- [ ] Configure `[workspace.dependencies]` for shared deps
- [ ] Set `[workspace.lints]` for consistent code quality
- [ ] Use `[workspace.package]` for shared metadata
- [ ] Define clear boundaries between crates
- [ ] Avoid circular dependencies
- [ ] Use path dependencies with versions
- [ ] Test entire workspace with `cargo test --workspace`
- [ ] Document inter-crate relationships
- [ ] Consider compilation time when splitting

---

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - For unsafe code
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
