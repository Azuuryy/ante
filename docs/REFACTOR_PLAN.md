# Plan de refacto : engine intégré, tests séparés, fichiers découpés

Statut : proposé le 2026-11-13, non commencé. Ce document est temporaire : on le supprime à la phase 7, une fois que la doc permanente reflète la nouvelle structure.

## Décisions prises

| Sujet | Décision |
| --- | --- |
| Engine | Vendorer dans le repo les crates `balatro-rs` (core), `balatro-types` et `balatro-seed`, avec les 12 patches déjà appliqués. Plus de checkout dans `target/` ni de patches à rejouer. |
| CLI | Supprimer `crates/ante-cli`. |
| Moteur natif d'`ante-core` | Supprimer `native.rs`, `rng.rs`, `hand_eval.rs`, `score.rs`, `card.rs` et `standard_deck`. `ante-core` ne garde que le contrat et l'adaptateur vers l'engine. |
| Balatrust et `score_stress` | Supprimer la dépendance de dev Balatrust, les tests de comparaison, `ExperimentalEngine` (s'il ne sert qu'à eux) et l'exemple `score_stress`. |
| Emplacement des tests | Convention Rust standard : tests unitaires dans un `#[cfg(test)] mod tests` du module testé, tests d'intégration dans `crates/<crate>/tests/*.rs`. Si les tests font dépasser la limite de taille, on les sort dans un sous-module `tests.rs` voisin, toujours avec `#[cfg(test)] mod tests;`. |
| Taille des fichiers | Viser environ 400 lignes maximum par fichier source ou de test. On découpe par responsabilité, pas à la ligne près. |

### Convention de tests

On suit l'usage standard de Rust :

- **Tests unitaires** : dans le module testé, sous `#[cfg(test)] mod tests { use super::*; ... }`. Ils accèdent aux éléments privés et sont exclus du build normal.
- **Fichier trop long** : quand un module est devenu un dossier (`run/mod.rs`, ou `run.rs` accompagné de `run/`), ses tests vont dans `run/tests.rs`, déclaré par `#[cfg(test)] mod tests;`. On garde le nom `tests.rs`, sans `#[path]` ni arborescence parallèle. Ce découpage accompagne en général celui de la phase 5 : un fichier découpé par responsabilité donne aussi des tests plus courts.
- **Tests d'intégration** : directement dans `crates/<crate>/tests/*.rs`, contre l'API publique uniquement. Un code partagé entre plusieurs tests d'intégration va dans `tests/common/mod.rs`.

Ce qui existe déjà et respecte la convention reste en place. `app/tests.rs` et `app/layout_tests.rs` sont déjà des sous-modules séparés ; `layout_tests.rs` est renommé si le découpage de `app/` le demande.

## Structure cible

```
Cargo.toml                 workspace : crates/*, engine/*
crates/
  ante-core/               headless, sans Bevy : contrat + adaptateur engine
    src/contract/          ids, content, commands, views, trace, errors (contract.rs découpé)
    src/engine/            ex-experiments/.../mvp.rs, découpé :
      backend.rs           ExternalBackend (GameBackend)
      run/                 MvpRun : cycle de vie, commandes, shop, packs, consommables
      convert/             cartes, packs, vouchers, tags, traces de score
      describe/            textes des Jokers, consommables, règles, conditions de déblocage
    tests/*.rs             intégration (ex-mvp_flow si on y laisse playback, voir phase 2)
  ante-app/                client Bevy + binaire `ante` (main.rs = ante_app::run(ExternalBackend))
    src/{app,screens,components,services}/
engine/                    code vendoré, noms upstream conservés pour l'attribution
  balatro-rs/  balatro-types/  balatro-seed/
  LICENSE, NOTICE.md       origine, révision 3d66de9, liste des 12 patches, provenance non vérifiée
```

Disparaissent : `experiments/`, `crates/ante-cli`, `crates/ante-app/src/main.rs` actuel (le prototype `standard_deck`), le `#[path]` qui incluait `playback.rs` dans l'expérience, `target/engine-evaluation/` (local, hors Git).

## Phases

Chaque phase compile (`cargo check --workspace --all-targets`) avant de passer à la suivante. La suite complète et la revue se font en phase 7, comme d'habitude. On fait un commit par phase, chacun seulement avec ton accord.

### Phase 0 : point de départ propre

