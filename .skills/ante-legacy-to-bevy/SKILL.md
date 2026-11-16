---
name: ante-legacy-to-bevy
description: Convert a behaviour observed in Balatro into an Ante Bevy screen or interaction, traceable to the engine's states and actions.
---

Read the relevant section of the [legacy guide](../../docs/UI_LEGACY_GUIDE.md), then its counterpart in the [Bevy guide](../../docs/UI_BEVY_GUIDE.md). First find the entry point and the callback in the local installation, read-only.

For the interaction at hand, record its conditions, visible data, input, action, effects and return. Decide whether the state belongs to the engine or to navigation. Route game actions through `Command` and expose their results through `RunView`. Presentation sorting, a modal and the animated score read the projections without changing the simulation. Reuse the existing board layers and controls. Describe the gaps and update the Bevy guide's acceptance checks after verification.

Port the original logic, formulas, constants and sequences faithfully, adapted to Rust and the ECS. Never commit the original files (Lua, images, sounds, fonts, `.fs` shaders, text): load them at runtime from the local installation through the allowlisted importer, with an explicit fallback. Change the vendored engine in `engine/` directly, with tests, and keep `engine/NOTICE.md` up to date.
