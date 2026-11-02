# Working on Ante

- Preserve the separation between headless simulation and Bevy presentation; the simulation owns gameplay state.
- Implement behavior independently; never copy original source, assets, shaders, or localization. Do not execute game scripts or bypass access controls.
- Keep third-party assets optional and out of Git. Document unverified compatibility and incomplete behavior.
- Prefer small changes and focused tests. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo build --workspace` when practical; report any failures.
- Preserve deterministic simulation behavior and version any changes to RNG or future replay/save formats.
- Update the changelog and relevant documentation when behavior changes. Do not claim untested features work.
- Never commit, push, publish, or deploy without explicit authorization.
