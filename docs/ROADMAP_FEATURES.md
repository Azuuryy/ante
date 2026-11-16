# Feature roadmap

This roadmap tracks detailed behaviour, surface by surface. The high-level roadmap stays in [ROADMAP.md](ROADMAP.md); this list splits the mechanics of the [game logic notes](GAME_LOGIC_WIKI.md) into verifiable work.

## Statuses

- **To map**: rules and edge cases still to record.
- **To do**: confirmed missing from the client or the target engine.
- **In progress**: part of it is present, acceptance is not finished.
- **To verify**: code present, behaviour or rendering not confirmed.
- **Done**: criteria and edge cases verified.

## Hand, cards and score

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Selecting and deselecting hand cards | In progress | Check the mouse, the maximum selection and the selection kept after actions. |
| Automatic sort by rank / suit | In progress | Commands present; check the order and the kept selection on both engines. |
| Manual hand reordering | In progress | The card follows the grab point and changes position on release. Check the scoring order and the behaviour after a draw. |
| Hand detection and priority | In progress | Compare every hand type and enhanced card with the reference. |
| Chips × Mult preview and resolution | In progress | Validate each contribution and the order of steps in the trace. |
| Enhancements and editions | In progress | Cover effects, stacking, triggers and the interaction with debuffs. |
| Jokers with evolving values | Done for the jokers recorded, to validate in game | The `joker-rules` engine patch aligns Green Joker, Castle, Obelisk, Yorick and Joker Stencil with the original, their state carried by the Joker (`JokerState.counter`). Hit the Road and Steel Joker already matched. The "Currently" line reads this state. Left: older engine saves lack these counters (they restart from zero). |

## Inventories and ordering

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Manual Joker reordering | In progress | The drop changes the simulation order and re-registers the effects. Check the scoring windows and copies. |
| Joker effect order | In progress | Check trigger windows, editions, copies and retriggers with traces. |
| Consumable reordering | In progress | The drop moves the instances. Check uses, sales and resumes. |
| Selling and using consumables/Jokers | In progress | Original Sell (green) and Use (red) buttons, selected card in front. The Emperor creates a Tarot from full slots. Check the rendering in game at 1920 × 1056. |
| Held tags and triggering | In progress | Check the order, removal and stacking of end-of-round tags. |

## Packs, shop and economy

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Shop purchase and reroll | In progress | Panel anchored like the original (−5.3 below the hand, down to the bottom), pixel boxes, dynamic colours; "DEFEAT / BOSS BLIND / TO RESTOCK" in the empty voucher area; orange Buy, red Buy & Use (`Command::BuyAndUseConsumable`). Check proportions, extra offers and transitions in game. |
| Standard / Buffoon / Arcana / Celestial / Spectral packs | In progress | Check contents, number of choices, limits and return to the calling phase. |
| Pack card descriptions | To verify | The code already attaches `item.description`; check that no selection state replaces it with a generic hint. |
| Targeting from a pack | Done, to validate in game | Free selection in the pack's hand, card brought forward then USE or SELECT, Skip unchanged. |
| Vouchers and prerequisites | In progress | Eligibility, discount, immediate effects and persistent perks. |
| Reward, cash out and interest | In progress, to validate in game | The engine keeps the `evaluate_round` rows (Blind, remaining hands, Joker bonuses, tags, interest) until cash out; `RunView.reward_rows` exposes them. The original screen reveals them at the pace of `add_round_eval_row`, with its sounds, then shows Cash Out. |

## Playing cards: state and effects

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Gold Seal | To verify | Check assignment, the payout on scoring and retriggers. |
| Red Seal | To verify | Check assignment and the retriggered activation, with other retrigger sources. |
| Blue Seal | To verify | Check the conditional Planet creation at end of round and capacity. |
| Purple Seal | To verify | Check the Tarot on discard and consumable space. |
| Seal assignment | In progress | Test Spectral cards, card effects and seal replacement/exclusivity. |
| Engine debuff | In progress | Compare the rules per blind and Joker; check which contributions are cancelled. The Psychic, The Eye and The Mouth refuse their hands (score 0, "Not Allowed!", Matador pays); The Ox, The Hook and The Tooth apply. Whole runs against them are tested (`game/tests/boss_runs.rs`). Left: the boss's juice and the second `tarot2` note when its ability triggers. |
| Debuff visuals | In progress | Hand and Deck cards show a red cross and a red tint; compare opacity and extend to other surfaces if they can carry the state. |
| Seal visuals | In progress | The hand and the Deck show a single seal on the left, under the rank corner. Check readability in game. |

