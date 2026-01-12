# Performance Considerations

## Prefer Borrowing Over Cloning

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

## Use `Cow` for Conditional Cloning

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

## Allocations and Capacity

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

## Avoid Unnecessary Copies

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
