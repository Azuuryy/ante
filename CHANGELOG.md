# Changelog

## Unreleased

### Added

- Unlocks like the original: a profile starts with the original's 91 items locked, and each one unlocks when its condition is met (career statistics, specific hands and discards, rounds and runs won, antes reached, the deck's cards, discoveries, wins per deck and stake). Locked Jokers and vouchers stay out of the shop and packs, a locked deck cannot start, and a stake opens once the deck has won at the stake below. "Unlocked!" shows the item's name. Seeded runs unlock nothing, like the original.
- Credit Card (money can go down to -$20, shown as "-$n"), Gift Card ("Value Up!" on every Joker and consumable each round) and Perkeo (a Negative copy of a random consumable when leaving the shop, with its own slot). Every Joker of the original is now in the game.
- All eight stakes can start a run, cumulative like the original: Red (no Small Blind reward), Green and Purple (higher blind amounts), Black (Eternal Jokers that cannot be sold), Blue (one discard less), Orange (Perishable Jokers, "Disabled!" after 5 rounds) and Gold (Rental Jokers, $1 to buy and $3 a round). The run log records the stake.
- The 13 other decks can start a run: Yellow, Green, Black, Magic, Nebula, Ghost, Abandoned, Checkered, Zodiac, Painted, Anaglyph, Plasma and Erratic, with their original setup (`Back:apply_to_run`) and rules. Green's discards show their own row at cash out, and Plasma shows "Balanced" over the play area.
- Ten more Jokers: Sixth Sense, DNA ("Copied!"), Certificate, Showman (owned cards may show up again), Astronomer (Planet cards and Celestial packs are free), Invisible Joker (sold after 2 rounds, duplicates a random Joker), Blueprint and Brainstorm (copy the Joker to the right or the leftmost one, without making it grow twice), and the Legendaries Triboulet and Canio. 144 of the 150 Jokers are now in the game.
- Twelve Jokers that create cards or level up hands: 8 Ball, Superposition, Seance, Vagabond, Hallucination and Cartomancer ("+1 Tarot" or "+1 Spectral"), Riff-raff ("+1 Joker"), Marble Joker ("+1 Stone"), Space Joker ("Level Up!"), Burnt Joker (levels up the first discarded hand each round), Loyalty Card (X4 every sixth hand) and Diet Cola (a Double Tag when sold).
- Four Fingers, Shortcut and Smeared Joker: hands are read like the original `get_flush` and `get_straight` (flushes and straights of 4 cards, straights skipping one rank, Hearts with Diamonds and Spades with Clubs), and the "contains a Straight/Flush" Jokers follow them.
- Nine Jokers that change the round's rules: Oops! All 6s (doubles every listed probability), Chaos the Clown (a free reroll in each shop), Burglar ("+3 Hands", no discards), Juggler, Drunkard, Troubadour, Merry Andy, Splash (every played card scores) and Mr. Bones ("Saved!" when the last hand reaches a quarter of the blind; the blind then pays nothing).
- Seven economy Jokers: Delayed Gratification, Egg ("Value Up!" at the end of each round), To Do List (its hand changes each round and shows in the tooltip), To the Moon (interest becomes `min($/5, cap/5) × (1 + Moons)` like the original), and Cloud 9, Rocket and Satellite, which pay on their own row at cash out.
- Seven more Jokers: Hiker (scored cards keep +5 Chips for good), Vampire (drains the enhancements of scoring cards before they score), Seltzer (retriggers played cards for 10 hands, then "Drank!"), Hologram (grows with each playing card added), Campfire (grows with each card sold, resets after a Boss Blind), Ceremonial Dagger (slices the Joker to its right when a blind is selected) and Madness (grows and destroys a random Joker on Small and Big Blinds).
- Twelve more Jokers that grow or wear out: Square Joker, Runner, Throwback, Fortune Teller, Flash Card, Red Card, Constellation, Lucky Cat, Glass Joker, and the food Jokers Popcorn, Turtle Bean and Ramen, which are "Eaten!" once used up. Each says its growth like the original ("+2 Mult", "X1.1 Mult", "Upgrade!", "-X0.01").
- Ten more Jokers in the shop and packs: Swashbuckler, Blackboard, Flower Pot, Seeing Double, Stuntman (its hand is 2 cards smaller), Bootstraps, Gros Michel and Cavendish (Gros Michel may go "Extinct!" at the end of a round, 1 in 6, and only then can Cavendish appear), Ancient Joker and The Idol (their suit or card changes each round and shows in the tooltip).
- Castle and Mail-In Rebate name the round's suit and rank in their tooltip ("per discarded Heart card", "each discarded 10"), like the original `castle_card` and `mail_card`, instead of "the current suit".
- Words about gold (Gold card, Gold Seal, Golden Ticket) shine in tooltips: their letters ripple between gold tones under a sweeping light. An Ante addition, drawn by animating each letter's colour.
- Jokers speak outside the score animation too, once it is over, one message at a time like `card_eval_status_text`: Castle "Upgrade!" per card of its suit, Hit the Road "X1.5 Mult", "X2 Mult"… per Jack and "Reset" at the end of the round, Yorick "X2 Mult" every 23 cards, Green Joker "-1 Mult", Mail-In Rebate "$5", Trading Card "$3" and Faceless Joker "$5" on a discard. A held Gold card shows "$3" and a Purple Seal "+1 Tarot" on the card. Ride the Bus and Obelisk say "Reset" during the hand when they go back to their start.
- Run logs use format 2: selections grouped on one line as short codes, only the state fields that changed, short card names, and an `explain` block (blind, hand type and Chips × Mult, the score trace summarised by source, engine notes such as a Gold card paying or a seal creating a card, Jokers whose value moved, shop and pack offers). A run takes about a fifth of the space. The replay still reads format 1, and `cargo run -p ante-core --example log_summary` prints a log one line per command.
- [Content coverage](docs/CONTENT_COVERAGE.md) lists the Jokers, decks, stakes and unlock rules Ante plays and those still missing.
- Score popups look like `attention_text` with a `backdrop_colour`: white text with a shadow over a square of the popup colour that turns slowly and grows (one `Particles` square, 0.96 units a second), chips and money at text scale 1 and the others at 0.7.
- Jokers announce their growth during a hand: Wee Joker, Square Joker and Runner show "Upgrade!" in blue, Spare Trousers in red, Green Joker "+1 Mult", before the value they add. The engine records a `JokerUpgrade` step when a Joker's value grows, and the adapter keeps the Jokers whose original text announces it (`ScoreSource::JokerMessage`).
- Score flames: the original `flame` shader burns behind the Chips and Mult boxes once a hand's Chips × Mult reaches the blind's score, at `log₅(score) − 2` with the easing of `G.FUNCS.flame_handler`, and none while a pack is open. The fire and organ ambience (`ambientFire1` to `3`, `ambientOrgan1`) follows it like `modulate_sound`.
- More original sounds: `coin7` on Cash Out, `coin1` when money changes, `chips2` when hands or discards change, `highlight2` and `generic1` on a new ante, `timpani` and `generic1` on a new round, the `button` sound when the total shows (then `chips2` as it is added), and `whoosh1` with a `crumple` when shop, reroll and pack cards materialize. The importer copies `coin1`, `coin7`, `highlight2`, `timpani`, `whoosh1` and the four ambient loops.
- Open pack cards bob like the original `consumeable` area (`0.05 × sin(3.332·t + x)` units) until one is selected, and the drawn hand of an Arcana or Spectral pack sits above the pack cards, which are drawn over it.
- The Ox, The Hook and The Tooth: the Ox empties your money when you play your most played hand, the Hook discards two random held cards with each hand played, the Tooth costs $1 per card played. Each pays Matador. Runs that meet The Hook draw two more random numbers per hand, so their seeds play out differently from earlier versions.
- A hand the boss refuses shows "Not Allowed!" above the play area, as `play_area_status_text` does, with the `cardFan2` and `tarot2` sounds and a room shake; the engine marks it with a `NotAllowed` score step. `tarot2` joins the imported sounds.
- Music: the five original tracks loop together like `RESTART_MUSIC` (pitch 0.7, volume 0.6) and crossfade at `3·dt` toward the track `modulate_sound` picks: music1 in menus and blinds, music5 on a boss blind, music4 in the shop, music2 in Arcana, Spectral, Buffoon and Standard packs, music3 in Celestial packs. A game over slows them toward half pitch. The tracks load from the local installation; the importer copies `music1` to `music5`, and without them the game stays silent.
- Opening a pack, bought or given by a tag, plays `explosion_buildup1` then `explosion_release1` at the delays of `Card:open` and `Card:explode`, scaled by the game speed (`ScheduledSfx` plays sounds after a delay).
- The Audio settings tab has the three sliders of `G.UIDEF.settings_tab('Audio')` (`create_slider`): Master, Music and Game volume, dragged along the track and saved on release. The profile (version 4) stores `music_volume`, 100 by default; version 3 profiles keep their volumes.
- The main menu draws the original `splash` shader over the whole window (`G.SPLASH_BACK`: RED and BLUE, `vort_speed` 0.4); the still swirl remains the fallback without the shader.
- Round evaluation after a beaten blind, after `add_round_eval_row`: the Blind row (chip, target, stake), then the remaining hands, Joker bonuses, tags and interest under a dotted line, each row and dollar sign appearing on the original event timing with the `cancel`, `highlight1` and `coin3` sounds, then the "Cash Out: $N" button. Rows come from `RunView.reward_rows`, now with an `Interest` row.
- The deck view follows `G.UIDEF.view_deck`: the deck name and description, base and effective tallies (aces, faces, numbers, suits) with the ui_assets icons, the rank column, and one `title` card row per suit at 0.7 card size. The Remaining tab greys the cards already drawn with the original `played` shader instead of hiding them.
- The Collection follows `create_UIBox_your_collection*`: the hub with its Consumables group, one black box of card areas per page (Jokers 3 × 5, Tarots 5 + 6, Planets 6 + 6, Spectrals 4 + 5, Boosters 2 × 4 at 1.27 size, Vouchers in overlapping pairs, Enhanced 2 × 4, Seals, Editions on the Joker), a red "Page n/N" cycle, the Tags grid, the Blinds grid beside the ante table and the deck carousel with its pips. Undiscovered cards show the original undiscovered centre with the "?" over it; collection cards hover like hand cards.
- Shared pieces: `tiles::generic_options` (JOKER_GREY frame, L_BLACK body, full-width Back), `tiles::option_cycle` and `components/card_row.rs` (a card area drawn as a row with the slot fan).
- Hovered cards lean toward the pointer like the original: a vertex stage written for Ante (`components/tilt.wgsl`) ports the `position()` stage of the card shaders, and card sprites inside a card switch to it automatically, with their edition overlays.
- Cards lean with their sideways speed while they travel (`Moveable:move_r`: 0.015 × the x velocity, eased by `exp(-190·dt)`), then straighten.
- "Tarot", "Planet" and "Spectral" in descriptions catch a light sweeping across them every 2.4 seconds. This is an Ante addition; the original draws them in their plain set colour.
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

