# Capture library

Captures are used to reproduce the interface 1:1: placement, sizes, colours and component states. Captures of the original game are images of the game. They stay out of Git (`/captures/` is ignored) and are never shipped.

## Folders

| Folder | Content |
|---|---|
| `captures/reference/` | Captures of the original game, taken by the user |
| `captures/ante/` | Captures of Ante in the same state, copied from `%LOCALAPPDATA%\Ante\captures\` |

In the Dev build, **F6** saves the window to `%LOCALAPPDATA%\Ante\captures\` as `<surface>_<timestamp>_<W>x<H>.png`. The surface is the current phase (`blind-select`, `playing`, `shop`, etc.). Then rename it after the state being compared.

## Capture rules

- Windowed, 1920×1080 client area, the same resolution for both games. A wide (2560×1080) and a narrow (1280×1024) size come second.
- Game speed 1x, screen effects (CRT) at their default setting, original textures in Ante (`--steam-textures`).
- For an animation, capture the stated moment. A short 60 fps video (OBS, Xbox Game Bar) helps measure durations: name it `<surface>_<state>_<W>x<H>.mp4`.

## Naming

`<surface>_<state>_<W>x<H>.png`, lower case, words separated by hyphens. Example: `run_hand-5-selected_1920x1080.png`.

## Checklist

### Menus and run

| File | State |
|---|---|
| `menu_main` | Main menu |
| `menu_profile` | Profile choice |
| `run-setup_deck` | New run, deck choice |
| `blind-select_small` | Blind choice, Small selectable |
| `blind-select_boss` | Blind choice, Boss selectable |

### Board

| File | State |
|---|---|
| `run_hand-idle` | Hand of 8 cards, nothing selected, 2 jokers and 1 consumable |
| `run_hand-1-selected` | One card selected |
| `run_hand-5-selected` | Five cards selected |
| `run_play-moved` | Played cards arrived in the play area, before the first popup |
| `run_play-first-card` | Popup of the first scoring card |
| `run_play-joker` | Popup of a joker during scoring |
| `run_play-total` | Chips × mult total shown, before the discard |
| `run_deck-hover` | Deck hover |

### Tooltips

| File | State |
|---|---|
| `tip_joker-held` | Held joker with a "Currently" line (Green Joker after a few hands) |
| `tip_joker-shop` | Joker in the shop |
| `tip_consumable` | Held Tarot or planet |
| `tip_playing-card` | Hand card with an enhancement or a seal |
| `tip_voucher` | Voucher in the shop |
| `tip_tag` | Tag at the blind choice |
| `tip_pack` | Booster in the shop, then a card in the open pack |
| `tip_edition` | Foil, holo or polychrome joker with the side tooltip |

### Shop and end of round

| File | State |
|---|---|
| `cash-out` | Earnings screen |
| `shop_idle` | Shop at rest |
| `shop_pack-open` | Open pack |
| `run-info` | Run Info, hands tab |
| `collection_jokers` | Collection, jokers page |
| `options_game` | Options, game tab |
| `end_win` | Win |
| `end_lose` | Loss |

### Visual effects

| File | State |
|---|---|
| `vfx_foil`, `vfx_holo`, `vfx_polychrome`, `vfx_negative` | One joker of each edition, at rest and hovered |
| `vfx_dissolve` | Selling a joker, middle of the effect |
| `vfx_debuff` | Card debuffed by a boss |

## After comparing

Report the gaps and the status in the matrix of [the roadmap](ROADMAP.md). Measurements (positions, sizes) are noted as fractions of the client area, as in [the Bevy guide](UI_BEVY_GUIDE.md).
