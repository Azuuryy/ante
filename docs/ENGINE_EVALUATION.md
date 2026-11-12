# Engine evaluation (2026-11-02)

Status: preliminary; no external engine has been adopted. The isolated experiment requires local checkouts in `target/engine-evaluation/`: balatro-rs at `3d66de905d5824bcb92c453214dc2badfbb9fc3f` and Balatrust at `57c3512844243e3fdd3ef30285dc417e918543ab`. Clone the listed revisions into those paths and apply the eleven patches in the order below before testing.

Balatrust includes an MIT license naming Yannick Herrero; `balatrust_core` depends on rand. balatro-rs core declares MIT in its manifest, but no root LICENSE was found; it depends on balatro-types and balatro-seed as well as rand, rand_chacha, itertools, indexmap, strum and thiserror, with serde enabled by default. Its seed crate identifies itself as a port and attributes research to TheSoul. Provenance and rights of all components remain unverified; no upstream code is copied into Ante.

On Windows 11 with Rust 1.99: `cargo test -p balatro-rs` passed 479 unit tests and 2 integration tests (2 ignored); `cargo test -p balatrust_core` passed 25 unit tests. The independent experiment passed four tests: same-seed state after save and a subsequent discard; non-mutating score preview and play; one pair with kickers; one wild flush. These tests do not establish compatibility with Balatro. The experiment selects Fast mode but balatro-rs still builds the balatro-seed dependency; it must not be shipped without the provenance audit.

Observations: Balatrust returns scoring indices, but accepts an empty hand as HighCard rather than reporting an error. balatro-rs rejects empty and more than five cards; its score trace can be generated on a cloned game, but this has not been established as equivalent to every play action, especially random effects. Its scoring uses usize and its effects registry is skipped in serde. Reconstitution of active effects and exact continuation still need dedicated tests. Balatrust uses StdRng and has no verified versioned snapshot interface. Neither engine has been benchmarked in this evaluation.

Decision: retain ante-core for production for now. Evaluate balatro-rs actions, hand detection and trace as candidates after provenance audit and further tests; Balatrust scoring steps and round flow are useful comparisons. Do not import either RNG or engine wholesale before provenance, deterministic save/load with active effects, all twelve categories, ordering, progression, animation independence, and performance are verified. No rendering FPS or simulation throughput is claimed.

## Follow-up (2026-11-08)

Both evaluators classified one independently assembled example of each of the twelve hand categories as expected; this does not verify all card combinations or compatibility. The adapter now uses `Game::from_json` to rebuild the effect registry and advance card IDs. A focused test confirms that one Hanging Chad registration is restored; matching effects and random outcomes after continued play remain unverified. Six experimental tests pass on Windows.

## One ante Bevy MVP

The experimental binary uses `balatro-rs` in Fast mode. The default campaign goes to ante 8. Pass `--one-ante` to replay the historical route below. The production workspace remains independent of the experiment. This milestone validates the interaction flow with generated UI cards and requires no game assets.