## Motion, hover and tooltips

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Hover/lift of hand cards | In progress | `HoverMotion` component: scale +0.05 on the 0.95 base smoothed by `exp(-60·dt)`, position by `exp(-50·dt)`, hover juice (0.05 / 0.03) over 0.4 s, no vertical lift on hover as in the original. A card held by a drag keeps its transform. Selection: lift of 0.2 × card height. To compare in game with `run_hand-1-selected`. Cursor tilt: to do. |
| Hover of held Jokers and consumables | In progress | Shared motion on cards and the action area; check it does not hide the actions. |
| Hover of pack cards and offers | In progress | Shared motion on offers and chosen cards; check each type. |
| Hover in Full Deck and Remaining | In progress | Motion and tooltip present; check that the draw order is not revealed and the hitbox after zoom. |
| Contextual tooltips | In progress | Original anchoring: above the card, below it if its top is within 0.8 card heights of the room's top, on the left for shop offers and the Deck view, then clamped to the window. Structured "(Currently …)" line (`CurrentValue`) for 17 jokers, coloured like the original. Deck view: only the attributes present. Side boxes (`info_queue`) left of the popup, centred on it: the effect's references (Marble Joker → Stone Card, The Chariot → Steel Card, Wheel of Fortune → editions, Spectral seals, tag editions), then edition, seal and stickers. Texts written by Ante. Left: references of uncovered Jokers (Diet Cola, Perkeo…), tag boosters, the Planet named by the Blue Seal. |
| Selection animation | To verify | Apply uniformly to the listed categories and keep controls reachable. No tween for now. |
| Tooltip debug panel | Done, to validate in game | Dev build only: separate light grey panel under the tooltip (id and instance, engine state, raw current value, price and sell value, edition and stickers, atlas coordinates). Covers jokers, consumables, hand cards, offers, packs and the Deck view. Deck back on the board: no tooltip (draw preview). |
| In-game debug tools | Done, to validate in game | Dev build: run with $1000, Tab panel (+$10, +$100, +1 hand, +1 discard, +1 round, +1 ante, +100 score, win the blind, lose the run) through `Command::Debug`. The run is flagged `debug`. F6 saves a capture to `%LOCALAPPDATA%\Ante\captures\`. |

## Collection and progression

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Joker, voucher and tag collection | In progress | Paging and discovery present; items with an unlock condition show a padlock, a "Locked" tooltip and their condition. Check each condition and the padlock rendering at real size. |
| Card, booster, enhancement, edition, seal and blind collection | Done, to compare | Pages after `create_UIBox_your_collection_*`: card areas, page cycle, Tag and Blind grids, ante table. |
| Unlocks and career | In progress | Profile JSON: `version`, `discovered`, `all_unlocked`, with migration from older versions. The "Unlock all" debug button is in Settings > GAME (`dev-tools`). Discovery still stands in for real unlocks; counting career statistics is still to do. |
| Decks, stakes and challenges | To map | The 15 decks have an id and an unlock condition shown in the Collection. Stake rules, locked selection and challenges are not wired. |

## Consumables, targeting and conditions

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Single availability rule | Done | A single function (`consumable_usable`) decides Use, for the generator as for the application. The check happens before the card is removed. In the Shop, Spectrals that touch the hand stay blocked (empty hand). |
| Planets and Tarots without a target | In progress | Usable in PreBlind, Blind, PostBlind, Shop and while a pack is open, like `can_use_consumeable`. Covered by `consumable_use_availability_follows_targets_and_stage`. |
| Targeted Tarots (1 to 3 cards) | In progress | Usable during a Blind or an open Arcana/Spectral pack, when the number of selected cards is within `min_targets..=max_targets`. The "Use" button only appears once the selection is valid; the reference first enters a targeting mode. |
| Spectrals without a target | In progress | Same window as Planets, open pack included. |
| Spectrals that touch the hand | Done, to validate in game | Familiar, Grim, Incantation, Sigil, Ouija and Immolate are usable in a Blind as soon as the hand has more than one card, without a selection, as in the reference. They stay refused outside a Blind, where the hand is empty. |
| Targeting from a pack | Done, to validate in game | The hand of an Arcana or Spectral pack is selected first; a click brings the pack card forward, and its USE button (red if the selection fits) uses it at once with those targets. The `TarotHand`/`SpectralHand` stages no longer exist. |
| Missing "Use" and lost cards | Fixed, to validate in game | Cause: duplicated availability rules, checked after the card was removed. Familiar, Grim, Incantation, Immolate, Sigil and Ouija become usable in a Blind as soon as the hand has more than one card, regardless of the selection. Judgement, Wraith and Soul need a joker slot; Ankh, Hex, Ectoplasm and Wheel of Fortune need a joker; The Fool needs a last used consumable other than itself; Aura needs a card without an edition. Hex and Ectoplasm only target jokers without an edition. Patch `balatro-rs-consumable-use.patch`, 16 dedicated tests (`consumable_use_tests.rs`). |
| Consumable description and state | In progress | Description, category and dev-tools technical fields present; check the text of each Tarot/Spectral. |

## Profile, unlocks and collection

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Profile format | In progress | `profile.json` version 4, reads versions 1 to 3, atomic write. Fields: `discovered` (sorted set), `all_unlocked` (boolean), `game_speed` and the `volume`, `music_volume` and `game_volume` settings. |
| Debug Unlock All | Done | Covers every category and stays available after a first unlock, for categories added later. |
| Collection lock | In progress | `Content.unlock` carries the condition; locked entries show a padlock, a "Locked" tooltip and the condition. Legendary Jokers point to the Spectral pack. |
| Achievement unlocks | To do | Conditions are descriptive and not evaluated. Missing: an `unlocked` set distinct from `discovered`, career statistics and the condition checks (discarded Royal Flush, 5 Jacks, etc.). |
| Collection counters | Done | Each hub button shows discovered / total. The overall percentage is not shown. |
| Other categories | Done, to compare | Tarot, Planet, Spectral, enhanced cards, seals, editions, boosters and blinds are active, with their original visuals. Consumables, packs and bosses met are discovered in game; enhancements, editions and seals only through Unlock All for now. Known defect: at 1920×1057, the pages' ">" arrow is missing. |

## Shaders and VFX

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Strategy | Decided | The original shaders are reused from the user's installation, never committed or shipped. The Local Assets import copies the allowlisted `.fs` files to `%LOCALAPPDATA%\Ante\local-assets\shaders\`. On load, a LÖVE compatibility prelude written by Ante (`extern`, `Image`, `Texel`, `effect` → `main`) is added, then naga converts the GLSL to WGSL for a `UiMaterial`. Falls back to no effect if the file is missing or the conversion fails. |
| Loading the `.fs` files | Done, rendering to validate | The 19 files of `resources/shaders` are LÖVE 11 GLSL (`effect`/`position`, `extern`, `number`, `Image`, `Texel`, `love_ScreenSize`). Bevy 0.20 loads `wgsl`; the conversion goes through naga's GLSL frontend after the prelude. 17 of 19 shaders converted and validated by naga on the local installation; `skew` and `vortex` are vertex only. |
| Uniforms and draw order | Done | Recorded in the [legacy guide](UI_LEGACY_GUIDE.md#shaders): `time`, `dissolve`, `texture_details`, `image_details`, `burn_colour_*`, `shadow`, a parameter named like the shader, and the `Card:draw` / `Sprite:draw_shader` order. |
| Foil / Holo / Poly / Negative editions | In progress, rendering to validate | Original shaders overlaid on Jokers (held, shop) and playable cards (hand, play area, Deck view), on the base and the face like `Card:draw`. Left: `negative` replacing the base instead of covering it, tilt and juice in the shader parameter, consumables. |
| Dissolve, flame, played, debuff, booster, voucher | In progress | `dissolve`: a Joker or consumable leaving the inventory burns over 0.7 s like `Card:start_dissolve` (gold when sold, otherwise black and orange), with particles and sounds; verified in game. `debuff` on debuffed cards (red cross fallback without the shader), `booster` and `voucher` in the shop and in Run Info. Left: `played`, `flame`, dissolving cards destroyed in hand. |
| Skew / vortex / CRT / splash / background | In progress | `background` (runs) and `splash` (main menu, without the splash screen's flash) are ported. Left: skew, vortex and CRT, same pipeline. The vertex warps assume LÖVE screen coordinates to reproduce; the CRT is a multi-pass post-process. |
| Non-shader effects | In progress | The original room shake (`update_canvas_juice`: +0.7 per popup, damped by (1 − 5·dt), slow drift and cursor pull), dissolve particles. The room's slight permanent rotation is left out: with crisp pixel edges it creates a crawling step. |

## Animation and audio

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Score playback | In progress | Timeline in game seconds with the original delays (entry 0.4 + 0.1/card + 0.2 + 0.1/card + 0.4; chip popups 0.75, mult 0.8125, joker 0.9375, retriggers extra; end 0.1/card + 0.4 + 0.8 + 0.5 + 0.3; discard 0.1/card). Speeds x0.5 to x128 by doubling, the original automatic acceleration (+0.2·speed/s, capped at 16, effective above 2), speed saved in the profile. Instant removed, Skip kept. Each step shows its popups one after another like `card_eval_status_text`: "Again!", chips, +Mult, XMult ("X1.5 Mult") then money ("$3"), with the source's juice, above the cards and below the Jokers. The `trace-effects` engine patch records the XMult factor and the money of each step. Mult is fractional as in the original (`fractional-mult` patch) and printed with the `number_format` rules. Left: time it against the original. |
| Hover | In progress, to validate in game | Original smoothing (see Hover), hand fan, tilt toward the cursor (`tilt.wgsl`, after the card shaders' `position()`) and lean while travelling (`Moveable:move_r`). |
| Card movement | In progress, to compare | Play area 3.6 units above the hand. Travel animated with the `Moveable:move_xy` speed model (gain 35, `exp(-50·dt)` smoothing, cap 70 units/s): draw from the deck, hand → play area, then play area → discard off screen on the right, one card every 0.1 game seconds. A card waiting to be drawn stays hidden in the deck. Selection, sorting and drag drops glide too. Cards discarded with the Discard button leave one by one for the discard pile, then the draw starts, like `discard_cards_from_highlighted` followed by `draw_from_deck_to_hand`. The hand only spreads over the cards that arrived: it tightens during a discard and widens with each card drawn. A drawn card leaves the deck face down and flips as it enters the hand (`Moveable:move_wh` pinch, 0.125 s to close then 0.125 s to reopen). The fan follows the number of cards present. Left: the `card1` sound, flipping cards turned over by bosses. |
| Score juice | In progress | Popups over a turning colour square, source juice, Chips and Mult value pulses, room shake, Joker upgrade messages and score flames (`flame` shader, `flame_handler` easing). Left: particles of big scores, the messages of Jokers that grow outside the scoring pass (Castle, Hit the Road, Ride the Bus reset). |
| Edition and destruction VFX | In progress | Editions and dissolve: see the Shaders and VFX section. LÖVE shaders compute in gamma space; Ante converts input and output, and edition layers are drawn after the sprite. |
| Audio | In progress | Original sound effects read from the local installation (36 effects, 5 music tracks and 4 ambient loops from the allowlist, copied by the Local Assets import): hover, selection, cards leaving, score popups, total, purchases and sales, buttons, dissolve. Pack opening explosion. Volume `vol × master/100 × sounds/100`. Music: five tracks crossfaded by screen, `vol × 0.6 × master/100 × music/100`, pitch 0.7 slowed on a game over. Master (50), music (100) and sound (100) settings in Options > AUDIO, saved in the profile. Fire and organ ambience follow the score flames. Money, hands, discards, ante, round, cash out, materialize and scoring highlight sounds follow the `ease_*` functions. Left: pitch variation of the effects. |

## Interfaces, saves and compatibility

| Feature | Current status | Work / exit criterion |
| --- | --- | --- |
| Run Info, Deck and Pause modals | In progress | Check the return to the phase and that selections are kept. |
| Save / resume | To verify | Compare snapshot/resume on pack, shop, trace and manual order. |
| RNG and seeds | To map | Document the streams per system before any promise of seed compatibility. |
| Win, loss and Endless | In progress | Check transitions, ante 8 and the representation of large scores. |
| Multi-size visual comparison | To verify | Reference and client captures at equal size; document gaps and VFX separately. |

Update the statuses after each piece of work with the code path or acceptance check that justifies the progress. Only move to **Done** after functional verification and, for visual surfaces, a capture comparison.
