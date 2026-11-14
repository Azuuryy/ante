# Working on the vendored engine

These crates are a vendored, patched copy of balatro-rs (`balatro-rs` rules engine, `balatro-types` vocabulary, `balatro-seed` seed and RNG port). The root `AGENTS.md` still applies.

- Record every behavior change in `NOTICE.md` (under "Changes made after vendoring") and in the changelog. Keep `LICENSE` and the attribution intact.
- Do not change the order or number of RNG draws in existing paths: snapshots, seeded routes and tests depend on it. A change that must alter draws is versioned and documented.
- Keep `gen_actions` and the fixed-size action space (`gen_action_space`, `space.rs`) on the same rules: both read `consumable_usable`, `pack_card_pickable` and the stage checks in `game/`.
- Stay faithful to the original rules; compare with the local installation read-only and cite the original function (`Card:use_consumeable`, `Blind:modify_hand`, …) in a comment when porting a rule.
- Test each change next to its module (`#[cfg(test)] mod tests`, or a focused `*_tests.rs` file declared in `lib.rs`); seeded integration tests go in `balatro-rs/tests/`. Run with and without `ante-extra-jokers`.
- No Bevy and no Ante types here: `ante-core::engine` adapts these crates to the contract.
