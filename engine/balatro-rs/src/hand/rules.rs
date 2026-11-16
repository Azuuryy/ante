//! Flushes and straights under the held Jokers' rules, like the original `get_flush` and
//! `get_straight`: Four Fingers makes them with 4 cards, Shortcut lets a straight skip one
//! rank, Smeared Joker merges Hearts with Diamonds and Spades with Clubs.

use super::*;

/// The Jokers that change how hands are read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HandRules {
    pub four_fingers: bool,
    pub shortcut: bool,
    pub smeared: bool,
}

impl HandRules {
    fn needed(&self) -> usize {
        if self.four_fingers {
            4
        } else {
            5
        }
    }

    /// `Card:is_suit(suit, nil, true)`: a Stone card is no suit, a Wild card every suit,
    /// and Smeared Joker joins the two suits of a colour.
    pub fn flush_suit(&self, card: &Card, suit: Suit) -> bool {
        if card.enhancement == Some(Enhancement::Stone) {
            return false;
        }
        if card.enhancement == Some(Enhancement::Wild) {
            return true;
        }
        if self.smeared && is_red(card.suit) == is_red(suit) {
            return true;
        }
        card.suit == suit
    }
}

fn is_red(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

impl SelectHand {
    /// `get_flush`: the first suit of Spades, Hearts, Clubs, Diamonds that enough cards
    /// carry, and those cards in hand order.
    pub(crate) fn flush_with(&self, rules: HandRules) -> Option<SelectHand> {
        let needed = rules.needed();
        if self.len() > 5 || self.len() < needed {
            return None;
        }
        [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond]
            .into_iter()
            .map(|suit| {
                self.0
                    .iter()
                    .filter(|card| rules.flush_suit(card, suit))
                    .copied()
                    .collect::<Vec<_>>()
            })
            .find(|cards| cards.len() >= needed)
            .map(SelectHand::new)
    }

    /// `get_straight`: ranks from Ace-low to Ace-high, enough in a row, Shortcut letting
    /// one missing rank pass at a time; every card of the ranks in the run counts.
    pub(crate) fn straight_with(&self, rules: HandRules) -> Option<SelectHand> {
        let needed = rules.needed();
        if self.len() > 5 || self.len() < needed {
            return None;
        }
        let of_rank = |rank: u16| -> Vec<Card> {
            self.0
                .iter()
                .filter(|card| {
                    card.enhancement != Some(Enhancement::Stone) && rank_id(card.value) == rank
                })
                .copied()
                .collect()
        };
        let mut run: Vec<Card> = Vec::new();
        let (mut length, mut skipped, mut straight) = (0, false, false);
        for j in 1..=14u16 {
            let cards = of_rank(if j == 1 { 14 } else { j });
            if !cards.is_empty() {
                length += 1;
                skipped = false;
                run.extend(cards);
            } else if rules.shortcut && !skipped && j != 14 {
                skipped = true;
            } else {
                length = 0;
                skipped = false;
                if straight {
                    break;
                }
                run.clear();
            }
            if length >= needed {
                straight = true;
            }
        }
        straight.then(|| SelectHand::new(run))
    }

    /// A straight flush: a flush and a straight, scoring the cards of both.
    pub(crate) fn straight_flush_with(&self, rules: HandRules) -> Option<SelectHand> {
        let flush = self.flush_with(rules)?;
        let straight = self.straight_with(rules)?;
        let (flush, straight) = (flush.cards(), straight.cards());
        Some(SelectHand::new(
            self.0
                .iter()
                .filter(|card| flush.contains(card) || straight.contains(card))
                .copied()
                .collect(),
        ))
    }

    /// `best_hand` under `rules`.
    pub fn best_hand_with(&self, rules: HandRules) -> Result<MadeHand, PlayHandError> {
        if rules == HandRules::default() {
            return self.best_hand();
        }
        if self.0.is_empty() {
            return Err(PlayHandError::NoCards);
        }
        if self.len() > 5 {
            return Err(PlayHandError::TooManyCards);
        }
        let flush = self.flush_with(rules);
        let made = |hand: SelectHand, rank: HandRank| MadeHand {
            hand,
            rank,
            all: self.cards(),
        };
        if flush.is_some() {
            if self.is_five_of_kind().is_some() {
                return Ok(made(self.clone(), HandRank::FlushFive));
            }
            if self.is_fullhouse().is_some() {
                return Ok(made(self.clone(), HandRank::FlushHouse));
            }
        }
        if let Some(hand) = self.is_five_of_kind() {
            return Ok(made(hand, HandRank::FiveOfAKind));
        }
        if let Some(hand) = self.straight_flush_with(rules) {
            let royal = hand.cards().iter().all(|card| card.value >= Value::Ten);
            let rank = if royal {
                HandRank::RoyalFlush
            } else {
                HandRank::StraightFlush
            };
            return Ok(made(hand, rank));
        }
        if let Some(hand) = self.is_four_of_kind() {
            return Ok(made(hand, HandRank::FourOfAKind));
        }
        if let Some(hand) = self.is_fullhouse() {
            return Ok(made(hand, HandRank::FullHouse));
        }
        if let Some(hand) = flush {
            return Ok(made(hand, HandRank::Flush));
        }
        if let Some(hand) = self.straight_with(rules) {
            return Ok(made(hand, HandRank::Straight));
        }
        // Below a straight nothing depends on the rules.
        self.best_hand()
    }
}

/// `get_id`: 2 to 14, the Ace high.
fn rank_id(value: Value) -> u16 {
    match value {
        Value::Two => 2,
        Value::Three => 3,
        Value::Four => 4,
        Value::Five => 5,
        Value::Six => 6,
        Value::Seven => 7,
        Value::Eight => 8,
        Value::Nine => 9,
        Value::Ten => 10,
        Value::Jack => 11,
        Value::Queen => 12,
        Value::King => 13,
        Value::Ace => 14,
    }
}
