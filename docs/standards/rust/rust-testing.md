# Testing Guidelines

## Use `.expect()` in Tests - Never `unwrap()` or `panic!()`

**In test code, only use `.expect()` with descriptive messages.** Never use `.unwrap()` or `panic!()` directly - the descriptive message from `.expect()` makes test failures much easier to debug.

### Why `.expect()` Only

- **Clear failure messages**: When a test fails, you immediately know what went wrong
- **Self-documenting**: The expect message describes what should have happened
- **Consistent debugging**: All failures have context, not just "called `unwrap()` on a `None` value"

### Forbidden in Tests

```rust
// BAD: No context when test fails
let result = process_input("valid").unwrap();

// BAD: Generic panic with no context about what was being tested
if result.is_none() {
    panic!("failed");
}

// BAD: Even with a message, panic! doesn't chain with Result/Option
panic!("expected value to be present");
```

### Required Pattern

```rust
// GOOD: Clear message on failure
let result = process_input("valid").expect("process_input should succeed with valid input");

// GOOD: For Option types
let value = maybe_value.expect("value should be present after initialization");

// GOOD: For error cases
let err = result.expect_err("process_input should fail with empty input");
```

## Test Organization

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

## Property-Based Testing

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

## Integration Tests

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
