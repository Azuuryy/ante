# Ante — roadmap

## Milestones

1. **Functional parity and static layout.** Stabilise the simulation contract, reuse the client components and finish the playable flows. The engine vendored in `engine/` (Fast mode) drives the client.
2. **Integrated engine.** Extend the rules of the vendored engine (shops, packs, campaigns and effects) directly in `engine/`, with tests on transitions, scores, traces and resumes. Measure how very large scores are represented before extending Endless. Audit the RNG and the draw order separately before committing to Balatro seed compatibility.
3. **Visual effects.** Record the animations, transitions, movements, parallax and background or edition shaders. Design their Bevy adaptation and check provenance before any reuse.
4. **1:1 polish.** Check fidelity per state and per window size, then fix the rendering, motion and timing gaps.

Detailed tracking of the wiki's mechanics and surfaces lives in [the feature roadmap](ROADMAP_FEATURES.md). The game rules adopted and their implementation status are summarised in [the game logic notes](GAME_LOGIC_WIKI.md).

## Statuses

**Not implemented** surface missing. **MVP** minimal flow. **In progress** playable surface with known gaps. **Done** functional equivalence and close static layout, compared with the reference. **Done + VFX** the reference's effects reproduced. **Exactly 1:1** final fidelity verified.

## Interfaces

The common acceptance check compares a capture at equal client size and walks the flow described in the [legacy](UI_LEGACY_GUIDE.md) and [Bevy](UI_BEVY_GUIDE.md) guides.