1. Committer le travail en cours (31 fichiers modifiés, 7 non suivis), avec ton accord.
2. Relever la base de référence : nombre de tests réussis dans le workspace et dans `experiments/engine-evaluation` (avec et sans `extra-jokers`), puis `cargo test -p balatro-rs` sur le checkout patché (517 attendus). La phase 7 compare à ces chiffres, moins les tests supprimés volontairement.

Base relevée le 2026-11-13 (commit `780fcec`) :

| Cible | Réussis (ignorés) |
| --- | --- |
| `ante-app` | 91 (3) |
| `ante-core` | 8 |
| expérience, `--features mvp` | lib 44, `mvp_flow` 4, `native_comparison` 1 |
| expérience, `--features mvp,extra-jokers` | lib 47, `mvp_flow` 2, `native_comparison` 1 |
| `balatro-rs` | 517, intégration 2 (2) |
| `balatro-seed` | 19 |
| `balatro-types` | 85, `canonical_counts` 1 |

### Phase 1 : vendorer l'engine

1. Copier l'arbre de travail patché de `core`, `balatro-types` et `balatro-seed` (patches indexés et non indexés compris) dans `engine/`.
2. Retirer ce qu'Ante n'utilise pas : features `python`/`pyo3` et `colored`, `core/src/bin/`, bench `benchmark`, `CLAUDE.md` upstream. Garder `ante-extra-jokers`, `serde` et `uuid`.
3. Ajouter `engine/LICENSE` (MIT tel que déclaré dans les manifests) et `engine/NOTICE.md` (dépôt d'origine, révision, liste ordonnée des 12 patches, mention de la provenance non vérifiée et de l'attribution à TheSoul de `balatro-seed`).
4. Intégrer les trois crates au workspace. Fusionner les deux `Cargo.lock` sans mettre à jour les versions (`rand ~0.8.5`, `rand_chacha 0.3` : le déterminisme en dépend).
5. Supprimer `experiments/engine-evaluation/patches/`, puisque le code patché devient la source.

### Phase 2 : fondre l'adaptateur dans `ante-core`

1. Déplacer `mvp.rs` dans `ante-core/src/engine/` en le découpant comme indiqué plus haut. `ante-core` dépend de `balatro-rs` et expose la feature `extra-jokers`.
2. Supprimer le moteur natif et ses tests. Les tests de l'app qui utilisaient `NativeBackend` (`app/tests.rs`, `layout_tests.rs`) passent sur `ExternalBackend`.
3. Supprimer Balatrust, les tests de comparaison, `score_stress`, ainsi que `ExperimentalEngine` si plus rien ne s'en sert. Ses deux tests utiles (snapshot déterministe, effets de Joker restaurés au chargement) sont réécrits sur `MvpRun`.
4. `mvp_flow.rs` utilise `ante_core` et `app::playback`. Il devient un test d'`ante-app` (sous-module de test de `app::playback`, ou test d'intégration si `playback` devient public). Cela supprime le `#[path]` hors crate.
5. Vérifier que le graphe de dépendances d'`ante-core` reste sans Bevy.

### Phase 3 : faire d'`ante-app` le binaire, supprimer CLI et expérience

1. `crates/ante-app/src/main.rs` lance `ante_app::run(ExternalBackend)`. Le binaire s'appelle `ante`. La feature `dev-tools` reste sur `ante-app`, et `extra-jokers` est relayée.
2. Supprimer `crates/ante-cli` et `experiments/`.
3. Mettre à jour `scripts/build-windows-client.ps1` (`ante.exe` / `ante-dev.exe` depuis le workspace), `scripts/check.sh` et `.github/workflows/headless.yml` (garde anti-Bevy sur `ante-core` + tests `ante-core`, `balatro-rs`, `balatro-types`, `balatro-seed`).
4. Renommer les identifiants `Mvp*` / `mvp` en noms définitifs (par exemple `EngineRun`), en un commit séparé et mécanique.

### Phase 4 : ranger les tests selon la convention

Les tests restent inline par défaut. Cette phase ne traite que les cas qui posent problème :

1. Quand la phase 5 découpe un module en dossier, ses tests partent dans `<module>/tests.rs`, ou sont répartis dans le fichier dont ils testent le comportement. Le `mod tests` de 600 lignes de `mvp.rs` (scoring, inventaire, campagne) en est un exemple.
2. Les tests d'intégration sont regroupés directement dans `tests/*.rs`, avec un `tests/common/mod.rs` pour les helpers partagés.
3. Côté engine, les fichiers de test ajoutés par les patches (`consumable_use_tests.rs`, `debug_tests.rs`, `trace_effect_tests.rs`, `consumable_room_tests.rs`) sont déjà des sous-modules séparés : ils suivent leur module si celui-ci est découpé.

