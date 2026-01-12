# API Design

## `#[must_use]` Attribute

Use `#[must_use]` when ignoring a value would likely be a mistake. **Always include a custom message.**

### When to Use

```rust
// Results and error types
#[must_use = "errors must be handled, not silently ignored"]
pub fn validate_config(config: &Config) -> Result<(), ValidationError> { }

// Builder patterns
#[must_use = "builders must be used to construct the final value"]
pub struct QueryBuilder { /* ... */ }

impl QueryBuilder {
    #[must_use = "this returns a new builder with the filter added"]
    pub fn filter(mut self, f: Filter) -> Self { /* ... */ }
}

// Expensive computations
#[must_use = "computing the hash is expensive; use the result"]
pub fn compute_hash(data: &[u8]) -> Hash { }

// State changes
#[must_use = "the guard must be held to maintain the lock"]
pub fn acquire_lock(&self) -> LockGuard<'_> { }
```

### When NOT to Use

```rust
// Side-effect functions where return is optional info
pub fn log_event(event: &Event) -> usize { }

// Simple getters
pub fn len(&self) -> usize { }
```

## Function Parameter Types

### Decision Hierarchy

| Scenario | Recommended Type | Example |
|----------|------------------|---------|
| Read-only access | `&str`, `&Path`, `&[T]` | `fn print(msg: &str)` |
| Read-only, flexible input | `impl AsRef<T>` | `fn read(path: impl AsRef<Path>)` |
| Need ownership, want flexibility | `impl Into<T>` | `fn new(name: impl Into<String>)` |
| Need exact type | Concrete type | `fn process(data: Vec<u8>)` |

### Examples

```rust
// Borrowed when you only need to read
pub fn validate_name(name: &str) -> bool {
    !name.is_empty() && name.len() < 100
}

// AsRef for maximum flexibility without ownership
pub fn read_config(path: impl AsRef<Path>) -> Result<Config> {
    let content = std::fs::read_to_string(path.as_ref())?;
    // ...
}

// Into when you need ownership
impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self { name: name.into(), email: email.into() }
    }
}

// Clean call site - no .to_string() needed
let user = User::new("Alice", "alice@example.com");
```