| Surface | Status | Gap or next check |
| --- | --- | --- |
| Main menu | In progress | Original `splash` background over the whole window (RED and BLUE), music1. Compare logo, buttons and returns with and without the atlas. The opening flash (`mid_flash`) waits for the splash screen. |
| Profiles and records | MVP | Check persistent discoveries. Add multiple profiles and records. |
| Run setup / seed | In progress | Compare carousel and input. Extend decks and stakes. |
| Pause | In progress | Compare resume, restart and return to the run. |
| Game options | MVP | Check the speed and list the other settings. |
| Video options | MVP | Check VSync, resolution and display modes. |
| Graphics / resources options | In progress | Test import, replacement and fallback. |
| Audio options | In progress | Three `create_slider` sliders (Master, Music, Game) saved on release. Compare slider size and rendering. |
| Collection / hub | In progress | `create_UIBox_your_collection` layout (Consumables group, counters). To compare at equal size. |
| Collection / jokers | In progress | 3 `title` areas of 5, "Page n/N" cycle, undiscovered centre and "?". In-run popups (rarity badge) in every category. The "?" does not sway yet. |
| Collection / vouchers | In progress | 2 `voucher` areas of 4 (stacked pairs). Compare prerequisites. |
| Collection / tags | In progress | Grid of 6 per row, "?" tag for undiscovered ones. |
| Collection / decks | In progress | Carousel with dots, name and description on white. The stack remains Ante's preview. |
| Collection / tarots | In progress | Areas of 5 and 6, 2 pages. To compare. |
| Collection / planets | In progress | 6 + 6 from Mercury to Eris, no cycle. To compare. |
| Collection / spectrals | In progress | Areas of 4 and 5, 2 pages. To compare. |
| Collection / boosters | In progress | 2 × 4 at 1.27 cards in areas of 1.3 cards. To compare. |
| Collection / enhancements | In progress | 2 × 4. The info text is Ante's wording. |
| Collection / editions | In progress | Base then 4 editions on the Joker, with their shader. |
| Collection / seals | In progress | 1 × 4 on a blank card. |
| Collection / blinds | In progress | Grid of 5 in the original order and ANTE / BASE table (antes 1 to 16). The highest ante is not recorded: every amount stays grey. |
| Customize Deck | Not implemented | Card face choice (`G.UIDEF.customize_deck`) loaded from the local import, falling back on the default faces. |
| Mods | Not implemented | To design: Ante's mod API (data, then scripts), loaded before the run and recorded in the run logs. No direct execution of Steamodded mods. |
| Challenges | Not implemented | List conditions, selection and exit. |
| HUD and counters | In progress | Rebuilt after `create_UIBox_HUD`: inner boxes, stake chip, Chips × Mult side by side, animated sign, dynamic colours and pixel corners. To compare at 1920 × 1056; the drop shadow does not follow the horizontal parallax yet. |
| Blind choice | In progress | The panels have no popup, as in the original; the tag next to Skip Blind has its own. Check the three states and the boss objective. A boss that wants its own popup (modding) has no field to provide it yet. |
| Held tags | In progress | Compare stack, acquisition and removal. |
| Hand and actions | In progress | The Play Hand / Sort / Discard row stays in place when the selection changes, and a disabled button stays grey on hover. Compare size, selection and overlap. Check drag and drop and Rank/Suit sorting. |
| Chips × Mult preview | In progress | Check level, empty selection and score readout. |
| Score resolution | In progress | Popups with the turning colour square of `attention_text`, Joker "Upgrade!" messages, score flames and their fire and organ ambience. Check the trace and interactions against the reference. |
| Cash out | In progress | Rows and dollars of `add_round_eval_row` at the original pace, then Cash Out. The panel does not rise from the bottom yet. |
| Shop / cards | In progress | Top at 3.587 units, `UIBox_dyn_container` frame down to the bottom, atomic Buy & Use. Choosing an offer no longer rebuilds the shop. Compare variable offers and controls. |
| Shop / vouchers | In progress | Test multiple purchases, prices and prerequisites. |
| Shop / boosters | In progress | Compare atlas, sizes, prices and opening. |
| Packs / Standard | In progress | Compare card choice and return. |
| Packs / Buffoon | In progress | Pack Jokers have the shop popup (rarity, edition). Compare joker choice and capacity limits. |
| Packs / Celestial | In progress | Check the level gained and closing. |
| Packs / Arcana | In progress | The hand sits above the bobbing pack cards. Check target selection and return. |
| Packs / Spectral | In progress | Check target selection and return. |
| Inventory / jokers | In progress | Compare layout, sell panel and capacities. Check the trigger order after a drop. |
| Inventory / consumables | In progress | Compare drag and drop, the action panel and capacities. |
| Deck / Full Deck | In progress | `G.UIDEF.view_deck` layout: deck box, tallies, rank column, one `title` area per suit. The DynaText that alternates base and effective counts is replaced by both numbers side by side. |
| Deck / Remaining | In progress | Cards already drawn stay displayed, greyed by the `played` shader. |
| Deck / hover grid | In progress | Check counts, hidden cards and effective values. |
| Tooltip / playing card | In progress | Name centred on a white box, chips and enhancement rule, badges and side boxes for edition and seal (`card_h_popup`). To compare at equal size: fixed width of 4.2 tiles against the original's content width. |
| Tooltip / joker | In progress | Compare rarity, state and dynamic values. "Currently" is only computed for observable jokers; check every tracked value and do not expose raw state outside the Dev build. |
| Tooltip / consumable | In progress | Compare badges and values. |
| Tooltip / voucher | In progress | Compare description and discovery states. |
| Tooltip / booster | In progress | Local `p_<family>_<size>` text with "choose N of M". Arcana/Spectral opening: hand visible, box rise and particles; the engine's choice-then-targets order (documented gap). |
| Tooltip / tag | In progress | Check the effect and the Orbital target. |
| Tooltip / blind | MVP | Check the data and focus conditions. |
| Run Info / Poker Hands | In progress | Compare levels and hand counts. |
| Run Info / Blinds | In progress | Blind choice panels (`G.UIDEF.current_blinds`) with their coloured state and "Skip Reward" without a button. Compare size and states with the reference. |
| Run Info / Vouchers | In progress | Check the owned state and navigation. |
| Win, loss and Endless | In progress | Compare the three outcomes and the numeric limit. |
| Card hover and selection | In progress | The hover zoom (+5 %) is shared by the hand, shop, packs, inventories and Deck views. Left: `exp(-60·dt)` smoothing, juice, a selection lift of 0.2 × card height and cursor tilt. |
| Debuff visuals | MVP | Red cross and red tint on the hand, deck and packs. Left: compare with the `vfx_debuff` capture and port the original debuff shader. |
| Seals | In progress | The experimental engine projects the four seals and their descriptions. Test and complete assignment, scoring/discard/end-of-round effects and rendering. |

## Dev tools

Profiler, structured logs, export and hover details are compiled with `dev-tools`. Test the shortcuts in Dev and check that the profiler, Ante logs and exports are absent from Prod.
