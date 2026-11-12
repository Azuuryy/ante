# Ante — roadmap

## Jalons

1. **Parité fonctionnelle et composition statique.** Stabiliser le contrat de simulation, réutiliser les composants du client et terminer les parcours jouables. Le moteur Fast épinglé alimente le client pendant le développement.
2. **Moteur natif en parallèle.** La tranche d'une blind passe par le CLI. Étendre ensuite aux boutiques, packs, campagnes et effets. Basculer le client après validation des transitions, scores, traces et reprises. Mesurer la représentation des très grands scores avant d'étendre Endless. Auditer séparément le RNG et l'ordre des tirages avant d'engager la compatibilité des graines Balatro.
3. **Effets visuels.** Relever les animations, transitions, mouvements, parallaxes et shaders de fond ou d'édition. Concevoir leur adaptation Bevy et vérifier la provenance avant toute réutilisation.
4. **Polish 1:1.** Vérifier la fidélité par état et par format de fenêtre, puis corriger les écarts de rendu, de mouvement et de timing.

Le suivi détaillé des mécaniques et surfaces du wiki est dans [la roadmap des fonctionnalités](ROADMAP_FEATURES.md). Les règles de jeu retenues et leur état d'implémentation sont synthétisés dans [le dossier de logique](GAME_LOGIC_WIKI.md).

## Statuts

**Not implemented** surface absente. **MVP** parcours minimal. **In progress** surface jouable avec écarts identifiés. **Done** équivalence fonctionnelle et composition statique proche, comparée à la référence. **Done + VFX** effets de la référence reproduits. **Exactly 1:1** fidélité finale vérifiée.

## Interfaces

La recette commune compare une capture à taille cliente égale et vérifie le parcours décrit dans les [guides legacy](UI_LEGACY_GUIDE.md) et [Bevy](UI_BEVY_GUIDE.md).

| Surface | Statut | Écart ou recette suivante |
| --- | --- | --- |
| Menu principal | In progress | Comparer logo, boutons et retours avec et sans atlas. |
| Profils et records | MVP | Vérifier découvertes persistantes. Ajouter profils multiples et records. |
| Démarrage / graine | In progress | Comparer carrousel et saisie. Étendre les decks et stakes. |
| Pause | In progress | Comparer reprise, redémarrage et retour à la run. |
| Options de jeu | MVP | Vérifier la vitesse et inventorier les autres réglages. |
| Options vidéo | MVP | Vérifier VSync, résolution et modes d'affichage. |
| Options graphiques / ressources | In progress | Recetter import, remplacement et secours. |
| Options audio | Not implemented | Cartographier les contrôles et la persistance. |
| Collection / accueil | In progress | Comparer navigation et catégories. |
| Collection / jokers | In progress | Comparer pages, découverte et sprite de base. |
| Collection / vouchers | In progress | Comparer pages, découverte et prérequis. |
| Collection / tags | In progress | Comparer pages et infobulles. |
| Collection / decks | MVP | Étendre la couverture et vérifier la sélection. |
| Collection / tarots | Not implemented | Brancher le catalogue. |
| Collection / planètes | Not implemented | Brancher le catalogue. |
| Collection / spectraux | Not implemented | Brancher le catalogue. |
| Collection / boosters | Not implemented | Brancher le catalogue. |
| Collection / améliorations | Not implemented | Brancher le catalogue. |
| Collection / éditions | Not implemented | Brancher le catalogue. |
| Collection / sceaux | Not implemented | Brancher le catalogue. |
| Collection / blinds | Not implemented | Brancher le catalogue. |
| Challenges | Not implemented | Inventorier conditions, sélection et sortie. |
| HUD et compteurs | In progress | Comparer ancres, compteurs et enseigne. |
| Choix des blinds | In progress | Vérifier les trois états et l’objectif du boss. |
| Tags détenus | In progress | Comparer pile, acquisition et disparition. |
| Main et actions | In progress | Comparer taille, sélection et recouvrement. Vérifier le glisser-déposer et le tri Rank/Suit. |
| Aperçu Chips × Mult | In progress | Vérifier niveau, sélection vide et lecture du score. |
| Résolution du score | In progress | Vérifier la trace et les interactions. Les effets animés viendront au jalon VFX. |
| Encaissement | MVP | Ajouter les lignes de détail de la récompense. |
| Boutique / cartes | In progress | Comparer offres variables et contrôles. Ajouter l'action atomique Buy & Use. |
| Boutique / vouchers | In progress | Recetter achats multiples, prix et prérequis. |
| Boutique / boosters | In progress | Comparer atlas, tailles, prix et ouverture. |
| Packs / Standard | In progress | Comparer choix de cartes et retour. |
| Packs / Buffoon | In progress | Comparer choix de jokers et contraintes de capacité. |
| Packs / Celestial | In progress | Vérifier le niveau obtenu et la fermeture. |
| Packs / Arcana | In progress | Vérifier la sélection des cibles et le retour. |
| Packs / Spectral | In progress | Vérifier la sélection des cibles et le retour. |
| Inventaire / jokers | In progress | Comparer disposition, panneau de vente et capacités. Vérifier l'ordre d'activation après dépôt. |
| Inventaire / consommables | In progress | Comparer le glisser-déposer, le panneau d'actions et les capacités. |
| Deck / Full Deck | In progress | Comparer groupes et compteurs. |
| Deck / Remaining | In progress | Vérifier la pioche restante et le retour à la sélection. |
| Deck / grille au survol | In progress | Vérifier comptes, cartes masquées et valeurs effectives. |
| Infobulle / carte à jouer | In progress | Les choix de packs exposent déjà leur description; vérifier la présentation et comparer badges, bords et débuffs. |
| Infobulle / joker | In progress | Comparer rareté, état et valeurs dynamiques. Le « Currently » n'est calculé que pour les jokers observables; vérifier chaque valeur suivie et ne pas exposer l'état brut hors build Dev. |
| Infobulle / consommable | In progress | Comparer badges et valeurs. |
| Infobulle / voucher | In progress | Comparer description et états de découverte. |
| Infobulle / booster | In progress | Vérifier taille et nombre de choix; le choix de pack utilise désormais la même échelle de carte que la boutique. |
| Infobulle / tag | In progress | Vérifier effet et cible Orbital. |
| Infobulle / blind | MVP | Vérifier les données et conditions de focus. |
| Run Info / Poker Hands | In progress | Comparer niveaux et nombres de mains. |
| Run Info / Blinds | In progress | Comparer ordre et objectifs. |
| Run Info / Vouchers | In progress | Vérifier l'état détenu et la navigation. |
| Victoire, défaite et Endless | In progress | Comparer les trois résultats et la limite numérique. |
| Survol et sélection des cartes | In progress | Le zoom au survol (+5 %) est partagé par la main, la boutique, les packs, les inventaires et les vues Deck. Reste : lissage `exp(-60·dt)`, juice, soulèvement de sélection de 0,2·hauteur de carte et inclinaison selon le curseur. |
| Débuff visuel | MVP | Croix rouge et teinte rouge en place sur la main, le deck et les packs. Reste : comparer avec la capture `vfx_debuff` et porter le shader de débuff d'origine. |
| Sceaux | In progress | Le moteur expérimental projette les quatre sceaux et leurs descriptions. Recetter et compléter attribution, effets de scoring/défausse/fin de manche et rendu. |

## Outils Dev

Profiler, logs structurés, export et détails de survol sont compilés avec `dev-tools`. Recetter les raccourcis dans Dev et vérifier l'absence du profiler, des logs Ante et des exports dans Prod.
