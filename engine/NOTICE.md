# Vendored engine

The crates in this directory come from [balatro-rs](https://github.com/evanofslack/balatro-rs) at revision `3d66de905d5824bcb92c453214dc2badfbb9fc3f`:

| Directory | Upstream crate | Upstream path |
| --- | --- | --- |
| `balatro-rs/` | `balatro-rs` | `core/` |
| `balatro-types/` | `balatro-types` | `balatro-types/` |
| `balatro-seed/` | `balatro-seed` | `balatro-seed/` |

## License and provenance

The upstream manifests declare the MIT license. The upstream repository had no root license file at that revision, so `LICENSE` here reproduces the MIT text the manifests declare. `balatro-seed` describes itself as a port of the original seed algorithm and credits TheSoul for the research. Provenance and rights of this code have not been verified. Resolve both points before any public release (see `docs/LEGAL.md`).

## Changes made by Ante

These patches were applied in this order before the code was vendored. Their history is kept in Git; future engine changes are made directly in these crates.

1. `bevy-uuid`: align the optional UUID dependency.
2. `play-trace`: expose the score trace from the actual play action.
3. `jokers`: add the optional Joker rules behind `ante-extra-jokers`.
4. `endless`: extend the campaign with bounded Endless.
5. `shop-tags`: add the tag and shop transitions.
6. `consumable-use`: restrict consumable use to the original eligibility rules (`consumable_use_tests.rs`).
7. `debug`: add the `debug_add_score` and `debug_next_ante` entry points (`debug_tests.rs`).
8. `joker-current`: make Joker counters and deck helpers readable for tooltips.
9. `trace-effects`: record the XMult factor and money on each score step (`trace_effect_tests.rs`).
10. `joker-rules`: align Green Joker, Castle, Obelisk, Yorick and Joker Stencil with the original.
11. `fractional-mult`: keep Mult fractional and halve The Flint's base values like the original.
12. `consumable-room`: require room for created cards from packs; add Buy & Use (`consumable_room_tests.rs`).

When vendoring, Ante also removed the Python bindings (`python` feature), colored terminal output (`colored` feature), the upstream binaries (`calc`, `explore`), the benchmark and the Python smoke test. The other upstream crates (`cli`, `tui`, `pylatro`, `balatro-jkr`, `balatro-profile`, `balatro-storage`) are not included.
