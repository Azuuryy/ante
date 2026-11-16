# Changelog

## Unreleased

### Added

- Round evaluation after a beaten blind, after `add_round_eval_row`: the Blind row (chip, target, stake), then the remaining hands, Joker bonuses, tags and interest under a dotted line, each row and dollar sign appearing on the original event timing with the `cancel`, `highlight1` and `coin3` sounds, then the "Cash Out: $N" button. Rows come from `RunView.reward_rows`, now with an `Interest` row.
- The deck view follows `G.UIDEF.view_deck`: the deck name and description, base and effective tallies (aces, faces, numbers, suits) with the ui_assets icons, the rank column, and one `title` card row per suit at 0.7 card size. The Remaining tab greys the cards already drawn with the original `played` shader instead of hiding them.
- The Collection follows `create_UIBox_your_collection*`: the hub with its Consumables group, one black box of card areas per page (Jokers 3 × 5, Tarots 5 + 6, Planets 6 + 6, Spectrals 4 + 5, Boosters 2 × 4 at 1.27 size, Vouchers in overlapping pairs, Enhanced 2 × 4, Seals, Editions on the Joker), a red "Page n/N" cycle, the Tags grid, the Blinds grid beside the ante table and the deck carousel with its pips. Undiscovered cards show the original undiscovered centre with the "?" over it; collection cards hover like hand cards.
- Shared pieces: `tiles::generic_options` (JOKER_GREY frame, L_BLACK body, full-width Back), `tiles::option_cycle` and `components/card_row.rs` (a card area drawn as a row with the slot fan).
- Hovered cards lean toward the pointer like the original: a vertex stage written for Ante (`components/tilt.wgsl`) ports the `position()` stage of the card shaders, and card sprites inside a card switch to it automatically, with their edition overlays.
- Cards lean with their sideways speed while they travel (`Moveable:move_r`: 0.015 × the x velocity, eased by `exp(-190·dt)`), then straighten.
- "Tarot" and "Spectral" in descriptions catch a light sweeping across them every 2.4 seconds. This is an Ante addition; the original draws them in their plain set colour.
- `components/card_area.rs` holds the positions of `CardArea:align_cards` for the hand, play, title, Joker, consumable and voucher areas; the hand fan, the shop and inventory spread, the play area and pack rows use it.
- Run logs: every run writes its configuration and each command, with the confirmed state after it, to `%LOCALAPPDATA%\Ante\runs` (the 20 most recent are kept). `cargo run -p ante-core --example replay -- <log>` reruns a log and reports the first step that diverges (`ante_core::run_log`).
- Playing cards draw their enhancement centre (Gold, Steel, Glass, Bonus, Mult, Wild, Lucky, Stone) instead of the plain base; Stone cards hide their face, like `Card:draw`.
- The Gold seal shines with the original voucher shader and Spectral cards with the booster shader. Edition shines follow the original `tilt_var.amt`: the cursor offset on a hovered card, an ambient tilt otherwise.
- Pack and shop playing cards keep their enhancement, edition and seal (`ShopArt::Card` now carries a `CardLook`), and the run view names the open pack family (`RunView.open_pack`, inferred from the pack contents).
- Arcana and Spectral packs open like the original: the pack box rises from below, the drawn hand stays visible and selectable under it, and the `booster_pack_sparkles` particles fade in over the room.
- Booster popups give the number of cards to choose and shown, read from the local `p_<family>_<size>` localization (Ante fallback otherwise).
- Headless layout test: the HUD column stays inside a 1920 x 1056 window.
- Tests over every joker the engine offers: an Ante description, a Currently line for each joker whose original text has one, and a scored hand without errors.

- The shop follows the original layout in tiles: framed panel under the Joker row, two-line Next Round, Reroll with a large price, Joker, voucher (vertical ante label) and booster boxes, price tags behind the top of each offer, an orange Buy, green Redeem and Open, and a red Buy & Use for consumables that can be used at once (`Command::BuyAndUseConsumable`, `ConsumableView.can_buy_and_use`).
- Green Sell buttons with the original "SELL $n" layout beside the selected Joker or consumable, with a red Use under it for consumables; the selected card and its buttons draw in front of the other cards until it is deselected.
- Engine patch `balatro-rs-consumable-room.patch`: cards that create cards need room when picked from a pack, and Buy & Use applies a consumable without a slot.

