# Ante — roadmap

## Jalons

1. **Parité fonctionnelle et composition statique.** Stabiliser le contrat de simulation, réutiliser les composants du client et terminer les parcours jouables. Le moteur vendoré dans `engine/` (mode Fast) alimente le client.
2. **Moteur intégré.** Étendre les règles du moteur vendoré (boutiques, packs, campagnes et effets) directement dans `engine/`, avec des tests sur les transitions, scores, traces et reprises. Mesurer la représentation des très grands scores avant d'étendre Endless. Auditer séparément le RNG et l'ordre des tirages avant d'engager la compatibilité des graines Balatro.
3. **Effets visuels.** Relever les animations, transitions, mouvements, parallaxes et shaders de fond ou d'édition. Concevoir leur adaptation Bevy et vérifier la provenance avant toute réutilisation.
4. **Polish 1:1.** Vérifier la fidélité par état et par format de fenêtre, puis corriger les écarts de rendu, de mouvement et de timing.

Le suivi détaillé des mécaniques et surfaces du wiki est dans [la roadmap des fonctionnalités](ROADMAP_FEATURES.md). Les règles de jeu retenues et leur état d'implémentation sont synthétisés dans [le dossier de logique](GAME_LOGIC_WIKI.md).

## Statuts

**Not implemented** surface absente. **MVP** parcours minimal. **In progress** surface jouable avec écarts identifiés. **Done** équivalence fonctionnelle et composition statique proche, comparée à la référence. **Done + VFX** effets de la référence reproduits. **Exactly 1:1** fidélité finale vérifiée.

## Interfaces

La recette commune compare une capture à taille cliente égale et vérifie le parcours décrit dans les [guides legacy](UI_LEGACY_GUIDE.md) et [Bevy](UI_BEVY_GUIDE.md).