- The profile moves to version 5 with the unlocked items, career statistics and wins per deck. Earlier profiles keep their discoveries and settings but start with everything the original locks locked; "Unlock all" (Dev build) still opens everything.
- Joker descriptions write card ranks as figures, like the original: "each scored 10 or 4", "every scored 2", "Ace, 2, 3, 5 or 8".
- The documentation is in English: the roadmaps, the interface guides, the game logic notes, the capture library and the two project skills, so outside contributors can follow it.
- The Blinds tab of Run Info shows the blind choice panels side by side, as `G.UIDEF.current_blinds` does: each blind prints its state (Current, Defeated, Skipped, Upcoming or Select, in its colour) instead of the Select button, and the tag shows "Skip Reward" instead of the Skip Blind button.
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

- The "Scoring..." label (current score and Joker), the Skip button and the Speed button are hidden while a hand scores: with the hand lowered they no longer fit. Their code is commented out in `screens/hand.rs` until their place is decided.
- A Stone card has no suit, like the original `Card:is_suit`: it no longer counts for a Flush, for suit Jokers (Greedy, Lusty, Arrowhead…), and a suit boss such as The Club no longer debuffs it.
- A Negative Joker destroyed by an effect (eaten, extinct, sliced) gives its extra slot back, as selling one does.
- Opening the Deck view no longer shows it loading twice: the pointer leaving the deck pile under the new view rewrote the session and rebuilt the screen a frame later.
- Faceless Joker pays $5 when at least 3 face cards are discarded, like the original, instead of for any 5 cards. Mail-In Rebate no longer pays for a debuffed card.
- A debuffed Purple Seal no longer creates a Tarot when discarded, like `Card:calculate_seal`: under The Club, discarding a Purple Seal Club gave one.
- A held Gold card pays its $3 when the round ends, like `get_end_of_round_effect`, instead of after every hand played. A Red Seal or Mime repeats it, and a debuffed Gold card pays nothing.
- A Blue Seal held at the end of the round makes the Planet of the last hand played (`G.GAME.last_hand_played`), not a random Planet. A Red Seal or Mime repeats it while there is room.
- A Joker's price and sell value count its edition (Foil +$2, Holographic +$3, Polychrome and Negative +$5), and discounts round like `Card:set_cost` (`floor((cost + 0.5) × (100 − discount) / 100)`), so a $5 item costs $4 with Clearance Sale. Selling and Temperance follow the discount too.
- While a hand is played, the hand drops back to the bottom of the room, level with the deck, as `CardArea:move` does once the buttons go away. The played cards sit 3.6 units above it, like `G.play`, instead of 1.9 units higher, over the Jokers. The Scoring label and the Skip and Speed buttons wait above the deck.
- Selling or using a selected Joker or consumable dissolves it again: a selected card now keeps its hover tilt, whose material replaced the sprites the dissolve shader draws over, so the burning card gets plain sprites back first.
- Playing a hand no longer saturates: the scoring cards rise and fall without a sound of their own, since selecting and playing the cards already sound.
- Tooltip and interface text no longer loses a column of a letter or shows an edge of another letter: Bevy placed antialiased glyphs at fractional positions, which the nearest sampling of the font atlas cut, so `snap_glyphs` puts each glyph's corner on a whole physical pixel after layout. The text keeps its antialiased look.
- Whole test runs against the bosses (`game/tests/boss_runs.rs`) found two engine faults, now fixed: the "Not Allowed!" step was dropped from the trace because it changes no value, and The Ox never recorded the most played hand because the beaten boss's blind was forgotten first.
- Pack cards show the same popup as in the shop: a Joker carries its rarity badge, edition and side boxes, a Tarot, Planet or Spectral its set badge (`ShopItem.joker` and `ShopItem.consumable`). Consumable set badges also read "Tarot", "Planet" and "Spectral" with their colour instead of lower-case text.
- The Psychic, The Eye and The Mouth refuse the hands they forbid, like `Blind:debuff_hand`: a hand of fewer than 5 cards against The Psychic, a hand type already played this round against The Eye, or another hand type than the round's first against The Mouth scores 0, and Matador earns $8 on it. Before, The Psychic let a one-card hand score and Matador never paid against these bosses. The "Not Allowed!" message is not shown yet.
- Selecting a shop, pack or inventory card no longer rebuilds the shop, the pack box and the inventories: cards are spawned once and `components::selection` raises the selected one, brings it to the front and shows its buttons in place.
- A disabled Play Hand, Sort or Discard button stays grey: hovering or releasing it painted it back in its colour, so Discard turned red again once the discards ran out.
- A click on a button stops there, so a card's Buy, Sell or Use button no longer also toggles the card under it.
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