- Original sound effects from the user's installation (imported with Local Assets, never shipped) on hover, selection, card moves, score popups, the total, purchases and sales, buttons and dissolves, with master and game sound volumes in Options > AUDIO.
- Jokers and consumables that leave the inventory burn away with the original dissolve shader, burn colours (gold when sold), particles and sounds.
- The original debuff, booster and voucher shaders on debuffed cards, shop packs and vouchers.
- The original room sway and shake: score popups jiggle the board, which also drifts slowly and leans toward the cursor.
- Collection categories for Tarot, Planet, Spectral, enhanced cards, seals, editions, boosters and blinds, with discovered / total counts; consumables, packs and boss blinds met in a run are discovered, and Unlock All covers every category.
- Tooltip side boxes for Diet Cola, Perkeo and the tags that give booster packs.
- Dev build: every run starts with $1000; Tab opens a debug panel (money, hands, discards, round, ante, score, win blind, lose run) through `Command::Debug`, and F6 saves a window capture to `%LOCALAPPDATA%\Ante\captures`. Debug-modified runs are flagged in the view.
- Joker tooltips show the original grey "(Currently …)" line with a colored value for 17 scaling jokers, from a structured `CurrentValue` computed by the simulation.
- Dev tooltips show their technical fields in a separate light-grey panel under the tooltip, now covering hand cards, shop offers, pack choices and the deck view.
- Score playback follows the original timing in game seconds (card moves, highlights, chips/mult/joker popups, total and discard), with speeds from x0.5 to x128 and the original automatic acceleration. Colored "+N" popups and a juice pulse mark each scoring step, above played cards and below held Jokers. The chosen speed is saved in the profile (version 3).
- Played cards sit 3.6 room units above the hand and scoring cards rise by 0.2 card heights while scored; selected hand cards use the same lift.
- Playing cards travel like the original `Moveable:move_xy`: drawn from the deck one by one, from the hand to the play area, and from the play area or the Discard button to the off-screen discard, 0.1 game seconds apart; cards drawn after a discard wait for the discarded ones to leave. The hand only spreads over the cards that have arrived, closing up during a discard and opening as each card is drawn; its fan follows the cards present. Drawn cards leave the deck face down and flip face up as they enter the hand. The hand fans out with the original tilt, drop and sway.
- Card hover eases scale and lift like the original (`exp(-60·dt)`, `exp(-50·dt)`) with a short juice, and no longer jumps.
- Joker and playing-card editions (foil, holographic, polychrome, negative) render with the original shaders from the user's installation, converted to WGSL at load by naga. The importer copies the allowlisted `.fs` files; a missing or failing shader keeps the plain sprite.
- `docs/CAPTURES.md` defines the git-ignored reference capture library, naming and checklist.
- Score popups follow the original status text order and colors: "Again!", chips, +Mult, XMult ("X1.5 Mult") and money ("$3"), each with its own delay. The `balatro-rs-trace-effects.patch` engine patch records the XMult factor and the money of every score step without changing scores.
- Tooltips show the original side boxes left of the popup: what the effect mentions (Stone Card for Marble Joker, Steel Card for The Chariot, editions for Wheel of Fortune and edition tags, seals for Spectral cards), then the edition, seal and stickers. The rule texts are Ante's own wording.
- Engine patches `balatro-rs-debug.patch` (debug score and ante entry points) and `balatro-rs-joker-current.patch` (read access to scaling counters).
- Pointer-anchored drag and drop for hand cards, Jokers and consumables, contextual inventory actions, and color-coded card tooltips.

- Manual left/right reordering for hand cards, Jokers and consumables, with Joker effect registration following inventory order.
- Shared card hover motion, red debuff cross/tint, and seal badges on hand and deck cards.

- Isolated experimental adapter and preliminary comparison of two external simulation engines.
- Bevy prototype diagnostics, score playback, local card and Joker textures, and a persistent Joker collection.
- Frame-time graph, optional 1,000 FPS benchmark mode, and layered local card faces.
- Experimental Bevy MVP for a complete one-ante run with `balatro-rs` and reproducible win and loss scenarios.
- Main-menu navigation with deck selection (Red and Blue playable), options and collection screens.
- Balatro-style main menu recreated in tile units: logo over an ace of spades, Play / Options / Quit / Collection buttons with emboss rims, drop shadows and hover lift, plus an optional local swirl background.

- Tile-based deck selection, collection hub with Joker pages and deck previews, and tabbed settings.
- Guides to the original interface architecture and the experimental Bevy UI.

- Experimental game board and shop with joker and consumable slots, engine-validated actions, local consumable textures and card tooltips.