Use the two checkouts pinned above. On a fresh pinned `balatro-rs` checkout, apply the patches once from the Ante root. The eleven patches, in order: align the optional UUID dependency (`bevy-uuid`); expose the score trace from the actual play action (`play-trace`); add the optional Joker rules (`jokers`); extend the campaign with bounded Endless (`endless`); add the tag and shop transitions (`shop-tags`); restrict consumable use to the original eligibility rules through a single `consumable_usable` predicate, with 16 dedicated tests in `core/src/consumable_use_tests.rs` (`consumable-use`); add the `debug_add_score` and `debug_next_ante` entry points used by the Dev panel, with 4 tests in `core/src/debug_tests.rs` (`debug`); make the Joker counters and the full-deck and non-debuffed helpers readable so the tooltip can show the value an effect will use (`joker-current`); record on each score step the factor of a multiplicative Mult change and the money earned, without changing any computed value, with 3 tests in `core/src/trace_effect_tests.rs` (`trace-effects`); bring five Joker rules in line with the original, keeping their state on the Joker itself: Green Joker (+1 Mult per hand, -1 per discard, kept for the run), Castle (+3 Chips per discarded card of the round's suit, kept for the run), Obelisk (X0.2 per hand that is not strictly the most played, per card), Yorick (X1 more per 23 cards discarded since it was acquired) and Joker Stencil (X empty slots plus Stencils held), with the affected tests rewritten and 2 new ones (`joker-rules`); keep Mult fractional like the original, so XMult no longer truncates and the score is floor(chips x Mult), and halve The Flint's base values to the nearest whole number with Mult at least 1, with the affected expectations recomputed (`fractional-mult`). The `jokers` patch also makes `test_exactly_74_jokers_implemented` count the three optional jokers. On the patched checkout, `cargo test -p balatro-rs` passes 511 unit tests, with or without `ante-extra-jokers`.

```bash
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change --unidiff-zero ../../../experiments/engine-evaluation/patches/balatro-rs-bevy-uuid.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-play-trace.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-jokers.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-endless.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-shop-tags.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-consumable-use.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-debug.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-joker-current.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-trace-effects.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-joker-rules.patch
git -C target/engine-evaluation/balatro-rs apply --ignore-space-change ../../../experiments/engine-evaluation/patches/balatro-rs-fractional-mult.patch
```

Launch from the Ante root in native Windows. Select Play, enable Seeded Run, and type 0 to replay the route below.

```powershell
cargo run --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features mvp --bin ante-mvp -- --one-ante
```

Replay the verified win with seed **0**. The numbers below are card positions from left to right in the current hand, counted again after each play or discard.

| Blind | Actions | Expected result |
| --- | --- | --- |
| Small | Discard card 1. Play 4/5/6/7/8, then 1/5/6/7/8, then 3/5/6/7/8, then 2/3/4/7/8. | 349 / 300. Cash out, then click Next blind. |
| Big | Play 1/2/3/7/8, then 3/5/6/7/8, then 1/2/4/7/8, then 3/5/6/7/8. | 500 / 450. Cash out, buy the second offer named Joker, then click Next blind. |
| The Manacle | Play 1/2/3/5/6, then 3/4/5/6/7. | 672 / 600 and victory. |

To replay the loss, click Retry same seed and start Small. Check that Play hand is disabled without a selection. Select and play only the first card four times. Check the loss screen and retry once more.

The two scenarios also run through the same adapter without opening a window. They verify the cash out and purchase, displayed scores across transitions, deterministic observations across runs, and rejection of empty play. The initial native window walkthrough was confirmed by the user. The diagnostics, animations and atlas presentation below have their own manual checks.

## Score stress test (2026-11-09)

```sh
cargo run --manifest-path experiments/engine-evaluation/Cargo.toml --example score_stress --release --locked
```

The benchmark times the score pipeline and the copy of the prepared hand it consumes. It measures the median of nine batches after warm-up on an Intel Core i7-14700KF. Hand selection, rendering and animation are outside the timed region. The test runs five Mult cards with Red Seals, synthetic extra retriggers through the engine's effect registry and one Polychrome Joker. The trace confirms the activation count and both scoring paths match the expected score.

| Card activations in one hand | Windows, no trace | Windows, with trace | WSL, no trace | WSL, with trace |
| ---: | ---: | ---: | ---: | ---: |
| 10,010 | 9.91 µs | 135.03 µs | 11.86 µs | 31.43 µs |
| 50,010 | 46.71 µs | 1,290.79 µs | 65.91 µs | 157.90 µs |
| 100,010 | 95.06 µs | 2,690.40 µs | 111.58 µs | 357.09 µs |

The Glass variant applies ten actual ×2 effects per hand. A batch of 1,000 hands took about 0.30 ms without the trace and 0.42 ms with it on Windows. A single hand with 10,000 successive doublings would exceed the engine's `usize` score capacity.


## Diagnostics, score playback and Joker collection

Use `--features mvp` for the original seed 0 route. Use `--features mvp,extra-jokers` to add Misprint, Golden Joker and Ice Cream to the shop pool. The expanded pool can change the offers for the same seed. These three behaviours are independent implementations of the documented Joker rules. [Misprint](https://balatrogame.fandom.com/wiki/Misprint) · [Ice Cream](https://balatrogame.fandom.com/wiki/Ice_Cream) · [Golden Joker](https://balatrogame.fandom.com/wiki/Golden_Joker)

- Click **Speed** to cycle through ×1, ×4 and Instant. During playback, **Skip** reveals the committed result. The displayed animation uses the same trace returned by the actual play action. The test route checks the same score and transitions in all three modes.
- The seed menu opens the **Collection**. Purchases record discovery in `%LOCALAPPDATA%\Ante\profile.json`. A corrupt profile is preserved and the collection works in memory.
- **F3** toggles the frame profiler in its own window. **F2** clears its rolling ten-second history and **F4** exports the current samples and recent logs to `%LOCALAPPDATA%\Ante\logs`. The files are `ante-report.json` and `ante-frames.csv`. Fatal failures also attempt to write `ante-crash.json`. Release builds check integer overflow.
- The profiler separates application CPU timings, frame intervals and the most recent CPU and GPU render timings available from Bevy. CPU layout timings are wall time around the UI layout set. Render passes can overlap or nest and should be read individually. Trace timings include instrumentation overhead. [Bevy RenderDiagnosticsPlugin](https://docs.rs/bevy/0.20.1/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html)

### Local textures and manual performance comparison

Run the reference route in native Windows, with the same speed and graphics settings for each capture.

```powershell
cargo run --release --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features mvp --bin ante-mvp
cargo run --release --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features mvp --bin ante-mvp -- --assets "C:\Program Files (x86)\Steam\steamapps\common\Balatro\Balatro"
```

1. Use the first command for placeholder cards and the second for explicitly authorised local textures. The menu reports the asset status. Check the four suits, face cards and a mapped Joker in the shop. The 1x atlas covers all 52 card faces and a subset of supported Joker portraits. Other portraits use text.
2. In each mode, run the seed 0 route once with F3 closed and once with F3 open. Use F2 immediately before the same scoring hand and F4 shortly afterwards to retain that sample. Copy each CSV to a distinct name before the next export. Compare frame median, p95 and peak from the `frame` metric and the matching non-zero CPU and render measurements. The startup atlas load is exported separately.
3. Confirm that closing the profiler leaves the main window open. Verify the same final scores with ×1, ×4 and Instant, and retry after a loss. The comparison of native presentation timings is part of this manual recipe.

### Behaviour coverage

| Area | Verified in this branch | Next compatibility checks |
| --- | --- | --- |
| Hand types | One example of each of 12 categories, one wild flush, basic invalid sizes | Additional combinations, scored subset and order under modifiers |
| Card effects | Scoring parity with Lucky randomness and Hanging Chad repetitions, stress cases for Mult and Glass | Combined editions and seals, effects on held and debuffed cards |
| Jokers | Base pool, purchase and collection persistence, opt-in Misprint, Golden Joker and Ice Cream scenarios | Remaining Joker hooks, copies and conflicting triggers |
| Scoring numbers | Checked overflow in the experiment's Release profile | Representation and formatting beyond machine integers |
| Resources | Opt-in PNG atlas loader with fallback and path and size checks | Remaining Joker coordinates, 2x atlases, richer card layers and other resources |


### Graph and uncapped presentation test

The profiler plots up to ten seconds of frame time. Each bar keeps the slowest frame in its time bucket. FPS in the header is the observed application rate over the last second. The native default is FIFO VSync, so a 240 Hz display can limit a simple scene to around 240 FPS.

**F5** switches both windows between FIFO VSync and automatic no-VSync with a 1,000 updates/s ceiling. Launch with `--benchmark-fps` to select the benchmark mode immediately. The ceiling is an upper bound. Actual presentation still depends on GPU capabilities and the OS compositor. Compare runs with the profiler open and closed using F4 exports, and keep the game or profiler focused.

**--steam-textures** selects the default Windows Steam installation. **--assets "path"** selects another directory. The card renderer combines the face atlas with the base card from `Enhancers.png` and uses its red back for the deck counter. All three atlases are read from the local installation.

```powershell
cargo run --release --manifest-path experiments/engine-evaluation/Cargo.toml --locked --features mvp --bin ante-mvp -- --steam-textures --benchmark-fps
```

### Menu and deck selection

The experimental client opens on a main menu. Play opens deck selection; Red (4 hands, 4 discards) and Blue (5 hands, 3 discards) are available, while the other decks are previews. Enable Seeded Run to enter or paste a seed, or leave it off to start with a random seed. Options are also accessible during a run and preserve it. Score speed and VSync are functional; visual effects are not implemented. At the end of a run, retry retains both seed and deck.

`--steam-textures` or `--assets` also load `resources/textures/1x/balatro.png`, `resources/textures/1x/ui_assets.png` and `resources/fonts/m6x11plus.ttf` locally when valid. Missing or invalid files fall back to Ante text, a plain background and its default font. No game resources are included in the repository. Visual comparison and interaction checks require a manual Windows session.

### Main menu presentation and window scaling (2026-11-09)

The main menu was recreated from the original game's layout, observed read-only in the local installation (v1.0.1o). No original source, assets or shaders are copied into the repository; textures stay optional and are read at runtime.

- **Tile units.** Balatro draws everything in tiles (1 tile = 20 base px x TILESCALE 3.65 = 73 px). `menu_ui.rs` exposes `TILE`, `ROOM` (1460 x 839.5 px, the 20 x 11.5-tile virtual room), `tiles()`, `tile_px()` and `text_px()` so menu geometry is expressed in the original tile values. Colours (`RED`, `BLUE`, `ORANGE`, `PALE_GREEN`, `L_BLACK`) and the emboss/hover channel maths match the observed constants.
- **Window fit.** A fixed-size room root carries a `UiTransform`; the `fit_room` system writes `scale = min(w/1606, h/941.7)` and a centring translation every frame, reproducing the original continuous fit (letterbox/pillarbox, black bars). This replaces the previous fixed-pixel layout that did not scale. `UiScale` is left at 1.0 because it is a global resource that would also scale the separate F3 profiler window; the profiler has no `Screen` component and keeps its own unscaled layout. Resizing only writes a transform, so text is not re-rasterised and picking stays correct at any size.
- **Buttons.** `tile_button` builds a single node with an emboss rim (`Node.border` + `BorderColor`), a native `BoxShadow`, hover overlay + lift and press scale, reusing the existing `Pointer*` observer pattern. The `Click` message contract is unchanged, so the headless navigation tests still apply.
- **Documented deviations.** The animated RED/BLUE swirl shader and CRT/bloom are deferred to polish; the background is a static tinted `ui_assets.png` cell (or a plain colour). The F3/F4/F5 hint line and the Ante version string are evaluation tooling, not present in Balatro. `FONT_BASE` (58.4 px) and `MENU_ROW_Y` (8.85 tiles) are the two values not pinned by the observed source and are named constants for visual tuning.

Manual checks: drag-resize, maximise and snap the window — the menu scales continuously, stays centred with black bars, and buttons remain clickable; the F3 profiler stays fixed-size; with `--steam-textures` the logo, ace of spades, swirl cell and m6x11plus font appear, and without assets the ANTE text and plain background are used; the seed-0 win route is still playable.

### Eight-ante campaign and bounded Endless

The patched engine preserves the existing serialized ante names through Eight and uses a numeric payload above it. The victory and continuation flags default to false when reading earlier snapshots. Endless resumes after victory at the pending cash-out and draws the next ante from the same RNG. On 64-bit builds the White Stake curve admits antes 9 to 15. The next threshold is outside the current score representation. The standalone historical route uses `--one-ante`.


## Client extraction and native slice

The experimental binary assembles `ante-app` with the pinned Fast backend. The adapter converts commands and ordered traces at the boundary defined in `ante-core::contract`. The production workspace resolves no external engine checkout. Ante CLI exercises the native one-blind slice with its own versioned RNG. Differential scoring uses controlled card fixtures, since equal seeds do not identify equal draws across the two engines.

For a new Balatrust checkout inside the ignored target directory, set `workspace = ".."` in the `[package]` section of `balatrust_core/Cargo.toml` if Cargo resolves inherited dependencies against the Ante workspace.
