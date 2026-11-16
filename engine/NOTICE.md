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

Changes made after vendoring:

- The action space follows the same consumable and pack rules as `gen_actions`, and the enhancement and suit Tarots, Strength, Death and the seal Spectrals clear the hand selection after use, like `G.hand:unhighlight_all()` in the original (`consumable_highlight_tests.rs`, `tests/action_space.rs`).
- Per-card Jokers act on each scored or held card, once per repetition, after the card's own effects, like `calculate_joker{individual = true}` in `evaluate_play`: new `OnCardScored` and `OnCardHeld` effects and a `JokerOnCard` score source. Converted: Greedy, Lusty, Wrathful and Gluttonous Joker, Fibonacci, Scary Face, Even Steven, Odd Todd, Scholar, Business Card, Photograph, Walkie Talkie, Smiley Face, Golden Ticket, Rough Gem, Bloodstone, Arrowhead, Onyx Agate (scored), Baron, Shoot the Moon, Reserved Parking and Raised Fist (held). Retriggers now repeat them, and Business Card, Bloodstone and Reserved Parking roll once per repetition, which changes the number of RNG draws in a hand that holds them. Raised Fist picks the last lowest-ranked held card and skips Stone cards, like the original (`per_card_joker_tests.rs`).
- Targets are chosen in the hand before a targeting card is used, like `can_use_consumeable`: the hand stays selectable while an Arcana or Spectral pack has drawn it, a pack Tarot or Spectral is taken only when the selection fits its bounds and is used at once with it, and slot consumables stay usable while a pack is open. The `TarotHand` and `SpectralHand` stages, `ApplyTarot` and `ApplySpectral` are removed; the fixed-size action space loses its two apply slots (109 to 107) and the replay log format moves to version 4. A Planet or Tarot taken from a pack now counts as the last one used for The Fool, like `set_consumeable_usage`.
- A cleared Blind keeps its reward as rows (`reward::RoundReward`: Blind, hands left, Joker dollar bonuses, tags, interest), in the order of `G.FUNCS.evaluate_round`, until cash out. Golden Joker's $4 is one of those rows, paid at cash out like `calculate_dollar_bonus`, instead of being added at round end; the Investment tag is a tag row.
- Bosses refuse hands like `Blind:debuff_hand` (`game/boss_hand.rs`): The Psychic a hand of fewer than 5 cards, The Eye a hand type already played this round, The Mouth any hand type other than the round's first (`Game.mouth_hand`, saved with a serde default). A refused hand scores 0 but still counts as played, triggers the boss's ability and pays Matador $8, like `context.debuffed_hand` (`joker/tests/boss_interactions.rs`).

When vendoring, Ante also removed the Python bindings (`python` feature), colored terminal output (`colored` feature), the upstream binaries (`calc`, `explore`), the benchmark and the Python smoke test. The other upstream crates (`cli`, `tui`, `pylatro`, `balatro-jkr`, `balatro-profile`, `balatro-storage`) are not included.