- Added the New Run and pause overlays, optional manual and clipboard seeds, and a three-blind board with skip actions and local chip art.

- Added the eight-ante campaign, the victory overlay and bounded Endless continuation.
- Added a shared run board, translucent inventories, held tags and the Full Deck / Remaining overlay.
- Expanded the interface guides and added repository skills for legacy conversion and screenshot parity.

- Added selected-hand base Chips and Mult, level, and tabbed Run Info.
- Added all Joker, Voucher and Tag entries to Collection, voucher redemption, playing-card offers and booster choices.
- Added engine-side tag effects, including queued free boosters and contextual shop offers.

- Added a headless run contract, a reusable Bevy client and a deterministic native one-blind CLI slice.
- Rebuilt the shop around adaptive offer zones, larger card art, structured hover details and a remaining-deck grid.
- Enlarged booster pack choices to the shop card scale. Joker tooltips now follow Balatro's palette and layout: grey "Currently" lines for tracked scaling values, green chances, yellow money, orange card values and counts, exact rarity colors, a larger centred rarity badge, and no release-visible raw edition or state fields.
- Added a version 2 profile with `all_unlocked`, a migration from version 1, and a dev-only "Unlock all" action in Settings > GAME.
- Collection entries with an unlock condition now show a padlock overlay, a locked tooltip and the unlock text; decks have the same treatment.
- Extended dev-only technical tooltip fields to consumables, vouchers, boosters, shop and pack cards, blinds, tags, deck cards and collection entries.
- Added a short Chips/Mult pop during score playback and aligned card hover growth with the reference value (5%).
- Documented the consommable eligibility matrix, the profile/unlock model, shader conversion blockers, and animation/audio gaps in `docs/ROADMAP_FEATURES.md`.
- Added transactional local asset import and separate production and developer Windows builds.

### Changed

- Collection popups are the ones cards have in a run, as `Card:generate_UIBox_ability_table` builds them: Jokers carry their rarity badge, Tarot, Planet and Spectral cards their set, vouchers and boosters their type; an enhancement shows its rule with its name as a badge, a seal its name and badge, an edition the "Edition" badge and its own; tags have no badge. Locked entries read "Locked" with their unlock condition. The "Discovered" badge is gone. The catalog now gives each Joker's rarity (`Content.rarity`).
- The Blind choice panels have no popup any more, as in the original; this lets the tag next to Skip Blind show its popup, which the panel's popup used to cover. Tag popups have no "Tag" badge.
- Selecting hand cards keeps the Play Hand, Sort and Discard row in place and only repaints the buttons whose state changes, instead of rebuilding the row each time.
- Shop price tags overlap the top of their card as the original `create_shop_card_ui` aligns them; booster tags sit a little lower.
- Hovering the deck shows "View Deck" in white on a half-transparent black box centred on the top card, which follows the pile as it shrinks.
- The engine catalog lists Planets (Mercury to Eris), Blinds and Tags in their original `order`, as the collection shows them.
- Edition badges (Foil, Holographic, Polychrome, Negative) cycle through the original `G.C.DARK_EDITION` colours instead of staying black.
- "Tarot", "Planet" and "Spectral" are capitalised in descriptions and drawn in their `G.C.SECONDARY_SET` colours.
- The round reward is kept as the original evaluation rows (Blind, hands left, Joker bonuses, tags, interest) and exposed to the client as `RunView.reward_rows`. Golden Joker's $4 is now paid at cash out on its own row, like the original, rather than at the end of the round.
- The balatro-rs engine is vendored in `engine/` with Ante's twelve patches applied, its Python bindings and colored output removed; `engine/NOTICE.md` records its origin and unverified provenance. It no longer needs a local checkout or a manual patch sequence.
- `ante-core` holds the contract and the engine adapter (`ante-core::engine`, `EngineRun`, `ExternalBackend`); `ante-app` builds the `ante` binary on it. `extra-jokers` is an `ante-core` feature relayed by `ante-app`. Release builds keep integer overflow checks.
- Large client, contract and engine files are split into modules by responsibility, and long test modules move to sibling `tests.rs` files. Behavior is unchanged.
- `scripts/build-windows-client.ps1` builds from the workspace into `target/windows`; the check script and CI cover the engine and the optional features.
- `docs/ENGINE_EVALUATION.md` becomes `docs/ENGINE.md`.
- Chances read as a fraction in every popup ("1/4 chance" for "1 in 4"), written as one word so a line never breaks inside it, and coloured like the original chance text. This departs from the original wording at the user's request.
- Neighbouring cards slide aside while a hand card, Joker or consumable is dragged over them, like `align_cards` sorting the row by position every frame; the move is still committed on release.
- Tarot and Spectral cards that take hand cards say "Select at most N cards"; those that take none say nothing about selection.
- Banner and Mystic Summit show a "(Currently ...)" line from the discards left (an Ante addition; the original has none for them).
- Playing card popups write their name at the description scale on one line, the rank in grey and the suit in its colour, like the original `playing_card` text.
- Card popups follow `card_h_popup` in tiles: pixel boxes with emboss, centred name (white box for playing cards), descriptions at the original 0.32 scale and `create_badge` badges. Playing cards list only their chips and enhancement rule; edition and seal show as badges and side boxes, and the full attributes stay in the Dev panel. The "Playing card" badge is gone, as in the original.
- The HUD body is 30 tiles tall and centred on the room like `create_UIBox_HUD`, so only its two side bands show and the counters stay on screen. Ante and Round no longer have the extra 2 px.
- Menu buttons, panels and modals no longer shrink or wrap when text grows, which fixes the overlapping Back button and the stacked Collection columns.
- Text sizes snap to whole physical pixels per glyph pixel; tooltips sit on whole pixels with wider side padding so their lines no longer jump.
- Ante and Round end about 2 px lower, level with Options; the menu footer sits lower and the F5 shortcut is removed (VSync stays in Settings).

