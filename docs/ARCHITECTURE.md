# Architecture

| Crate | Role | Depends on |
| --- | --- | --- |
| `engine/balatro-rs`, `balatro-types`, `balatro-seed` | Vendored rules engine: game state, legal actions, RNG, scoring and its trace. See [ENGINE.md](ENGINE.md). | — |
| `crates/ante-core` | Headless simulation boundary. `contract` defines commands, events, the `RunView` projection and the score trace; `engine` adapts balatro-rs to it (`EngineRun`, `ExternalBackend`). No Bevy dependency. | engine |
| `crates/ante-app` | Bevy client and the `ante` binary: session, navigation, presentation, screens, shared UI and services. | ante-core |

The simulation owns gameplay state. The client sends a `Command` through the session command system, the only writer to the run driver, and reads back a confirmed `RunView` and its ordered trace. Score playback consumes the committed trace and never draws randomness. Presentation must not control simulation outcomes.

In `ante-app`, `app/` holds session and presentation, `screens/` screen composition, `components/` shared UI and `services/` storage, assets and diagnostics. Content IDs resolve to art in the client (`services::assets`). Profiler, diagnostics, structured logs and technical tooltip fields compile only with `dev-tools`.

## Code layout

- Files stay around 400 lines at most. A file that grows beyond that is split by responsibility into a module directory (`run.rs` with `run/board.rs`, `run/hud.rs`, …); a large `impl` can be spread over several files of the same module.
- Unit tests follow the Rust convention: a `#[cfg(test)] mod tests` in the module they test, moved to a sibling `<module>/tests.rs` when the module is long. Integration tests that only need the public API live in each crate's `tests/` directory.
