//! Money earned by clearing a Blind, row by row like `G.FUNCS.evaluate_round`.

use crate::joker::Jokers;
use crate::tag::Tag;

/// The rows of one round evaluation, in the original order: the Blind, the hands left,
/// each Joker's dollar bonus, each tag paying out, then interest. Paid at cash out.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoundReward {
    pub blind: usize,
    pub hands_left: usize,
    pub hand_dollars: usize,
    pub jokers: Vec<(Jokers, usize)>,
    pub tags: Vec<(Tag, usize)>,
    pub interest: usize,
}

impl RoundReward {
    pub fn total(&self) -> usize {
        self.blind
            + self.hand_dollars
            + self
                .jokers
                .iter()
                .map(|(_, dollars)| dollars)
                .sum::<usize>()
            + self.tags.iter().map(|(_, dollars)| dollars).sum::<usize>()
            + self.interest
    }
}

/// A Joker's `calculate_dollar_bonus` at the end of a round: Golden Joker $4, Cloud 9 $1
/// per 9 in the full deck, Rocket $1 and $2 more per Boss Blind beaten, Satellite $1 per
/// Planet card used this run, Delayed Gratification $2 per discard left when none was used.
pub(crate) fn dollar_bonus(joker: &Jokers, game: &crate::game::Game) -> Option<usize> {
    let bonus = match joker {
        #[cfg(feature = "ante-extra-jokers")]
        Jokers::GoldenJoker(_) => 4,
        Jokers::Cloud9(_) => game
            .full_deck()
            .iter()
            .filter(|card| {
                card.value == crate::card::Value::Nine
                    && card.enhancement != Some(crate::card::Enhancement::Stone)
            })
            .count(),
        Jokers::Rocket(_) => 1 + joker.state().counter as usize,
        Jokers::Satellite(_) => game.planets_used.len(),
        Jokers::DelayedGratification(_) if game.discards_used_this_round() == 0 => {
            2 * game.discards_remaining
        }
        _ => return None,
    };
    (bonus > 0).then_some(bonus)
}
