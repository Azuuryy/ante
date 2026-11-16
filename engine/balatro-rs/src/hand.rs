use indexmap::IndexMap;
use itertools::Itertools;
use std::fmt;

use crate::card::Card;
use crate::card::Enhancement;
use crate::card::Suit;
use crate::card::Value;
use crate::error::PlayHandError;
use crate::rank::HandRank;

mod rules;
pub use rules::HandRules;

// Hand, SelectHand and MadeHand are all representations of a collection of Card,
// just at different phases in the cycle of selecting, executing and scoring cards.
// Hand represents all drawn cards, cards available for action (play/discard).
// SelectHand represents (up to 5) cards user selects from hand for action.
// MadeHand represents actual poker hand level and associated cards from a selected hand.

// Hand represents all drawn cards, cards available for action (play/discard)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Hand(Vec<Card>);

// MadeHand represents actual poker hand level and associated cards from a selected hand.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MadeHand {
    pub hand: SelectHand,
    pub rank: HandRank,
    pub all: Vec<Card>,
}

// SelectHand represents (up to 5) cards user selects from hand for action
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SelectHand(Vec<Card>);

impl SelectHand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    // Get all values in a hand. Sorted lowest to highest.
    fn values(&self) -> Vec<Value> {
        self.0.iter().map(|x| x.value).sorted().collect()
    }
    pub(crate) fn cards(&self) -> Vec<Card> {
        self.0.clone()
    }

    // Get map of each value with corresponding cards.
    // For example, Ks, Ah, Jh, Jc, Jd -> {A: [Ah], K: [Ks], J: [Jh, Jc: Jd]}
    fn values_freq(&self) -> IndexMap<Value, Vec<Card>> {
        let mut counts: IndexMap<Value, Vec<Card>> = IndexMap::new();
        for card in self.0.clone() {
            if let Some(cards) = counts.get(&card.value) {
                let mut copy = cards.clone();
                copy.push(card);
                counts.insert(card.value, copy);
            } else {
                counts.insert(card.value, vec![card]);
            }
        }
        // Return sorted by value
        counts
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
            .collect()
    }

    // Get map of each suit with corresponding cards.
    // For example, Ks, Ah, Jh, Jc, Jd -> {h: [Jh, Ah], s: [Ks], c: [Jc], d: [Jd]}
    #[allow(dead_code)]
    pub(crate) fn suits_freq(&self) -> IndexMap<Suit, Vec<Card>> {
        let mut counts: IndexMap<Suit, Vec<Card>> = IndexMap::new();
        for card in self.0.clone() {
            if let Some(cards) = counts.get(&card.suit) {
                let mut copy = cards.clone();
                copy.push(card);
                counts.insert(card.suit, copy);
            } else {
                counts.insert(card.suit, vec![card]);
            }
        }
        // Return sorted by suit
        counts
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
            .collect()
    }

    // Original relative order, values_freq()'s groups are order-preserving
    // individually, but concatenating two of them (two pair, full house)
    // isn't, without this.
    fn cards_of_values(&self, values: &[Value]) -> Vec<Card> {
        self.0
            .iter()
            .filter(|c| values.contains(&c.value))
            .cloned()
            .collect()
    }

    /// Can play any number of cards, it is our responsibility
    /// to determine the best hand. Higher tier hands take precedence
    /// over lower tier hands regardless of their level or scoring.
    /// For example, if hand is Kd Kd Kd Kd 2d, best hand will be a
    // Four of a Kind and never a Flush.
    //
    // Hand ranking:
    // FlushFive
    // FlushHouse
    // FiveOfAKind
    // RoyalFlush
    // StraightFlush
    // FourOfAKind
    // FullHouse
    // Flush
    // Straight
    // ThreeOfAKind
    // TwoPair
    // OnePair
    // HighCard
    pub fn best_hand(&self) -> Result<MadeHand, PlayHandError> {
        if self.0.is_empty() {
            return Err(PlayHandError::NoCards);
        }
        if self.len() > 5 {
            return Err(PlayHandError::TooManyCards);
        }

        // We start trying to evaluate best hands first, so we
        // can return best hand right when we find it.
        let (hand, rank) = if let Some(hand) = self.is_flush_five() {
            (hand, HandRank::FlushFive)
        } else if let Some(hand) = self.is_flush_house() {
            (hand, HandRank::FlushHouse)
        } else if let Some(hand) = self.is_five_of_kind() {
            (hand, HandRank::FiveOfAKind)
        } else if let Some(hand) = self.is_royal_flush() {
            (hand, HandRank::RoyalFlush)
        } else if let Some(hand) = self.is_straight_flush() {
            (hand, HandRank::StraightFlush)
        } else if let Some(hand) = self.is_four_of_kind() {
            (hand, HandRank::FourOfAKind)
        } else if let Some(hand) = self.is_fullhouse() {
            (hand, HandRank::FullHouse)
        } else if let Some(hand) = self.is_flush() {
            (hand, HandRank::Flush)
        } else if let Some(hand) = self.is_straight() {
            (hand, HandRank::Straight)
        } else if let Some(hand) = self.is_three_of_kind() {
            (hand, HandRank::ThreeOfAKind)
        } else if let Some(hand) = self.is_two_pair() {
            (hand, HandRank::TwoPair)
        } else if let Some(hand) = self.is_pair() {
            (hand, HandRank::OnePair)
        } else if let Some(hand) = self.is_highcard() {
            (hand, HandRank::HighCard)
        } else {
            return Err(PlayHandError::UnknownHand);
        };
        Ok(MadeHand {
            hand,
            rank,
            all: self.cards(),
        })
    }

    pub(crate) fn is_highcard(&self) -> Option<SelectHand> {
        if self.len() < 1 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| !val.is_empty())?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_pair(&self) -> Option<SelectHand> {
        if self.len() < 2 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 2)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_two_pair(&self) -> Option<SelectHand> {
        if self.len() < 4 {
            return None;
        }

        let (first_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 2)?;

        let (second_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(key, val)| *key != first_val && val.len() >= 2)?;

        Some(SelectHand::new(
            self.cards_of_values(&[first_val, second_val]),
        ))
    }

    pub(crate) fn is_three_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 3 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 3)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_straight(&self) -> Option<SelectHand> {
        if self.len() != 5 {
            return None;
        }
        // Iterate our sorted values. Each value must be one more than the previous.
        let values = self.values();
        if values.windows(2).all(|v| (v[1] as u16 - v[0] as u16) == 1) {
            return Some(self.clone());
        }

        // Special case for low ace.
        // Values are sorted with Ace as high (2, 3, 4, 5, A)
        // Therefore, we can check that last value is ace, first value is two.
        // Then remove the last value (ace) from vec and check for incremental values
        // for everything else (2, 3, 4, 5).
        if values[4] == Value::Ace && values[0] == Value::Two {
            let skip_last: Vec<Value> = values.into_iter().rev().skip(1).rev().collect();
            if skip_last
                .windows(2)
                .all(|v| (v[1] as u16 - v[0] as u16) == 1)
            {
                return Some(self.clone());
            }
        }
        None
    }

    pub(crate) fn is_flush(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }
        // Each wild card in hand reduces the number of a suit needed to make a flush
        // since a wild can be any suit.
        let wilds: Vec<Card> = self
            .0
            .iter()
            .filter(|c| c.enhancement == Some(Enhancement::Wild))
            .cloned()
            .collect();
        let wild_count = wilds.len();
        let needed = 5usize.saturating_sub(wild_count);

        if needed == 0 {
            return Some(self.clone());
        }

        let mut suit_groups: IndexMap<Suit, Vec<Card>> = IndexMap::new();
        for card in self
            .0
            .iter()
            .filter(|c| c.enhancement != Some(Enhancement::Wild))
        {
            suit_groups.entry(card.suit).or_default().push(*card);
        }

        if let Some((_, mut cards)) = suit_groups.into_iter().find(|(_, v)| v.len() >= needed) {
            cards.extend(wilds);
            return Some(SelectHand::new(cards));
        }
        None
    }

    pub(crate) fn is_fullhouse(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }

        let (three_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 3)?;

        let (two_val, _) = self
            .values_freq()
            .into_iter()
            .find(|(key, val)| *key != three_val && val.len() >= 2)?;

        Some(SelectHand::new(self.cards_of_values(&[three_val, two_val])))
    }

    pub(crate) fn is_four_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 4 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 4)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_straight_flush(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_straight().is_some() {
            return Some(self.clone());
        }
        None
    }

    pub(crate) fn is_royal_flush(&self) -> Option<SelectHand> {
        if self.is_straight_flush().is_some()
            && self.values().into_iter().eq(vec![
                Value::Ten,
                Value::Jack,
                Value::Queen,
                Value::King,
                Value::Ace,
            ])
        {
            return Some(self.clone());
        }
        None
    }

    pub(crate) fn is_five_of_kind(&self) -> Option<SelectHand> {
        if self.len() < 5 {
            return None;
        }
        let (value, _) = self
            .values_freq()
            .into_iter()
            .find(|(_key, val)| val.len() >= 5)?;
        Some(SelectHand::new(self.cards_of_values(&[value])))
    }

    pub(crate) fn is_flush_house(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_fullhouse().is_some() {
            return Some(self.clone());
        }
        None
    }

    pub(crate) fn is_flush_five(&self) -> Option<SelectHand> {
        if self.is_flush().is_some() && self.is_five_of_kind().is_some() {
            return Some(self.clone());
        }
        None
    }
}

impl Default for SelectHand {
    fn default() -> Self {
        let cards: Vec<Card> = Vec::new();
        Self(cards)
    }
}

impl fmt::Display for SelectHand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for card in &self.0 {
            write!(f, "{}", card)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests;
