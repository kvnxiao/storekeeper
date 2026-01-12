# Unsafe Rust

## Core Principle: Avoid `unsafe`

**Unsafe Rust should be treated as a last resort.** The vast majority of Rust code should never need `unsafe` blocks.

## Why Avoid `unsafe`

- Defeats Rust's safety guarantees (memory, thread, type safety)
- Hard to audit and maintain
- Better alternatives usually exist

## Use Safe Wrapper Crates

Before writing `unsafe`, **search for a safe wrapper crate**.

| Domain | Unsafe Bindings | Safe Wrapper |
|--------|-----------------|--------------|
| Windows API | `windows-sys` | `winsafe` |
| POSIX/Unix | `libc` | `nix`, `rustix` |
| SQLite | `libsqlite3-sys` | `rusqlite` |
| OpenSSL | `openssl-sys` | `openssl` |
| Memory | raw pointers | `bytemuck`, `zerocopy` |

## If You Must Use `unsafe`

1. **Minimize surface area** — Keep unsafe blocks as small as possible
2. **Document all invariants** — Use `/// # Safety` section
3. **Wrap in safe abstractions** — Never expose unsafe in public API
4. **Run Miri** — `cargo +nightly miri test`

```rust
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is non-null and properly aligned
/// - `ptr` points to a valid, initialized instance of `T`
unsafe fn read_from_ptr<T>(ptr: *const T) -> &T {
    &*ptr
}

// Safe public API wrapping unsafe internals
pub fn safe_operation(data: &mut [u8]) -> Result<()> {
    if data.is_empty() {
        return Err(Error::EmptyData);
    }
    // SAFETY: We validated data is non-empty
    unsafe { raw_operation_internal(data.as_mut_ptr()) }
    Ok(())
}
```
