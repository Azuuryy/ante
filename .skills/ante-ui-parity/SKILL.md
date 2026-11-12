---
name: ante-ui-parity
description: Comparer et ajuster les écrans Bevy d'Ante à une capture de Balatro dans le même état et à la même résolution.
---

Utiliser la capture utilisateur ou la référence locale correspondant à la phase, rangée selon [la bibliothèque de captures](../../docs/CAPTURES.md) (`captures/reference/`, hors Git). Capturer Ante avec F6 dans le build Dev, au même format. Lire les conventions de [mise en page et recette](../../docs/UI_BEVY_GUIDE.md). Relever d'abord limites de la zone cliente, ancrages, proportions des régions, espacements, palette et états des contrôles. Garder la mise à l'échelle uniforme et tester les hitboxes après redimensionnement.

Composer à partir des composants Ante. Charger les sprites locaux disponibles et vérifier leur rendu de secours. L'exigence utilisateur prime sur la référence, notamment l'assombrissement sans mouvement des boutons. Comparer côte à côte à dimensions égales, puis vérifier un format large et un format étroit. Reporter les écarts observés et préciser les états qui restent à comparer. Une compilation valide le code. La comparaison visuelle demande une capture du rendu.
