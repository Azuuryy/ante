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

/// A Joker's `calculate_dollar_bonus` at the end of a round.
pub(crate) fn dollar_bonus(joker: &Jokers) -> Option<usize> {
    match joker {
        #[cfg(feature = "ante-extra-jokers")]
        Jokers::GoldenJoker(_) => Some(4),
        _ => None,
    }
}
