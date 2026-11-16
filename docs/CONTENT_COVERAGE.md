# Content coverage

What Ante plays today against the original's content, and what is still missing. Update it whenever a Joker, deck, stake or unlock rule lands. The [roadmap](ROADMAP.md) tracks screens and visuals; this file tracks game content.

## Jokers

150 Jokers in the original. The engine holds the data of all 150 (`engine/balatro-types/src/joker/data.rs`); only those with real behaviour enter the shop and packs (`JokerEffects::is_implemented` in `engine/balatro-rs/src/joker.rs`).

| Rarity | In the pool | Missing |
| --- | --- | --- |
| Common | 58 | 0 |
| Uncommon | 58 | 6 |
| Rare | 16 | 4 |
| Legendary | 2 | 3 |
| Total | 134 | 13 |

Three more (Misprint, Golden Joker, Ice Cream) are opt-in behind the `extra-jokers` feature ([ENGINE.md](ENGINE.md)), which changes the shop pool for a given seed.

Credit Card waits for money below $0, and Gift Card for a sell value on consumables; both need a change to how the engine stores money and consumables.

Missing, by rarity:

- **Common:** Credit Card (waits, see above).
- **Uncommon:** Sixth Sense, Gift Card, Certificate, Showman, Astronomer.
- **Rare:** DNA, Blueprint, Invisible Joker, Brainstorm.
- **Legendary:** Canio, Triboulet, Perkeo.

Joker messages: the score trace shows the messages of Jokers that grow or reset while a hand scores (Wee Joker, Spare Trousers, Green Joker, Ride the Bus and Obelisk resets; Square Joker and Runner once they exist). Messages on a discard or at the end of the round (Castle, Hit the Road, Yorick, Green Joker's −1, Mail-In Rebate, Trading Card, Faceless Joker, Gold cards and seals) show once the score animation is over. Hiker's "Upgrade!" on the card and Vampire's "X1.1 Mult" during the hand are not shown yet. Order differs slightly: Ante shows them Joker by Joker, where the original goes card by card across the Jokers.

## Decks and stakes

| Content | Playable | Missing |
| --- | --- | --- |
| Decks | Red, Blue | Yellow, Green, Black, Magic, Nebula, Ghost, Abandoned, Checkered, Zodiac, Painted, Anaglyph, Plasma, Erratic |
| Stakes | White | Red, Green, Black, Blue, Purple, Orange, Gold |
| Challenges | none | all 20 |

The 15 decks have an id, a description and an unlock condition shown in the Collection, and the deck selector lists them, but only Red and Blue can start a run (`DeckChoice::deck`, the contract's `Deck`). Several missing decks need missing content first: Magic (Crystal Ball, The Fool), Nebula (Telescope), Ghost (Hex, Spectral cards in the shop), Anaglyph (Double Tag), Plasma (its chip and Mult balance and doubled blinds), Erratic (random ranks and suits).

## Unlocks and discovery

| Part | State |
| --- | --- |
| Discovery | Done: Jokers, consumables, packs and bosses met in a run are discovered and saved in `profile.json` (`discovered`). |
| Unlock conditions | Shown only: locked Collection entries show a padlock and their condition, but no condition is evaluated. |
| Unlocked set | Missing: the profile has no `unlocked` set separate from `discovered`; discovery stands in for it. |
| Career statistics | Missing: the counters the conditions read (hands played, cards discarded, rounds won, money earned, etc.). |
| Deck and stake unlocks | Missing: decks unlock by winning with other decks and stakes, which needs the stakes first. |
| Debug | Done: "Unlock all" in Settings > GAME (`dev-tools`). |
