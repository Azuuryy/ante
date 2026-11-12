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

The workspace `ante-app` binary displays generated placeholder shapes and the CLI runs the native one-blind slice described below. The playable client is the experimental `ante-mvp` binary. The core evaluates unmodified hands of one to five cards, including extended categories with duplicate cards. No original assets are required or included.

See [architecture](docs/ARCHITECTURE.md), [roadmap](docs/ROADMAP.md), [compatibility](docs/COMPATIBILITY.md), and [legal](docs/LEGAL.md). Ante code is MIT licensed; dependencies retain their own licenses.

For local checks, run `bash scripts/check.sh`. Once this directory is a Git checkout, it may also be invoked from a locally installed pre-commit hook. A full workspace build requires access to crates.io on the first run.


## Play the experimental client on Windows

Set up the pinned engine first. See [Engine Evaluation](docs/ENGINE_EVALUATION.md).

```powershell
cargo run --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features mvp --bin ante-mvp
cargo run --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features dev-tools --bin ante-mvp
.\scripts\build-windows-client.ps1
```

The script writes `ante.exe` and `ante-dev.exe` to the experiment's `target/windows` directory. Dev includes F2–F5, the profiler, structured logs, the technical tooltip panel, a $1000 starting balance, the Tab debug panel and F6 window captures (see [docs/CAPTURES.md](docs/CAPTURES.md)). Both builds use the same Fast simulation and player profile.

Choose **Options → Graphics → Local Assets** to import supported atlases, the font, the shaders and the sound effects into `%LOCALAPPDATA%\Ante\local-assets`. Restart Ante after importing. `--assets <folder>` and `--steam-textures` load directly for one launch. The binaries contain only fallback art.

The headless native slice is available through `cargo run -p ante-cli -- 42 start`. It prints card IDs for subsequent `select:<ID>`, `play`, `discard` and `cash-out` commands. The replay and RNG versions identify the deterministic Ante format. The graphical client still uses the pinned backend while native coverage expands.