Contrôle de non-perte : le nombre de tests par crate est identique avant et après cette phase.

### Phase 5 : découper les gros fichiers

Côté Ante, par ordre de taille :

| Fichier | Lignes | Découpe envisagée |
| --- | --- | --- |
| `screens/run.rs` | 2241 | `run/{board, blind_panel, hud, deck_select, inspect_deck, result, pause, overlay}.rs` |
| `components/tooltip.rs` | 1252 | `tooltip/{tip, state, panel, ink, words}.rs` |
| `screens/shop.rs` | 821 | par zone : offres, packs, vouchers, actions |
| `services/assets/catalog.rs` | 764 | par famille d'assets |
| `app/session.rs` | 755 | état de session, commandes, synchronisation de la vue |
| `components/widgets.rs` | 746 | un fichier par widget ou famille |
| `contract.rs` | 677 | traité en phase 2 |
| `components/travel.rs`, `tiles.rs`, `diagnostics/dev.rs` | 600 à 660 | selon les responsabilités constatées |

Ensuite viennent les fichiers vendorés `game.rs` (5332), `joker.rs` (4258), `hand.rs`, `generator.rs` et `rng.rs`. Ce sont les plus risqués, parce que l'ordre des tirages RNG ne doit pas bouger. On les découpe en dernier, uniquement par déplacement de code, sans modifier aucune logique.

### Phase 6 : audit et mise à jour de la doc

À relire et réécrire selon la nouvelle structure :

- `README.md` : build, lancement, plus de CLI ni de clone d'engine.
- `AGENTS.md` : retirer les règles sur l'expérience et le checkout épinglé, et noter la convention de tests standard (`mod tests` inline, `tests.rs` voisin si le module est long) et la limite de taille des fichiers.
- `docs/ARCHITECTURE.md` : crates, frontières, flux commande → trace → `RunView`.
- `docs/ENGINE_EVALUATION.md` : renommé `docs/ENGINE.md`. L'historique d'évaluation est condensé, et on décrit l'engine vendoré, ses patches et la façon de le maintenir.
- `docs/LEGAL.md` et `docs/COMPATIBILITY.md` : statut de l'engine vendoré et provenance.
- `docs/ROADMAP.md` et `docs/ROADMAP_FEATURES.md` : chemins et jalons.
- `docs/UI_BEVY_GUIDE.md`, `docs/UI_LEGACY_GUIDE.md`, `docs/CAPTURES.md`, `docs/GAME_LOGIC_WIKI.md` : chemins de fichiers et commandes.
- `.skills/ante-legacy-to-bevy/SKILL.md`, `.skills/ante-ui-parity/SKILL.md` : chemins.
- `CHANGELOG.md` : une entrée « Changed / Removed ».

Pour finir, on cherche dans tout le repo `experiments/`, `ante-cli`, `ante-mvp`, `engine-evaluation`, `native`, `Balatrust` et `target/engine-evaluation`. Il ne doit rester aucune référence obsolète.

### Phase 7 : vérification finale

1. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (avec et sans `extra-jokers`) et `cargo build --workspace`.
2. Comparer les nombres de tests à la base de la phase 0.
3. Lancer `scripts/build-windows-client.ps1` pour obtenir `ante.exe` et `ante-dev.exe`.
4. Faire une revue de code du diff complet.
5. Tu testes le client à la main. Je ne lance pas le client moi-même.
6. Supprimer ce document.

## Risques

- **Déterminisme** : la fusion des lockfiles ou le découpage de `game.rs` peut changer l'ordre des tirages. Les tests de route de `mvp_flow` (mêmes scores, mêmes transitions) servent de garde-fou. On ne change aucune version de `rand`.
- **Licence et provenance** : vendorer rend le code distribué avec Ante. Le MIT n'est déclaré que dans les manifests, et `balatro-seed` se présente comme un port. C'est consigné dans `NOTICE.md` et `LEGAL.md`. À trancher avant toute release publique, pas avant le refacto.
- **Taille du diff** : environ 25k lignes vendorées en plus et des centaines de fichiers déplacés. Le diff est découpé en un commit par phase pour rester relisible.
