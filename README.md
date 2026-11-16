# Ante

Ante is an independent native Rust project inspired by Balatro, built on a deterministic headless simulation and a Bevy client. It is not affiliated with or endorsed by Balatro's creators or publishers. It is in development and **not a compatible replacement**.

## Getting started

Install stable Rust and the platform prerequisites for [Bevy](https://bevy.org/learn/quick-start/getting-started/setup/). Then, from the repository root:

```sh
cargo run -p ante-app --bin ante
cargo run -p ante-app --bin ante --features dev-tools
```

The workspace has three parts: `ante-core` (headless contract and engine adapter), `ante-app` (Bevy client and the `ante` binary) and `engine/` (the vendored balatro-rs rules engine). No original assets are required or included.

Run the local checks with `bash scripts/check.sh` (format, Clippy with and without features, tests and build). A first build needs access to crates.io.

See [architecture](docs/ARCHITECTURE.md), [engine](docs/ENGINE.md), [roadmap](docs/ROADMAP.md), [content coverage](docs/CONTENT_COVERAGE.md), [compatibility](docs/COMPATIBILITY.md) and [legal](docs/LEGAL.md). Ante code is MIT licensed; the vendored engine and dependencies keep their own licenses.

## Windows builds

```powershell
.\scripts\build-windows-client.ps1
```

The script writes `ante.exe` and `ante-dev.exe` to `target/windows`. Dev includes F2–F5, the profiler, structured logs, the technical tooltip panel, a $1000 starting balance, the Tab debug panel and F6 window captures (see [docs/CAPTURES.md](docs/CAPTURES.md)). Both builds use the same simulation and player profile.

Choose **Options → Graphics → Local Assets** to import supported atlases, the font, the shaders and the sound effects into `%LOCALAPPDATA%\Ante\local-assets`. Restart Ante after importing. `--assets <folder>` and `--steam-textures` load directly for one launch. The binaries contain only fallback art.