- During a blind the left panel shows the original blind box (name, rule, blind chip, target with the stake chip, reward); Play Hand, Discard and the outlined Sort Hand box follow `create_UIBox_buttons`; inactive buttons use the original inactive colours.
- Text in shadowed or embossed buttons is centred on the visible box; "1 / 8" is centred in its counter.
- The deck shows "View Deck" on hover, its count on the right, a tighter stack with greyer cards under the top one; the shop rows are further apart and price tags sit higher.
- Gamma, deck stack, shop gap and price tag height are named constants for tuning (see `docs/UI_BEVY_GUIDE.md`).

- The left panel follows `create_UIBox_HUD`: each counter has its inner grey box, the stake chip sits beside the round score, Chips and Mult are side by side, Run Info is on two lines, and the panel and shop share the same thin `DYN_UI.MAIN` outline.
- Interface boxes are drawn with the original stepped pixel corners, emboss lip and button shadow, and use the original palette and dynamic colours, which follow the phase and the blind; text is drawn at the original size (one room unit per scale) with its shadow.
- The shop starts 5.3 units above the hand and runs below the window like the original; the empty voucher area shows "DEFEAT / BOSS BLIND / TO RESTOCK".
- The SHOP sign is animated (4 frames at 10 fps) in its framed box, with the floating gold line under it read from the local localization file.
- The deck sits at its original place, its backs stacked with the original parallax offsets.
- The run background uses the original `background` shader from the local installation, with the blind colours; without it, a flat fill of the same colour.

- The window fits the original 22 x 12.9 tile canvas (room plus padding), so the whole interface is about 12% smaller than before at the same window size. Cards, Jokers, consumables, offers and the deck use the original card size (0.95 x `CARD_W` x `CARD_H`), and Jokers, consumables and offers spread like the original card areas.
- The left panel uses the original tile sizes for the score, Chips and Mult, Run Info and Options, hands, discards, money, ante and round; it is outlined in red in the shop. The shop sign sits in a dark box with a gold line under it.
- The deck stands at the original position and shows its thickness as stacked backs that thin out as it empties.
- Tooltips follow the original anchor rule: above the card, below near the top of the room, left of shop offers and deck views.
- The Instant speed is replaced by the original speed range; Skip still ends playback.
- Project policy: the original logic, formulas and constants are ported faithfully; the original files are never committed or shipped and are loaded from the user's installation.
- The jokers engine patch counts the three optional jokers in `test_exactly_74_jokers_implemented`.

### Removed

- The native one-blind engine, `ante-cli`, the `experiments/engine-evaluation` crate, the Balatrust comparisons and the `score_stress` example.

### Fixed

