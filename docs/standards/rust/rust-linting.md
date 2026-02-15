# Linter and Formatter Configuration

## Clippy

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

Run lint checks and auto-fix from the repository root:

```bash
just lint  # Check: clippy + format check
just fix   # Auto-fix: clippy --fix + cargo fmt
```
