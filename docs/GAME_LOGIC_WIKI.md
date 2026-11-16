# Game logic — wiki summary

This document gathers the reference rules useful to Ante and is the entry point to the entries of the [feature roadmap](ROADMAP_FEATURES.md). The current comparison target is Balatro 1.0.1o, as in the interface guide. This summary is deliberately implementation-oriented: it does not replace the wiki's detailed pages.

## Run loop

A run progresses through antes and blinds. The small blind, the big blind and the Boss blind each have a score target; beating a blind opens the cash out and the shop, then the next step. The shop lets the player buy or sell, use consumables, reroll the offers and open boosters. Packs offer a limited number of choices, sometimes followed by card targeting. A run ends in a win or a loss, possibly continued in Endless.

## Hand, cards and score

The selected cards make a poker hand according to the priority of the recognised hands. The cards that contribute to the score resolve from left to right. Card effects and Joker effects trigger during this resolution, so the position of cards and Jokers can change the result. The score is built from additions to Chips and Mult, then multiplications, in trigger order. Cards held in hand can also trigger separate effects.

Automatic sorting by rank and by suit already exists for the hand. It does not replace manual moves. Manually sorting the hand must change the visual order used during resolution, without changing card ids or consuming randomness.

## Jokers and trigger order

Jokers trigger their effects in the left-to-right order of their area, within each trigger window. Effects that trigger at different moments keep their own windows; moving a Joker does not change their phase priority, but it orders the effects that run in the same window. Retrigger effects can repeat card activations.

The client must let the player reorder held Jokers. The simulation alone owns the order, and the score trace must reflect it. A move must preserve the object and its state (edition, stickers, accumulated values).

## Current Joker values

The tooltip of an evolving Joker ends with a grey "(Currently …)" line. The value is coloured by its effect: +Mult in red, +Chips in blue, XMult in a red pill, money in yellow, a counter in orange. In the original, this line exists for Abstract Joker, Bootstraps, Ceremonial Dagger, Erosion, Flash Card, Green Joker, Red Card, Ride the Bus, Spare Trousers and Swashbuckler (+Mult). It also exists for Blue Joker, Bull, Castle, Runner, Square Joker, Stone Joker and Wee Joker (+Chips). For XMult: Canio, Campfire, Constellation, Glass Joker, Hit the Road, Hologram, Lucky Cat, Madness, Obelisk, Throwback, Vampire, Steel Joker, Yorick and Joker Stencil. For money: Cloud 9 and Satellite. Fortune Teller, Driver's License and Invisible Joker show a counter. Ice Cream, Popcorn, Ramen and Turtle Bean show their value in the main line.

Ante computes the value in the simulation (`CurrentValue`) and formats it in the client. As in the original, Green Joker, Castle, Obelisk and Yorick keep their value on the card itself: Green Joker and Castle keep it for the whole run, Obelisk returns to X1 when the hand played is strictly the most played one, Yorick counts the cards discarded since it was acquired. Joker Stencil is worth X(empty slots + Stencils held) and only applies while a slot is empty; Hit the Road resets to X1 every round. Only the Jokers implemented by the engine are covered; the engine's rule gaps are tracked in [the feature roadmap](ROADMAP_FEATURES.md).

## Score pacing

The original's delays are in game seconds. The game clock runs `speed` times faster than real time, and speeds up further on long sequences. For a five-card hand that only gives chips, without Jokers, the sequence lasts about 8.75 s at 1x. The delays are detailed in [the legacy guide](UI_LEGACY_GUIDE.md#score-pacing-and-speed).

## Enhancements, editions and seals

Enhancements change how the card behaves (for example Chips, Mult, glass, steel, stone, gold or lucky). Editions add their own bonuses. Seals are a separate modifier:

| Seal | Rule in short |
| --- | --- |
| Gold | Gives money when the card scores. |
| Red | Retriggers the scoring card. |
| Blue | May create a Planet card at end of round if the card stays in hand. |
| Purple | Creates a Tarot card when the card is discarded, if there is room. |

The experimental engine already carries the seal in the projection and shows a summary of its effect. Assignment coverage and the actual triggers must be verified separately before the mechanic is considered done.

## Debuff

A debuffed card keeps its identity and its slot, but its affected effects do not contribute like an active card. The rendering to reproduce adds a red cross and a subtle red tint. Cards in the deck, the hand, packs and other surfaces must share the same visual state when they show a debuffed card.

## Presentation and interaction

Hovering a card shows its identity, its description and its modifiers. A pack card must therefore show the description of the chosen object; the instruction "Choose this card from the pack" belongs to the choice context, not to the card's tooltip. The hover motion and the selection lift must be shared by every interactive card type and by the deck views.

## Sources and verification

- [Balatro Wiki](https://balatrowiki.org/) — the main source requested; direct automated access is blocked by robots.txt. The facts above are to be validated against the reference game and the local guide.
- [Activation sequence guide](https://balatrogame.fandom.com/wiki/Guide%3A_Activation_Sequence) — resolution order and retriggers.
- [Joker activation types](https://balatrogame.fandom.com/wiki/Activation_Type) — windows and effect order.
- [Ante's legacy interface guide](UI_LEGACY_GUIDE.md) — local observations of Balatro 1.0.1o and entry points.
- [Ante's Bevy interface guide](UI_BEVY_GUIDE.md) — mapping to the client and known gaps.

This summary is not permission to copy the game's code, text or resources. Implement the rules independently and explicitly record the divergences not yet verified.
