# Architecture

`ante-core` contains portable card, sorting, hand evaluation and RNG foundations and has no Bevy dependency. `ante-app` owns presentation and consumes core data. `ante-cli` is a separate headless executable using the same core. Presentation must not control simulation outcomes. `ante-core::contract` defines the command, event, projection (`RunView`) and score-trace contract shared by the native slice and the experimental engine backend. Scoring lives in `ante-core::score`; local asset import and atlas resolution live in `ante-app::services::assets`.

RNG version 1 is SplitMix64 with wrapping integer arithmetic. Its state is private; save/restore and independent streams remain future work. This sequence is an Ante implementation, not a claim of Balatro seed compatibility.

An isolated comparison of candidate engines and its current limits is documented in [ENGINE_EVALUATION.md](ENGINE_EVALUATION.md). The production simulation still uses ante-core.
