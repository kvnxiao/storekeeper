fmt:
    cargo fmt

lint:
    cargo clippy --workspace --all-targets --all-features
    cargo fmt --check

dev:
    cargo tauri dev

test:
    cargo test --workspace

bundle:
    cargo tauri build
