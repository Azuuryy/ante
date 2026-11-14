# Working on ante-core

The headless simulation boundary. The root `AGENTS.md` still applies.

- `contract/` defines what the client may know and do: `Command`, `RunView` and its views, `ScoreTrace`, `RunEvent`, `GameBackend` and `RunDriver`. Keep it small, plain data, and independent of the engine's types.
- `engine/` adapts the vendored balatro-rs to the contract: `EngineRun` (run, resolve, view), conversions, catalogs and effect descriptions, `ExternalBackend`. Engine types never leave this module.
- No Bevy, rendering, timing or file system code. `cargo tree -p ante-core` must not list Bevy (CI checks it).
- A `RunView` is a projection: building it never draws randomness or mutates the run. Commands that index into a list carry the view `revision` and are refused when stale.
- Score traces come from the actual play action, never from a replay, so playback shows the committed result.
- Adapter tests live in `engine/tests/` (scoring, inventory, campaign); run them with and without `extra-jokers`.
