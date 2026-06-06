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

build: bundle

lint-web:
    cd frontend && vp check

fix-web:
    cd frontend && vp check --fix

icon:
    cd storekeeper-app-tauri && cargo tauri icon icons/app-icon.svg

upgrade-deps:
    cargo update
    cd frontend && vp update

upgrade-vp:
    vp upgrade
    cd frontend && vp update vite-plus @voidzero-dev/vite-plus-core @voidzero-dev/vite-plus-test

