# Bibliothèque de captures

Les captures servent à reproduire l'interface 1:1 : placement, tailles, couleurs et états des composants. Les captures du jeu d'origine sont des images du jeu. Elles restent hors Git (`/captures/` est ignoré) et ne sont jamais livrées.

## Dossiers

| Dossier | Contenu |
|---|---|
| `captures/reference/` | Captures du jeu d'origine, prises par l'utilisateur |
| `captures/ante/` | Captures d'Ante au même état, copiées depuis `%LOCALAPPDATA%\Ante\captures\` |

Dans le build Dev, **F6** enregistre la fenêtre dans `%LOCALAPPDATA%\Ante\captures\` sous le nom `<surface>_<horodatage>_<L>x<H>.png`. La surface est la phase courante (`blind-select`, `playing`, `shop`, etc.). Renommer ensuite avec l'état comparé.

## Règles de prise

- Fenêtré, zone cliente 1920×1080, même résolution pour les deux jeux. Un format large (2560×1080) et un étroit (1280×1024) en second temps.
- Vitesse de jeu 1x, effets d'écran (CRT) au réglage par défaut, textures d'origine dans Ante (`--steam-textures`).
- Pour une animation, capturer l'instant indiqué. Une courte vidéo à 60 i/s (OBS, Xbox Game Bar) est utile pour mesurer les durées : nommer `<surface>_<etat>_<L>x<H>.mp4`.

## Nommage

`<surface>_<etat>_<L>x<H>.png`, en minuscules, mots séparés par des tirets. Exemple : `run_hand-5-selected_1920x1080.png`.

## Checklist

### Menus et run

| Fichier | État |
|---|---|
| `menu_main` | Menu principal |
| `menu_profile` | Choix du profil |
| `run-setup_deck` | Nouvelle run, choix du deck |
| `blind-select_small` | Choix du blind, Small sélectionnable |
| `blind-select_boss` | Choix du blind, Boss sélectionnable |

### Plateau

| Fichier | État |
|---|---|
| `run_hand-idle` | Main de 8 cartes, rien de sélectionné, 2 jokers et 1 consommable |
| `run_hand-1-selected` | Une carte sélectionnée |
| `run_hand-5-selected` | Cinq cartes sélectionnées |
| `run_play-moved` | Cartes jouées arrivées dans la zone de jeu, avant le premier popup |
| `run_play-first-card` | Popup de la première carte qui score |
| `run_play-joker` | Popup d'un joker pendant le score |
| `run_play-total` | Total chips × mult affiché, avant la défausse |
| `run_deck-hover` | Survol du deck |

### Infobulles

| Fichier | État |
|---|---|
| `tip_joker-held` | Joker possédé, avec une ligne « Currently » (Green Joker après quelques mains) |
| `tip_joker-shop` | Joker en boutique |
| `tip_consumable` | Tarot ou planète possédé |
| `tip_playing-card` | Carte de la main avec amélioration ou sceau |
| `tip_voucher` | Voucher en boutique |
| `tip_tag` | Tag au choix du blind |
| `tip_pack` | Booster en boutique, puis une carte dans le pack ouvert |
| `tip_edition` | Joker foil, holo ou polychrome avec le tooltip latéral |

### Boutique et fin de manche

| Fichier | État |
|---|---|
| `cash-out` | Écran de gains |
| `shop_idle` | Boutique au repos |
| `shop_pack-open` | Pack ouvert |
| `run-info` | Run Info, onglet mains |
| `collection_jokers` | Collection, page des jokers |
| `options_game` | Options, onglet jeu |
| `end_win` | Victoire |
| `end_lose` | Défaite |

### Effets visuels

| Fichier | État |
|---|---|
| `vfx_foil`, `vfx_holo`, `vfx_polychrome`, `vfx_negative` | Un joker de chaque édition, au repos et au survol |
| `vfx_dissolve` | Vente d'un joker, milieu de l'effet |
| `vfx_debuff` | Carte débuffée par un boss |

## Après comparaison

Reporter les écarts et le statut dans la matrice de [la roadmap](ROADMAP.md). Les mesures (positions, tailles) se notent en fractions de la zone cliente, comme dans [le guide Bevy](UI_BEVY_GUIDE.md).
