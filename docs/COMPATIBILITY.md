# Compatibility

Ante's basic evaluator classifies one to five unmodified cards with standard poker categories and extended five-of-a-kind, flush house and flush five. Ace-low straights are supported. Modified ranks/suits, wild cards, special rules, scoring, event ordering, original seeds and saves are not implemented or verified against Balatro. Sorting uses explicit rank/suit/id keys; it is deterministic but not verified as compatible with the original game.

Local Assets also copies `localization/en-us.lua` when present. Ante reads only a closed list of interface keys and the `p_<family>_<size>` booster descriptions from it, by pattern, without executing Lua, and falls back to its own wording. The copy stays in local application data, out of Git and release packages. Verified only against the 1.0.1o file layout.

The external-engine evaluation is preliminary; see [ENGINE_EVALUATION.md](ENGINE_EVALUATION.md). No compatibility claims are derived from the compared engines.

Arcana and Spectral packs keep the engine order: pick the card first, then its targets in the drawn hand and Use. The original lets the player select hand cards before clicking Use on the pack card. The open pack family is inferred from its contents because the engine keeps no category for it. The pack box rise is a 0.45 s ease standing in for the UIBox alignment motion; it is not frame-matched.
