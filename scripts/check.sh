#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --features ante-app/extra-jokers,ante-app/dev-tools -- -D warnings
cargo test --workspace
cargo test -p ante-core -p ante-app -p balatro-rs --features ante-app/extra-jokers
cargo build --workspace