| Surface | Statut | Écart ou recette suivante |
| --- | --- | --- |
| Menu principal | In progress | Fond `splash` d'origine sur toute la fenêtre (RED et BLUE), musique music1. Comparer logo, boutons et retours avec et sans atlas. Le flash d'ouverture (`mid_flash`) attend l'écran de démarrage. |
| Profils et records | MVP | Vérifier découvertes persistantes. Ajouter profils multiples et records. |
| Démarrage / graine | In progress | Comparer carrousel et saisie. Étendre les decks et stakes. |
| Pause | In progress | Comparer reprise, redémarrage et retour à la run. |
| Options de jeu | MVP | Vérifier la vitesse et inventorier les autres réglages. |
| Options vidéo | MVP | Vérifier VSync, résolution et modes d'affichage. |
| Options graphiques / ressources | In progress | Recetter import, remplacement et secours. |
| Options audio | In progress | Trois curseurs `create_slider` (Master, Music, Game) enregistrés au relâchement. Comparer taille et rendu des curseurs. |
| Collection / accueil | In progress | Disposition de `create_UIBox_your_collection` (groupe Consumables, compteurs). À comparer à taille égale. |
| Collection / jokers | In progress | 3 zones `title` de 5, cycle « Page n/N », centre non découvert et « ? ». Popups de partie (badge de rareté) dans toutes les catégories. Le « ? » ne se balance pas encore. |
| Collection / vouchers | In progress | 2 zones `voucher` de 4 (paires superposées). Comparer prérequis. |
| Collection / tags | In progress | Grille de 6 par rangée, tag « ? » pour les non découverts. |
| Collection / decks | In progress | Carrousel avec pastilles, nom et description sur blanc. La pile reste l'aperçu d'Ante. |
| Collection / tarots | In progress | Zones de 5 et 6, 2 pages. À comparer. |
| Collection / planètes | In progress | 6 + 6 de Mercure à Éris, sans cycle. À comparer. |
| Collection / spectraux | In progress | Zones de 4 et 5, 2 pages. À comparer. |
| Collection / boosters | In progress | 2 × 4 à 1,27 carte dans des zones de 1,3 carte. À comparer. |
| Collection / améliorations | In progress | 2 × 4. Le texte d'information est une formulation d'Ante. |
| Collection / éditions | In progress | Base puis 4 éditions sur le Joker, avec leur shader. |
| Collection / sceaux | In progress | 1 × 4 sur une carte vierge. |
| Collection / blinds | In progress | Grille de 5 dans l'ordre d'origine et table ANTE / BASE (antes 1 à 16). L'ante la plus haute n'est pas enregistrée : tous les montants restent gris. |
| Customize Deck | Not implemented | Choix des faces de cartes (`G.UIDEF.customize_deck`) chargées depuis l'import local, avec secours sur les faces par défaut. |
| Mods | Not implemented | À concevoir : API de mods d'Ante (données puis scripts), chargée avant la run et inscrite dans les journaux de run. Pas d'exécution directe des mods Steamodded. |
| Challenges | Not implemented | Inventorier conditions, sélection et sortie. |
| HUD et compteurs | In progress | Reconstruit d'après `create_UIBox_HUD` : boîtes intérieures, jeton de mise, Chips × Mult côte à côte, enseigne animée, couleurs dynamiques et coins pixelisés. À comparer à 1920 × 1056 ; l'ombre portée ne suit pas encore la parallaxe horizontale. |
| Choix des blinds | In progress | Les panneaux n'ont pas de popup, comme l'original ; le tag à côté de Skip Blind a le sien. Vérifier les trois états et l’objectif du boss. Un boss qui voudrait sa propre popup (modding) n'a pas encore de champ pour la fournir. |
| Tags détenus | In progress | Comparer pile, acquisition et disparition. |
| Main et actions | In progress | La rangée Play Hand / Sort / Discard reste en place quand la sélection change, et un bouton désactivé reste gris au survol. Comparer taille, sélection et recouvrement. Vérifier le glisser-déposer et le tri Rank/Suit. |
| Aperçu Chips × Mult | In progress | Vérifier niveau, sélection vide et lecture du score. |
| Résolution du score | In progress | Vérifier la trace et les interactions. Les effets animés viendront au jalon VFX. |
| Encaissement | In progress | Lignes et dollars d'`add_round_eval_row` au rythme d'origine, puis Cash Out. Le panneau ne monte pas encore depuis le bas. |
| Boutique / cartes | In progress | Haut à 3,587 unités, cadre `UIBox_dyn_container` jusqu'en bas, Buy & Use atomique. Choisir une offre ne reconstruit plus la boutique. Comparer offres variables et contrôles. |
| Boutique / vouchers | In progress | Recetter achats multiples, prix et prérequis. |
| Boutique / boosters | In progress | Comparer atlas, tailles, prix et ouverture. |
| Packs / Standard | In progress | Comparer choix de cartes et retour. |
| Packs / Buffoon | In progress | Comparer choix de jokers et contraintes de capacité. |
| Packs / Celestial | In progress | Vérifier le niveau obtenu et la fermeture. |
| Packs / Arcana | In progress | Vérifier la sélection des cibles et le retour. |
| Packs / Spectral | In progress | Vérifier la sélection des cibles et le retour. |
| Inventaire / jokers | In progress | Comparer disposition, panneau de vente et capacités. Vérifier l'ordre d'activation après dépôt. |
| Inventaire / consommables | In progress | Comparer le glisser-déposer, le panneau d'actions et les capacités. |
| Deck / Full Deck | In progress | Disposition de `G.UIDEF.view_deck` : boîte du deck, décomptes, colonne des rangs, une zone `title` par couleur. Le DynaText qui alterne base et effectif est remplacé par les deux nombres côte à côte. |
| Deck / Remaining | In progress | Les cartes déjà piochées restent affichées, grisées par le shader `played`. |
| Deck / grille au survol | In progress | Vérifier comptes, cartes masquées et valeurs effectives. |
| Infobulle / carte à jouer | In progress | Nom centré sur boîte blanche, chips et règle d'amélioration, badges et boîtes latérales d'édition et de sceau (`card_h_popup`). À comparer à taille égale : largeur fixe de 4,2 tuiles contre la largeur au contenu de l'original. |
| Infobulle / joker | In progress | Comparer rareté, état et valeurs dynamiques. Le « Currently » n'est calculé que pour les jokers observables; vérifier chaque valeur suivie et ne pas exposer l'état brut hors build Dev. |
| Infobulle / consommable | In progress | Comparer badges et valeurs. |
| Infobulle / voucher | In progress | Comparer description et états de découverte. |
| Infobulle / booster | In progress | Texte `p_<famille>_<taille>` local avec « choisir N parmi M ». Ouverture Arcana/Spectral : main visible, montée de la boîte et particules ; ordre choix puis cibles du moteur (écart documenté). |
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
