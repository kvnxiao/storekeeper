# List all available recipes (default when running `just` with no arguments).
default:
    @just --list

lint: lint-clippy lint-fmt

lint-clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-fmt:
    cargo +nightly fmt --all --check

fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty
    cargo +nightly fmt --all

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

test-web:
    cd frontend && vp test run

icon:
    cd storekeeper-app-tauri && cargo tauri icon icons/app-icon.svg

upgrade-deps:
    cargo update
    cd frontend && vp update

# `vp migrate` re-pins vite-plus, the vite alias, and the bundled vitest to the
# CLI's release. Updating those packages by hand desynchronises the set.
upgrade-vp:
    vp upgrade
    cd frontend && vp migrate

