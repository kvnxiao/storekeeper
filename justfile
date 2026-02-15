lint:
    cargo clippy --workspace --all-targets --all-features
    cargo fmt --check

fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty
    cargo fmt --all

dev:
    cargo tauri dev

test:
    cargo test --workspace

bundle:
    cargo tauri build

lint-web:
    cd frontend && pnpm lint && pnpm tsc --noEmit

fix-web:
    cd frontend && pnpm fix

icon:
    cd storekeeper-app-tauri && cargo tauri icon icons/app-icon.svg