- Clicking the selected shop or pack card deselects it again (`Card:click`), and the selected shop card keeps its hover lean. A click goes to the innermost target only, so a card's Buy or Use button no longer also toggles the card.
- The shade behind menus (New Run, Options, Collection, pause) covers the whole window instead of only the room the menus are centred in.
- Hovered cards no longer vanish: the hover tilt lowered the clip-space w of cards that sit near the far plane of the UI projection, which clipped them away; the depth now scales with w.
- The deck view, the deck carousel and their popups show each deck's starting rule (the Red Deck gives "+1 discard every round"), read from the engine (`ante_core::engine::deck_description`), instead of placeholder text.
- Arcana and Spectral packs work like the original: select target cards in the hand first, then click a pack card to highlight it and use it with its USE button (SELECT for Jokers and playing cards), which is red only when the selection fits. Consumables already in your slots can be used while a pack is open. A Planet or Tarot taken from a pack now counts as the last one used for The Fool.
- Jokers that act on each card (Scary Face, Smiley Face, Fibonacci, the suit Jokers, Baron and 17 others) now trigger on every repetition of every scored or held card, as in the original: with Scary Face, Smiley Face, Sock and Buskin and Pareidolia, a five-card hand scores both Jokers ten times. Their popup appears on the card as it scores, and the Joker juices.
- Pack choices stay on one row: the area is `n × CARD_W` wide for Arcana and Spectral packs, `× 1.1` for Standard and Buffoon, `× 1.1 + 0.5` for Celestial, and the cards close up instead of wrapping (`create_UIBox_*_pack`). After a pick in a Mega pack the area follows the remaining cards rather than the original's initial width.
- Played cards keep the play area's step of 1.075 cards (`play_W = 5.3 × CARD_W`, 5 slots) instead of the hand's, so they no longer touch.
- A hand card released after a drag that did not move it no longer comes to the front: hand cards have one layer per position, and the drag only writes a layer that changes.
- The deck view no longer draws a cream box behind cards that have their atlas art.
- After the enhancement and suit Tarots, Strength, Death and the seal Spectrals, the hand selection is cleared like the original `G.hand:unhighlight_all()`; a second Tarot picked from the same pack no longer starts with the previous targets selected. The engine's action space now follows the same consumable and pack rules as its action list, which fixes the occasional failure or hang of the upstream random-game test.
- Small text was snapped to the font's glyph grid, which shrank tooltip descriptions to about 16 px at 1080p; sizes now only round to whole physical pixels.
- The blind target number kept Bevy's default font after each update; text sizes now change through `TextSize` and the game font stays.
- The Emperor and The High Priestess can no longer be picked from a pack with full consumable slots, where they created nothing; used from the inventory with full slots, The Emperor creates one Tarot as before.
- Edition overlays on held and shop Jokers were drawn under the sprite and could not be seen.
- Converted shaders now work in the gamma space of the original, so editions are no longer washed out.
- The Options tabs fit on one row; AUDIO no longer covers the speed setting.
- Unlock All stays available after a first unlock.
- Mult is fractional like the original (`balatro-rs-fractional-mult.patch`): XMult effects no longer truncate (X1.5 on 7 Mult gives 10.5), the hand score is floor(chips x Mult), the Mult tile follows the original number format, and The Flint halves base values to the nearest whole number with Mult at least 1.
- Green Joker, Castle, Obelisk, Yorick and Joker Stencil follow the original rules through the `balatro-rs-joker-rules.patch` engine patch: Green Joker and Castle keep their value for the whole run, Obelisk grows per card and resets only when the hand is strictly the most played, Yorick counts discards since it was acquired, and Joker Stencil counts itself as an empty slot.
- Selecting a card no longer shakes the whole hand: rebuilt cards keep their position, velocity, fan tilt and drop, and the fan eases like the original.
- Consumable "Use" availability is now covered by a regression test confirming that non-target cards remain usable outside a blind while hand-touching cards wait for a valid target selection.
- Made hand selection a hold-and-sweep gesture that toggles each newly hovered card immediately; starting the hold on a card remains reserved for dragging it.
- Kept the run counters beside the action buttons across window scales.

- UI text is rasterized at the fitted window scale and uses nearest-neighbor glyph sampling.

- Prevented an overflow when rendering the first Joker or consumable. Simulation failures during run startup and card selection now reach the in-app error screen.
- Moved hand reordering controls below the card face and placed seals near the upper-left rank using the optional local atlas.

- The interface now scales continuously with the window size (centred, letterboxed) instead of staying at fixed pixel sizes; the separate frame profiler window keeps its own unscaled layout.

## [0.1.0] - 2026-11-02

### Added

- Initial Rust workspace with a headless card model, basic hand evaluation, deterministic RNG, CLI and Bevy placeholder scene.
