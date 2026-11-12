# Roadmap des fonctionnalités

Cette roadmap suit les comportements détaillés, surface par surface. La roadmap macro reste dans [ROADMAP.md](ROADMAP.md); cette liste sert à découper les mécaniques du [dossier de logique](GAME_LOGIC_WIKI.md) en travail vérifiable.

## Statuts

- **À cartographier** : règles et cas limites à relever.
- **À faire** : absence confirmée dans le client ou le moteur ciblé.
- **En cours** : une partie est présente, la recette n'est pas terminée.
- **À vérifier** : code présent, comportement ou rendu non attesté.
- **Terminé** : critères et cas limites vérifiés.

## Main, cartes et score

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Sélection et désélection des cartes de la main | En cours | Vérifier souris, sélection maximale et maintien de sélection après actions. |
| Tri automatique par rang / couleur | En cours | Commandes présentes; vérifier l'ordre et la sélection conservée sur les deux moteurs. |
| Réordonnancement manuel de la main | En cours | La carte suit le point saisi et change de position au relâchement. Vérifier l'ordre de score et le comportement après tirage. |
| Détection et priorité des combinaisons | En cours | Comparer toutes les combinaisons et cartes améliorées à la référence. |
| Aperçu Chips × Mult et résolution | En cours | Valider chaque contribution et l'ordre des étapes dans la trace. |
| Améliorations et éditions | En cours | Couvrir effets, cumul, déclencheurs et interaction avec débuff. |
| Jokers à valeur évolutive | Fait pour les jokers relevés, à valider en jeu | Le patch moteur `joker-rules` aligne Green Joker, Castle, Obelisk, Yorick et Joker Stencil sur l'original, avec leur état porté par le Joker (`JokerState.counter`). Hit the Road et Steel Joker étaient déjà conformes. La ligne « Currently » lit cet état. Reste : les sauvegardes moteur antérieures n'ont pas ces compteurs (ils repartent de zéro). |

## Inventaires et ordonnancement

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Réordonnancement manuel des Jokers | En cours | Le dépôt modifie l'ordre de la simulation et réenregistre les effets. Vérifier les fenêtres de score et les copies. |
| Ordre des effets Joker | En cours | Vérifier les fenêtres d'activation, éditions, copies et répétitions avec traces. |
| Réordonnancement des consommables | En cours | Le dépôt déplace les instances. Vérifier les usages, les ventes et la reprise. |
| Vente et utilisation des consommables/Jokers | En cours | Vérifier prix, capacité, refus, révision et cas pendant les phases incompatibles. |
| Tags détenus et déclenchement | En cours | Vérifier ordre, suppression et empilement de tags de fin de manche. |

## Packs, boutique et économie

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Achat et relance de boutique | En cours | Prix, gratuité, capacité, offres supplémentaires et transitions. |
| Packs Standard / Buffoon / Arcana / Celestial / Spectral | En cours | Vérifier contenus, quantité de choix, limites et retour à la phase appelante. |
| Description des cartes d'un pack | À vérifier | Le code attache déjà `item.description`; vérifier qu'aucun état de sélection ne la remplace par une consigne générique. |
| Ciblage depuis un pack | En cours | Cibles légales, application unique, annulation et Skip. |
| Vouchers et prérequis | En cours | Éligibilité, remise, effets immédiats et avantages persistants. |
| Récompense, cash out et intérêts | En cours | Valider calcul détaillé, absence de double paiement et limites. |

## Cartes à jouer : état et effets

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Sceau Or | À vérifier | Vérifier attribution, gain lors du score et répétitions. |
| Sceau Rouge | À vérifier | Vérifier attribution et répétition de l'activation, avec autres sources de répétition. |
| Sceau Bleu | À vérifier | Vérifier création conditionnelle de Planète en fin de manche et capacité. |
| Sceau Violet | À vérifier | Vérifier Tarot lors d'une défausse et espace consommable. |
| Attribution des sceaux | En cours | Tester cartes Spectrales, effets de cartes et remplacement/exclusivité du sceau. |
| Débuff moteur | En cours | Comparer les règles par blind et Joker; vérifier quelles contributions sont annulées. |
| Débuff visuel | En cours | Main et cartes du Deck affichent une croix rouge et une teinte rouge; comparer opacité et étendre aux autres surfaces si elles peuvent porter l'état. |
| Sceaux visuels | En cours | La main et le Deck affichent un seul sceau à gauche, sous le coin du rang. Vérifier la lisibilité en jeu. |

