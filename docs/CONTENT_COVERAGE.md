# Content coverage

What Ante plays today against the original's content, and what is still missing. Update it whenever a Joker, deck, stake or unlock rule lands. The [roadmap](ROADMAP.md) tracks screens and visuals; this file tracks game content.

## Jokers

150 Jokers in the original. The engine holds the data of all 150 (`engine/balatro-types/src/joker/data.rs`); only those with real behaviour enter the shop and packs (`JokerEffects::is_implemented` in `engine/balatro-rs/src/joker.rs`).

| Rarity | In the pool | Missing |
| --- | --- | --- |
| Common | 58 | 0 |
| Uncommon | 64 | 0 |
| Rare | 20 | 0 |
| Legendary | 5 | 0 |
| Total | 147 | 0 |

Three more (Misprint, Golden Joker, Ice Cream) are opt-in behind the `extra-jokers` feature ([ENGINE.md](ENGINE.md)), which changes the shop pool for a given seed. Every other Joker is in the shop and packs.

Known gaps: Hiker's "Upgrade!" on the card and Vampire's "X1.1 Mult" during the hand are not shown; Glass cards destroyed by The Hanged Man do not count for Glass Joker; Negative consumables (Perkeo) have no Negative look yet.

Joker messages: the score trace shows the messages of Jokers that grow or reset while a hand scores (Wee Joker, Spare Trousers, Green Joker, Ride the Bus and Obelisk resets; Square Joker and Runner once they exist). Messages on a discard or at the end of the round (Castle, Hit the Road, Yorick, Green Joker's −1, Mail-In Rebate, Trading Card, Faceless Joker, Gold cards and seals) show once the score animation is over. Hiker's "Upgrade!" on the card and Vampire's "X1.1 Mult" during the hand are not shown yet. Order differs slightly: Ante shows them Joker by Joker, where the original goes card by card across the Jokers.

## Decks and stakes

| Content | Playable | Missing |
| --- | --- | --- |
| Decks | All 15 | none |
| Stakes | All 8 | none |
| Challenges | none | all 20 |

Every deck applies `Back:apply_to_run` (limits, money, starting vouchers and consumables, the cards of the deck) and keeps its rules for the run: Green pays $2 per hand and $1 per discard left without interest, Ghost puts Spectral cards in the shop, Anaglyph gives a Double Tag after each Boss Blind, Plasma doubles every blind and balances Chips and Mult ("Balanced"). Left: Plasma's purple Chips and Mult boxes and its `gong` sound; decks are not locked yet (Lot 6).

Stakes are cumulative from White to Gold: Red takes the Small Blind's reward, Green and Purple raise blind amounts (`get_blind_amount` scaling 2 and 3), Black puts Eternal stickers on shop and pack Jokers, Blue takes a discard, Orange adds Perishable (disabled after 5 rounds) and Gold adds Rental ($1 to buy, $3 a round). Left: the sticker sprites on the cards (the tooltips already show them).

## Unlocks and discovery

| Part | State |
| --- | --- |
| Locks | Done: like the original, a profile starts with 91 items locked (Jokers, vouchers, decks). Locked Jokers and vouchers stay out of the shop and packs; Legendaries are never locked out and count as unlocked once found. |
| Unlock conditions | Done: the conditions of `check_for_unlock` are checked after each command (`services/unlocks`): career statistics, hands played or discarded, rounds and runs won, antes reached, the deck's cards, the Jokers held, discoveries, wins per deck and per stake. A seeded run counts statistics but unlocks nothing and records no win. |
| Career statistics | Done: hands, cards played, face cards, discards, sales, shop spending and rerolls, Tarot and Planet cards bought and used, playing cards bought, losses, the interest and single-hand streaks, Blank redeemed. |
| Decks and stakes | Done: a locked deck shows its condition and cannot start; a stake opens once the deck has won at the stake below. |
| Notification | In progress: "Unlocked!" and the item's name at the top of the window for 3 seconds, Ante's own look rather than the original `unlock_notify` with its card. |
| Gaps | Throwback ("Continue a saved run") unlocks on continuing in endless mode, since Ante has no saved runs. Face cards played count the J, Q and K played rather than the scoring ones. |
| Debug | Done: "Unlock all" in Settings > GAME (`dev-tools`). |
