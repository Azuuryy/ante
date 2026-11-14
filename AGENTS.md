# Working on Ante

- Preserve the separation between headless simulation and Bevy presentation; the simulation owns gameplay state.
- Stay faithful to the original game's logic in every aspect. Its logic, formulas, constants and event sequences may be ported and adapted to Rust and Bevy; the internals differ (ECS instead of the LÖVE event queue), the observable result must match.
- Never commit or redistribute the original files themselves: Lua sources, images, sounds, fonts, `.fs` shaders and localization text. Load them at runtime from the user's local installation through the allowlisted importer. Do not execute game scripts or bypass access controls.
- Keep third-party assets optional and out of Git, with a usable fallback. Document unverified compatibility and incomplete behavior.
- Prefer small changes and focused tests. Run `bash scripts/check.sh` (format, Clippy with and without features, tests and build) when practical; report any failures.
- Preserve deterministic simulation behavior and version any changes to RNG or future replay/save formats.
- Update the changelog and relevant documentation when behavior changes. Do not claim untested features work.
- Never commit, push, publish, or deploy without explicit authorization.

- `engine/AGENTS.md`, `crates/ante-core/AGENTS.md` and `crates/ante-app/AGENTS.md` add the rules of each part; read the one for the files you change.
- For Balatro behavior conversion, read [.skills/ante-legacy-to-bevy/SKILL.md](.skills/ante-legacy-to-bevy/SKILL.md). For matching reference screenshots, read [.skills/ante-ui-parity/SKILL.md](.skills/ante-ui-parity/SKILL.md).

## Architecture and builds

- Keep the command, event, projection and trace contract in `ante-core::contract` and the engine adapter in `ante-core::engine`. `ante-core` and the vendored engine in `engine/` must remain buildable without Bevy.
- Change the vendored engine directly in `engine/`, with tests, and keep `engine/NOTICE.md` accurate. Engine types stay behind the adapter. The Bevy client reads a confirmed `RunView`, and the session command system is the only writer to the run driver. Playback consumes the committed trace.
- Keep files around 400 lines at most; split a larger file by responsibility into a module directory. Unit tests stay in `#[cfg(test)] mod tests`, moved to a sibling `<module>/tests.rs` when the module is long; integration tests go in the crate's `tests/`.
- Organize the Bevy client under `app/` for session and presentation, `screens/` for screen composition, `components/` for shared UI, and `services/` for storage, assets and diagnostics. Keep `main.rs` and `lib.rs` at the crate entry points. Resolve content IDs to art in the client.
- Compile profiler, diagnostics, structured logs and technical tooltip fields with `dev-tools`. Prod uses the same rules and recovery paths without the diagnostic systems.
- Import only the allowlisted user-selected resources into local application data. Keep the fallback usable and keep copies out of Git and release packages.
- Update the interface matrix in [the roadmap](docs/ROADMAP.md) after comparing reference and Ante captures at equal client size. Record verified behavior, visual gaps and remaining VFX separately.
