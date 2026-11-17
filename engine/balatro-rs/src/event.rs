//! What the game did outside a score trace, for run logs and messages: money a held card
//! paid at the end of the round, the consumables seals created, and the tags gained.
//! The adapter drains them after each action with [`Game::take_events`].

use crate::card::Card;
use crate::consumable::Consumable;
use crate::game::Game;
use crate::joker::Jokers;
use crate::tag::Tag;

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    /// A held Gold card paid at the end of the round.
    CardPaid { card: Card, amount: usize },
    /// A Blue Seal (end of round) or Purple Seal (discard) created a consumable.
    SealCreated { card: Card, consumable: Consumable },
    /// A tag was gained: it joins the tag row or takes effect at once.
    TagGained(Tag),
    /// A Joker's message outside the score trace, on a discard or at the end of the
    /// round, like `card_eval_status_text(card, 'extra')`.
    JokerMessage { joker: Jokers, note: JokerNote },
}

/// What a Joker says outside the score trace, with the original's text and colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JokerNote {
    /// "Upgrade!" in `G.C.CHIPS` (Castle).
    Upgrade,
    /// "Reset" in `G.C.RED` (Hit the Road at the end of the round).
    Reset,
    /// "$n" in `G.C.MONEY`.
    Dollars(usize),
    /// "Xn Mult" in `G.C.RED`, the factor in hundredths.
    XMult(usize),
    /// "-n Mult" in `G.C.RED` (Green Joker).
    MultMinus(usize),
    /// "Extinct!" (Gros Michel, Cavendish destroyed at the end of the round).
    Extinct,
    /// "Safe!" (they survived the roll).
    Safe,
    /// "+n Mult" in `G.C.RED` (Flash Card, Red Card, Fortune Teller).
    MultPlus(usize),
    /// "-Xn" in `G.C.RED`, the factor in hundredths (Ramen, per card discarded).
    XMultMinus(usize),
    /// "-n Hand Size" (Turtle Bean).
    HandSizeMinus(usize),
    /// "Eaten!" (a food Joker used up).
    Eaten,
    /// "Upgrade!" in the default colour (Campfire).
    UpgradePlain,
    /// The hands a Joker has left, as a bare number (Seltzer).
    Remaining(usize),
    /// "Drank!" (Seltzer used up).
    Drank,
    /// "Value Up!" in `G.C.MONEY` (Egg, Gift Card).
    ValueUp,
    /// "Upgrade!" in `G.C.MONEY` (Rocket).
    UpgradeMoney,
    /// "+n Hands" (Burglar).
    HandsPlus(usize),
    /// "Saved!" (Mr. Bones).
    Saved,
    /// "+1 Tarot" in `G.C.PURPLE`.
    PlusTarot,
    /// "+1 Spectral" in `G.C.SECONDARY_SET.Spectral`.
    PlusSpectral,
    /// "+1 Joker" in `G.C.BLUE` (Riff-raff).
    PlusJoker,
    /// "+1 Stone" (Marble Joker).
    PlusStone,
    /// "Level Up!" (Space Joker).
    LevelUp,
    /// "Copied!" in `G.C.CHIPS` (DNA).
    Copied,
    /// "-$n" paid for a Rental Joker.
    Rent(usize),
    /// "Disabled!" (a Perishable Joker's last round).
    Disabled,
    /// "Duplicated!" (Perkeo).
    Duplicated,
}

impl Game {
    pub(crate) fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// A message on the held Joker `id`, if it is still held.
    pub(crate) fn joker_message(&mut self, id: usize, note: JokerNote) {
        if let Some(joker) = self.jokers.iter().find(|j| j.instance_id() == id).cloned() {
            self.emit(GameEvent::JokerMessage { joker, note });
        }
    }

    /// Adds `amount` to every held Joker matching `kind`'s counter and says `note(value)`
    /// on it, with the counter's new value.
    pub(crate) fn grow_jokers(
        &mut self,
        kind: fn(&Jokers) -> bool,
        amount: f32,
        note: impl Fn(f32) -> JokerNote,
    ) {
        let ids: Vec<usize> = self
            .jokers
            .iter()
            .filter(|joker| kind(joker))
            .map(|joker| joker.instance_id())
            .collect();
        for id in ids {
            let Some(state) = self.joker_state_mut(id) else {
                continue;
            };
            state.counter += amount;
            let value = state.counter;
            self.joker_message(id, note(value));
        }
    }

    /// The events since the last call, oldest first.
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }
}
