# Ante

Ante is an independent native Rust project inspired by Balatro, intended to explore a deterministic, extensible gameplay engine. It is not affiliated with or endorsed by Balatro's creators or publishers. This repository is an early foundation, **not a playable game or a compatible replacement yet**.

## Getting started

Install stable Rust and the platform prerequisites for [Bevy](https://bevy.org/learn/quick-start/getting-started/setup/). Then:

```sh
cargo run -p ante-app
cargo run -p ante-cli
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

The graphical app displays simple generated placeholder shapes and the CLI prints a deterministic sample; neither implements a game loop. The core evaluates unmodified hands of one to five cards, including extended categories with duplicate cards. No original assets are required or included.

See [architecture](docs/ARCHITECTURE.md), [roadmap](docs/ROADMAP.md), [compatibility](docs/COMPATIBILITY.md), and [legal](docs/LEGAL.md). Ante code is MIT licensed; dependencies retain their own licenses.

For local checks, run `bash scripts/check.sh`. Once this directory is a Git checkout, it may also be invoked from a locally installed pre-commit hook. A full workspace build requires access to crates.io on the first run.
