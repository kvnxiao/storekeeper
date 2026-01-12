# Dependency Management

## Specify Exact Versions for Applications

```toml
# In applications (binaries), pin exact versions
[dependencies]
serde = "=1.0.195"
tokio = "=1.35.1"
```

## Use Semantic Versioning for Libraries

```toml
# In libraries, use flexible versions
[dependencies]
serde = "1.0"
tokio = { version = "1.35", features = ["full"] }
```

## Enable Only Needed Features

```toml
[dependencies]
tokio = { version = "1.35", features = ["rt-multi-thread", "net", "macros"] }
serde = { version = "1.0", features = ["derive"] }
# Not: features = ["full"]
```

## Review Dependencies Regularly

```bash
# Check for outdated dependencies
cargo outdated

# Audit for security vulnerabilities
cargo audit

# Check for unused dependencies
cargo machete
```
