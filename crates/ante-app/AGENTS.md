# Working on ante-app

The Bevy client and the `ante` binary. The root `AGENTS.md` still applies.

- `app/` holds the session (the only writer to the run driver), navigation, presentation and score playback; `screens/` composes screens; `components/` holds shared UI (cards, tiles, tooltips, drag, travel, shaders); `services/` holds assets, profile storage and diagnostics.
- The client reads the confirmed `RunView` and sends `Command`s through `Click`; it never decides a gameplay outcome. Sorting, modals, hover, drag and animation are presentation only.
- Geometry follows the original in room tiles (`components/tiles.rs`: `TILE`, `CARD_W`, `spread_x`, …) and card areas follow `CardArea:align_cards`. Cite the original function when porting a layout, timing or constant.
- Cards in a row need distinct `GlobalZIndex` layers (their position): Bevy moves a node whose z-index is written to the top of its ties. Write layers with `set_if_neq`.
- Original assets, shaders, sounds and localization load only from the local import (`services/assets`), with a fallback; nothing from the game ships in the binary.
- Profiler, logs, technical tooltip fields, captures and the debug panel compile only with `dev-tools`.
- Visual checks are done by the user in the running client. Headless tests (`app/layout_tests.rs`) cover layout and draw order; for reference captures read `.skills/ante-ui-parity/SKILL.md`.
