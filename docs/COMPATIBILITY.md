# Compatibility

The simulation is the vendored balatro-rs engine in Fast mode (see [ENGINE.md](ENGINE.md)). It does not reproduce original seeds or saves, and its rules are not verified against Balatro beyond the behaviour listed there and in [the feature roadmap](ROADMAP_FEATURES.md). Equal seeds do not give the original game's draws.

Local Assets also copies `localization/en-us.lua` when present. Ante reads only a closed list of interface keys and the `p_<family>_<size>` booster descriptions from it, by pattern, without executing Lua, and falls back to its own wording. The copy stays in local application data, out of Git and release packages. Verified only against the 1.0.1o file layout.

Arcana and Spectral packs keep the engine order: pick the card first, then its targets in the drawn hand and Use. The original lets the player select hand cards before clicking Use on the pack card. The open pack family is inferred from its contents because the engine keeps no category for it. The pack box rise is a 0.45 s ease standing in for the UIBox alignment motion; it is not frame-matched.