## Mouvement, survol et infobulles

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Hover/élévation des cartes en main | En cours | Composant `HoverMotion` : échelle +0,05 sur la base 0,95 lissée par `exp(-60·dt)`, position par `exp(-50·dt)`, juice au survol (0,05 / 0,03) sur 0,4 s, sans lift vertical au survol comme l'original. Une carte tenue par le drag garde sa transformation. Sélection : lift de 0,2 × hauteur de carte. À comparer en jeu avec `run_hand-1-selected`. Inclinaison selon le curseur : à faire. |
| Hover des Jokers et consommables détenus | En cours | Mouvement partagé sur cartes et zone d'actions; vérifier sans masquer les actions. |
| Hover des cartes de packs et offres | En cours | Mouvement partagé sur les offres et cartes choisies; vérifier chaque type. |
| Hover dans Full Deck et Remaining | En cours | Mouvement et tooltip présents; vérifier ordre de pioche non révélé et hitbox après zoom. |
| Infobulles contextualisées | En cours | Ancrage d'origine : au-dessus de la carte, en dessous si son haut est à moins de 0,8 hauteur de carte du haut de la salle, à gauche pour les offres de boutique et la vue Deck, puis bornage à la fenêtre. Ligne « (Currently …) » structurée (`CurrentValue`) pour 17 jokers, colorée comme l'original. Vue Deck : seuls les attributs présents. Encarts latéraux (`info_queue`) à gauche du popup, centrés sur lui : références de l'effet (Marble Joker → Stone Card, The Chariot → Steel Card, Wheel of Fortune → éditions, sceaux des Spectrales, éditions des tags), puis édition, sceau et stickers. Textes rédigés par Ante. Reste : références des Jokers non couverts (Diet Cola, Perkeo…), boosters des tags, Planète citée par le Blue Seal. |
| Animation de sélection | À vérifier | Appliquer uniformément aux catégories énumérées et garder les contrôles accessibles. Aucun tween pour l'instant. |
| Panneau debug des infobulles | Fait, à valider en jeu | Build Dev uniquement : panneau gris clair séparé sous l'infobulle (id et instance, état moteur, valeur courante brute, prix et revente, édition et stickers, coordonnées d'atlas). Couvre jokers, consommables, cartes de main, offres, packs et vue Deck. Dos de deck sur le plateau : pas d'infobulle (aperçu de pioche). |
| Outils debug en jeu | Fait, à valider en jeu | Build Dev : run à 1000 $, panneau Tab (+10 $, +100 $, +1 main, +1 défausse, +1 manche, +1 ante, +100 score, gagner le blind, perdre la run) via `Command::Debug`. La run est marquée `debug`. F6 enregistre une capture dans `%LOCALAPPDATA%\Ante\captures\`. |

## Collection et progression

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Collection jokers, vouchers, tags | En cours | Pagination et découverte présentes; les objets à condition de déblocage affichent un cadenas, un tooltip « Locked » et leur condition. Vérifier chaque condition et le rendu du cadenas à taille réelle. |
| Collection cartes, boosters, améliorations, éditions, sceaux, blinds | Fait, à comparer | Voir « Autres catégories ». |
| Déblocages et carrière | En cours | Profil JSON v2 : `version`, `discovered`, `all_unlocked`, avec migration v1. Le bouton debug « Unlock all » est dans Settings > GAME (`dev-tools`). La découverte sert encore de substitut au déblocage réel; compter les statistiques de carrière reste à faire. |
| Decks, stakes et challenges | À cartographier | Les 15 decks ont un identifiant et une condition de déblocage affichée en Collection. Les règles de stake, la sélection verrouillée et les challenges ne sont pas branchés. |

## Consommables, ciblage et conditions

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Règle de disponibilité unique | Fait | Une seule fonction (`consumable_usable`) décide du Use, pour le générateur comme pour l'application. La vérification a lieu avant le retrait de la carte. En Shop, les Spectrales qui touchent la main restent bloquées (main vide). |
| Planètes et Tarots sans cible | En cours | Utilisables en PreBlind, Blind, PostBlind et Shop; bloque pendant un pack ouvert. Couvert par `consumable_use_availability_follows_targets_and_stage`. |
| Tarots à cible (1 à 3 cartes) | En cours | Utilisables seulement pendant un Blind et quand le nombre de cartes sélectionnées est dans `min_targets..=max_targets`. Le bouton « Use » n'apparaît qu'une fois la sélection valide; la référence ouvre d'abord un mode de ciblage. |
| Spectres sans cible | En cours | Même fenêtre que les Planètes (hors pack). |
| Spectres qui touchent la main | Fait, à valider en jeu | Familiar, Grim, Incantation, Sigil, Ouija et Immolate sont utilisables en Blind dès que la main a plus d'une carte, sans sélection, comme dans la référence. Ils restent refusés hors Blind, où la main est vide. |
| Ciblage depuis un pack | En cours | `PickPack` entre dans `TarotHand`/`SpectralHand`, puis `ApplyTarot`/`ApplySpectral`; vérifier annulation, Skip et retour au pack. |
| « Use » manquant et cartes perdues | Corrigé, à valider en jeu | Cause : règles de disponibilité dupliquées et vérifiées après retrait de la carte. Familiar, Grim, Incantation, Immolate, Sigil et Ouija deviennent utilisables en Blind dès que la main a plus d'une carte, sans tenir compte de la sélection. Judgement, Wraith et Soul exigent une place de joker ; Ankh, Hex, Ectoplasm et Wheel of Fortune exigent un joker ; The Fool exige un dernier consommable utilisé différent de lui-même ; Aura exige une carte sans édition. Hex et Ectoplasm ne ciblent que les jokers sans édition. Patch `balatro-rs-consumable-use.patch`, 16 tests dédiés (`consumable_use_tests.rs`). |
| Description et état des consommables | En cours | Description, catégorie et champs techniques dev-tools présents; vérifier les textes de chaque Tarot/Spectre. |

## Profil, déblocages et collection

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Format du profil | En cours | `profile.json` version 2, lecture de la version 1, écriture atomique. Champs : `discovered` (ensemble trié) et `all_unlocked` (booléen). |
| Unlock All debug | Fait | Couvre toutes les catégories et reste disponible après un premier déblocage, pour les catégories ajoutées ensuite. |
| Verrou de collection | En cours | `Content.unlock` porte la condition; les entrées verrouillées affichent un cadenas, un tooltip « Locked » et la condition. Les Jokers légendaires indiquent le pack Spectral. |
| Déblocages par achievement | À faire | Les conditions sont descriptives et ne sont pas évaluées. Il manque un ensemble `unlocked` distinct de `discovered`, des statistiques de carrière et les contrôles de condition (Royal Flush défaussé, 5 Jacks, etc.). |
| Compteurs de collection | Fait | Chaque bouton du hub affiche découverts / total. Le pourcentage global n'est pas affiché. |
| Autres catégories | Fait, à comparer | Tarot, Planète, Spectrale, cartes améliorées, sceaux, éditions, boosters et blinds sont actifs, avec leurs visuels d'origine. Les consommables, packs et boss rencontrés sont découverts en jeu ; améliorations, éditions et sceaux seulement par Unlock All pour l'instant. Défaut connu : en 1920×1057, la flèche « > » des pages manque. |

## Shaders et VFX

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Stratégie | Décidée | Les shaders d'origine sont réutilisés depuis l'installation de l'utilisateur, jamais commités ni livrés. L'import Local Assets copie les `.fs` autorisés dans `%LOCALAPPDATA%\Ante\local-assets\shaders\`. Au chargement, un prélude de compatibilité LÖVE écrit par Ante (`extern`, `Image`, `Texel`, `effect` → `main`) est ajouté, puis naga convertit le GLSL en WGSL pour un `UiMaterial`. Repli sans effet si le fichier manque ou si la conversion échoue. |
| Chargement des `.fs` | Fait, rendu à valider | Les 19 fichiers de `resources/shaders` sont du GLSL LÖVE 11 (`effect`/`position`, `extern`, `number`, `Image`, `Texel`, `love_ScreenSize`). Bevy 0.20 charge `wgsl` ; la conversion passe par le frontend GLSL de naga après le prélude. 17 shaders sur 19 convertis et validés par naga sur l'installation locale ; `skew` et `vortex` sont vertex seulement. |
| Uniforms et ordre de dessin | Fait | Relevés dans le [guide legacy](UI_LEGACY_GUIDE.md#shaders) : `time`, `dissolve`, `texture_details`, `image_details`, `burn_colour_*`, `shadow`, paramètre nommé comme le shader, et ordre `Card:draw` / `Sprite:draw_shader`. |
| Éditions Foil / Holo / Poly / Negative | En cours, rendu à valider | Shaders d'origine en surcouche sur les Jokers (détenus, boutique) et les cartes jouables (main, zone de jeu, vue Deck), sur la base et la face comme `Card:draw`. Reste : `negative` qui remplace la base au lieu de la recouvrir, inclinaison et juice dans le paramètre du shader, consommables. |
| Dissolve, flame, played, debuff, booster, voucher | En cours | `dissolve` : un Joker ou consommable qui quitte l'inventaire brûle en 0,7 s comme `Card:start_dissolve` (or à la vente, sinon noir et orange), avec particules et sons ; vérifié en jeu. `debuff` sur les cartes débuffées (repli croix rouge sans shader), `booster` et `voucher` en boutique et dans Run Info. Reste : `played`, `flame`, dissolution des cartes détruites en main. |
| Skew / vortex / CRT / splash / background | À faire | Même pipeline. Les warps de sommet supposent des coordonnées écran LÖVE à reproduire ; le CRT est un post-process multi-passes. |
| Effets hors shader | En cours | Secousse de la salle d'origine (`update_canvas_juice` : +0,7 par popup, amortie en (1 − 5·dt), dérive lente et attrait du curseur), particules de dissolution. La légère rotation permanente de la salle est omise : avec des bords pixel nets elle crée une marche qui rampe. L'inclinaison en perspective au survol demande une déformation de sommets que l'interface Bevy ne permet pas pour l'instant. |

## Animations et audio

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Lecture du score | En cours | Timeline en secondes de jeu avec les délais d'origine (entrée 0,4 + 0,1/carte + 0,2 + 0,1/carte + 0,4 ; popups chips 0,75, mult 0,8125, joker 0,9375, retrigger en plus ; fin 0,1/carte + 0,4 + 0,8 + 0,5 + 0,3 ; défausse 0,1/carte). Vitesses x0,5 à x128 par doublement, accélération automatique d'origine (+0,2·vitesse/s, plafond 16, effet au-delà de 2), vitesse enregistrée dans le profil. Instant retiré, Skip conservé. Chaque pas affiche ses popups l'un après l'autre comme `card_eval_status_text` : « Again! », chips, +Mult, XMult (« X1.5 Mult ») puis argent (« $3 »), avec juice de la source, au-dessus des cartes et sous les Jokers. Le patch moteur `trace-effects` note le facteur XMult et l'argent de chaque pas. Le Mult est décimal comme dans l'original (patch `fractional-mult`) et s'affiche avec les règles de `number_format`. Reste : chronométrer contre l'original. |
| Survol | En cours | Lissage d'origine en place (voir Hover) et éventail de la main (`CardArea:align_cards` : inclinaison 0,2 × (k − n/2 − 0,5) / n, chute vers les bords, balancement lent). Reste : inclinaison selon le curseur (0,3) et inclinaison ambiante (0,2). |
| Déplacement des cartes | En cours, à comparer | Zone de jeu 3,6 unités au-dessus de la main. Trajet animé avec le modèle de vitesse de `Moveable:move_xy` (gain 35, lissage `exp(-50·dt)`, plafond 70 unités/s) : pioche depuis le deck, main → zone de jeu, puis zone de jeu → défausse hors écran à droite, une carte toutes les 0,1 s de jeu. Une carte qui attend sa pioche reste cachée dans le deck. Sélection, tri et dépôt d'un drag glissent aussi. Les cartes défaussées avec le bouton Discard partent une à une vers la défausse, puis la pioche commence, comme `discard_cards_from_highlighted` suivi de `draw_from_deck_to_hand`. La main ne s'étale que sur les cartes arrivées : elle se resserre pendant une défausse puis s'élargit à chaque carte piochée. Une carte piochée quitte le deck face cachée et se retourne en entrant dans la main (pincement de `Moveable:move_wh`, 0,125 s pour se fermer puis 0,125 s pour se rouvrir). L'éventail suit le nombre de cartes présentes. Reste : son `card1`, flip des cartes retournées par les boss. |
| Juice des scores | En cours | Popups, juice de la source, impulsion des valeurs Chips et Mult et secousse de la salle en place. Reste : particules et flammes des gros scores. |
| VFX d'édition et de destruction | En cours | Éditions et dissolution : voir la section Shaders et VFX. Les shaders LÖVE calculent en espace gamma ; Ante convertit l'entrée et la sortie, et les calques d'édition sont dessinés après le sprite. |
| Audio | En cours | Effets sonores d'origine lus depuis l'installation locale (28 fichiers `.ogg` de la liste autorisée, copiés par l'import Local Assets) : survol, sélection, départ des cartes, popups de score, total, achats et ventes, boutons, dissolution. Volume `vol × général/100 × sons/100`, réglages général (50) et sons (100) dans Options > AUDIO, enregistrés dans le profil. Reste : musique et ambiances, variation de hauteur `pitch_mod`. |

## Interfaces, sauvegarde et compatibilité

| Fonctionnalité | État actuel | Travail / critère de sortie |
| --- | --- | --- |
| Modaux Run Info, Deck et Pause | En cours | Vérifier retour à la phase et conservation des sélections. |
| Sauvegarde / reprise | À vérifier | Comparer snapshot/reprise sur pack, boutique, trace et ordre manuel. |
| RNG et graines | À cartographier | Documenter flux par système avant toute promesse de compatibilité de graine. |
| Victoire, défaite et Endless | En cours | Vérifier transitions, ante 8 et représentation des scores élevés. |
| Comparaison visuelle multi-format | À vérifier | Captures de référence et client à taille égale; documenter les écarts et VFX séparément. |

Mettre à jour les états après chaque tranche de travail avec le chemin du code ou de la recette qui justifie l'avancement. Ne passer à **Terminé** qu'après vérification fonctionnelle et, pour les surfaces visuelles, comparaison de captures.


