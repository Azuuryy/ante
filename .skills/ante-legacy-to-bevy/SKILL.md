---
name: ante-legacy-to-bevy
description: Convertir un comportement observé dans Balatro en écran ou interaction Bevy d'Ante, avec traçabilité vers les états et actions du moteur.
---

Lire la section pertinente du [guide legacy](../../docs/UI_LEGACY_GUIDE.md), puis son équivalent dans le [guide Bevy](../../docs/UI_BEVY_GUIDE.md). Chercher d'abord le point d'entrée et le callback dans l'installation locale, en lecture seule.

Pour l'interaction concernée, relever conditions, données visibles, entrée, action, effets et retour. Déterminer si l'état appartient au moteur ou à la navigation. Faire passer les actions de jeu par `Command` et exposer leurs résultats par `RunView`. Un tri de présentation, un modal et le score animé lisent les projections sans modifier la simulation. Réutiliser les couches du plateau et les contrôles existants. Décrire les écarts et actualiser la recette du guide Bevy après vérification.

Reprendre fidèlement la logique, les formules, les constantes et les séquences d'origine, adaptées à Rust et à l'ECS. Ne jamais commiter les fichiers d'origine (Lua, images, sons, polices, shaders `.fs`, textes) : les charger au runtime depuis l'installation locale via l'import autorisé, avec un secours explicite. Le moteur vendoré dans `engine/` se modifie directement, avec des tests, et `engine/NOTICE.md` reste à jour.
