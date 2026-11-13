# Engine

Ante's simulation runs on a vendored and patched copy of [balatro-rs](https://github.com/evanofslack/balatro-rs) in Fast mode. The code lives in `engine/` (`balatro-rs`, `balatro-types`, `balatro-seed`); [engine/NOTICE.md](../engine/NOTICE.md) records the upstream revision, the twelve changes Ante applied before vendoring, what was removed, and the unverified provenance. Engine changes are now made directly in these crates, with their tests.

`ante-core::engine` adapts the engine to the contract in `ante-core::contract`: `EngineRun` turns each `Command` into engine actions, returns the confirmed `RunView` and the ordered score trace, and `ExternalBackend` provides runs, collection catalogs and seed parsing to the client. No Bevy type crosses this boundary.

## History

The engine was chosen after an isolated evaluation (2026-11-02 to 2026-11-13) that compared balatro-rs with Balatrust and with Ante's former native one-blind slice. Both evaluators classified one example of each of the twelve hand categories as expected; this does not establish compatibility with Balatro. Balatrust accepted an empty hand as High Card; balatro-rs rejects empty hands and more than five cards. The comparison code, the native slice and its CLI were removed when the engine was vendored.

## Features

- `ante-core/extra-jokers` (relayed by `ante-app/extra-jokers`) adds Misprint, Golden Joker and Ice Cream to the shop pool. The expanded pool can change the offers for the same seed. These are independent implementations of the documented rules: [Misprint](https://balatrogame.fandom.com/wiki/Misprint) · [Ice Cream](https://balatrogame.fandom.com/wiki/Ice_Cream) · [Golden Joker](https://balatrogame.fandom.com/wiki/Golden_Joker).
- `ante-app/dev-tools` builds the Dev client (profiler, logs, technical tooltips, debug panel, $1000 start, captures).

## Campaign and Endless

The default campaign goes to ante 8; `--one-ante` replays the historical single-ante route. The engine preserves the serialized ante names through Eight and uses a numeric payload above it. The victory and continuation flags default to false when reading earlier snapshots. Endless resumes after victory at the pending cash-out and draws the next ante from the same RNG. On 64-bit builds the White Stake curve admits antes 9 to 15; the next threshold is outside the current score representation.

Snapshots go through `Game::from_json`, which rebuilds the effect registry and advances card IDs. A loaded snapshot deals the same cards after a discard (`engine/balatro-rs/tests/snapshot.rs`). Matching effects and random outcomes after longer continued play remain unverified.

## Seed 0 route

Launch from the Ante root in native Windows, select Play, enable Seeded Run, and type 0.

```powershell
cargo run --locked -p ante-app --bin ante -- --one-ante
```

The numbers below are card positions from left to right in the current hand, counted again after each play or discard.

| Blind | Actions | Expected result |
| --- | --- | --- |
| Small | Discard card 1. Play 4/5/6/7/8, then 1/5/6/7/8, then 3/5/6/7/8, then 2/3/4/7/8. | 349 / 300. Cash out, then click Next blind. |
| Big | Play 1/2/3/7/8, then 3/5/6/7/8, then 1/2/4/7/8, then 3/5/6/7/8. | 500 / 450. Cash out, buy the second offer named Joker, then click Next blind. |
| The Manacle | Play 1/2/3/5/6, then 3/4/5/6/7. | 672 / 600 and victory. |

To replay the loss, click Retry same seed and start Small. Check that Play hand is disabled without a selection. Select and play only the first card four times. Check the loss screen and retry once more.

`crates/ante-app/tests/campaign_route.rs` runs both scenarios without a window: cash out and purchase, displayed scores across transitions, deterministic observations across runs, rejection of empty play, and the same scores at every playback speed. It is skipped with `extra-jokers`, whose shop pool changes the route.

## Diagnostics and score playback

- Click **Speed** to change the playback speed. During playback, **Skip** reveals the committed result. The animation uses the same trace returned by the actual play action.
- The main menu opens the **Collection**. Purchases record discovery in `%LOCALAPPDATA%\Ante\profile.json`. A corrupt profile is preserved and the collection works in memory.
- **F3** toggles the frame profiler in its own window. **F2** clears its rolling ten-second history and **F4** exports the current samples and recent logs to `%LOCALAPPDATA%\Ante\logs` (`ante-report.json`, `ante-frames.csv`). Fatal failures also attempt to write `ante-crash.json`.
- The profiler separates application CPU timings, frame intervals and the most recent CPU and GPU render timings available from Bevy. CPU layout timings are wall time around the UI layout set. Render passes can overlap or nest and should be read individually. Trace timings include instrumentation overhead. [Bevy RenderDiagnosticsPlugin](https://docs.rs/bevy/0.20.1/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html)
- The graph plots up to ten seconds of frame time; each bar keeps the slowest frame in its bucket. The header FPS is the observed rate over the last second. The default is FIFO VSync. **F5** switches both windows to automatic no-VSync with a 1,000 updates/s ceiling; `--benchmark-fps` selects it at launch.

## Local textures and manual performance comparison

`--steam-textures` selects the default Windows Steam installation and `--assets "path"` another directory. Missing or invalid files fall back to Ante text, a plain background and the default font. No game resources are included in the repository.

```powershell
cargo run --release --locked -p ante-app --bin ante --features dev-tools
cargo run --release --locked -p ante-app --bin ante --features dev-tools -- --steam-textures --benchmark-fps
```

1. Use the first command for placeholder cards and the second for local textures. Check the four suits, face cards and a mapped Joker in the shop.
2. In each mode, run the seed 0 route once with F3 closed and once with F3 open. Press F2 just before the same scoring hand and F4 shortly after. Copy each CSV to a distinct name before the next export. Compare frame median, p95 and peak from the `frame` metric and the matching CPU and render measurements.
3. Confirm that closing the profiler leaves the main window open, and that the final scores are the same at every speed.

## Score cost (2026-11-09)

Measured before vendoring on an Intel Core i7-14700KF, median of nine batches after warm-up, with five Mult cards with Red Seals, synthetic retriggers and one Polychrome Joker. The benchmark was removed with the evaluation crate.

| Card activations in one hand | Windows, no trace | Windows, with trace | WSL, no trace | WSL, with trace |
| ---: | ---: | ---: | ---: | ---: |
| 10,010 | 9.91 µs | 135.03 µs | 11.86 µs | 31.43 µs |
| 50,010 | 46.71 µs | 1,290.79 µs | 65.91 µs | 157.90 µs |
| 100,010 | 95.06 µs | 2,690.40 µs | 111.58 µs | 357.09 µs |

A batch of 1,000 hands with ten ×2 Glass effects each took about 0.30 ms without the trace and 0.42 ms with it on Windows. A single hand with 10,000 successive doublings would exceed the engine's `usize` score capacity.

## Development iteration (2026-11-13)

On the development Windows machine, an incremental debug rebuild with `dev-tools` after editing a screen takes 13 to 19 s, mostly compiling `ante-app`.

- `rust-lld` gave the same times within noise, so it is not configured.
- Bevy `dynamic_linking` cannot link on Windows: the `bevy_dylib` DLL would export about 593 000 symbols, over the 65 535 limit of the format.
- Bevy hotpatching (`dx serve --hot-patch`) did not pick up saves on its own and was no faster than a restart.
- Restart on save with `cargo-watch`:

```powershell
cargo watch -w crates -w engine -x "run --locked -p ante-app --bin ante --features dev-tools"
```

## Action space and selection after a consumable

The fixed-size action space (`gen_action_space`) used to offer moves that the stricter consumable rules refused (The Fool picked from a pack without a free slot, a consumable used while a pack is open) and could stall in a Tarot target stage. Its consumable and pack masks now use the same `consumable_usable` and `pack_card_pickable` rules as `gen_actions`. As in the original `Card:use_consumeable`, the enhancement and suit Tarots, Strength, Death and the seal Spectrals now clear the hand selection after use, so a second Tarot picked from the same pack starts with no card selected; Aura, Cryptid and the other consumables keep it. `engine/balatro-rs/tests/action_space.rs` replays the eight seeds that used to fail; 1,500 seeded random games finish through both paths, and the upstream `test_game` passed 300 runs in a row.

## Behaviour coverage

| Area | Verified | Next compatibility checks |
| --- | --- | --- |
| Hand types | One example of each of 12 categories, one wild flush, basic invalid sizes | Additional combinations, scored subset and order under modifiers |
| Card effects | Lucky randomness, Hanging Chad repetitions, Mult and Glass stress cases | Combined editions and seals, effects on held and debuffed cards |
| Jokers | Base pool, purchase and collection persistence, opt-in Misprint, Golden Joker and Ice Cream | Remaining Joker hooks, copies and conflicting triggers |
| Scoring numbers | Integer overflow checks in Release builds | Representation and formatting beyond machine integers |
| Resources | Optional local atlas, font, shader and sound import with fallback, path and size checks | Remaining Joker coordinates, 2x atlases, other resources |
