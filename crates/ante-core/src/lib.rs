use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub id: CardId,
    pub rank: Rank,
    pub suit: Suit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Rank,
    Suit,
}

pub fn sort_cards(cards: &mut [Card], mode: SortMode) {
    match mode {
        SortMode::Rank => cards.sort_by_key(|c| (c.rank as u8, c.suit as u8, c.id)),
        SortMode::Suit => cards.sort_by_key(|c| (c.suit as u8, c.rank as u8, c.id)),
    }
}

pub fn standard_deck() -> Vec<Card> {
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let ranks = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
    let mut cards = Vec::with_capacity(52);
    for suit in suits {
        for rank in ranks {
            cards.push(Card {
                id: CardId(cards.len() as u32),
                rank,
                suit,
            });
        }
    }
    cards
}

/// SplitMix64, version 1. This is Ante's own RNG, not Balatro seed compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRng {
    state: u64,
}
impl GameRng {
    pub const VERSION: u32 = 1;
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandError {
    Empty,
    TooManyCards,
}

pub fn evaluate(cards: &[Card]) -> Result<Hand, HandError> {
    if cards.is_empty() {
        return Err(HandError::Empty);
    }
    if cards.len() > 5 {
        return Err(HandError::TooManyCards);
    }
    let mut counts = BTreeMap::new();
    for card in cards {
        *counts.entry(card.rank as u8).or_insert(0usize) += 1;
    }
    let mut groups: Vec<_> = counts.values().copied().collect();
    groups.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.len() == 5 && cards.iter().all(|c| c.suit == cards[0].suit);
    let ranks: Vec<_> = counts.keys().copied().collect();
    let straight = cards.len() == 5
        && ranks.len() == 5
        && (ranks[4] - ranks[0] == 4 || ranks == [2, 3, 4, 5, 14]);
    Ok(match (groups.as_slice(), flush, straight) {
        ([5], true, _) => Hand::FlushFive,
        ([3, 2], true, _) => Hand::FlushHouse,
        ([5], _, _) => Hand::FiveOfAKind,
        (_, true, true) => Hand::StraightFlush,
        ([4, ..], _, _) => Hand::FourOfAKind,
        ([3, 2], _, _) => Hand::FullHouse,
        (_, true, _) => Hand::Flush,
        (_, _, true) => Hand::Straight,
        ([3, ..], _, _) => Hand::ThreeOfAKind,
        ([2, 2, ..], _, _) => Hand::TwoPair,
        ([2, ..], _, _) => Hand::Pair,
        _ => Hand::HighCard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hand(ranks: &[Rank], suits: &[Suit]) -> Vec<Card> {
        ranks
            .iter()
            .zip(suits)
            .enumerate()
            .map(|(i, (&rank, &suit))| Card {
                id: CardId(i as u32),
                rank,
                suit,
            })
            .collect()
    }
    #[test]
    fn categories() {
        use Hand::*;
        use Rank::*;
        use Suit::*;
        let mixed = [Clubs, Diamonds, Hearts, Spades, Clubs];
        let cases: &[(&[Rank], Hand)] = &[
            (&[Two, Four, Six, Eight, Ten], HighCard),
            (&[Two, Two, Six, Eight, Ten], Pair),
            (&[Two, Two, Six, Six, Ten], TwoPair),
            (&[Two, Two, Two, Eight, Ten], ThreeOfAKind),
            (&[Two, Three, Four, Five, Six], Straight),
            (&[Two, Two, Two, Eight, Eight], FullHouse),
            (&[Two, Two, Two, Two, Ten], FourOfAKind),
            (&[Two, Two, Two, Two, Two], FiveOfAKind),
        ];
        for (ranks, expected) in cases {
            assert_eq!(evaluate(&hand(ranks, &mixed)), Ok(*expected));
        }
        for (ranks, expected) in [
            ([Two, Four, Six, Eight, Ten], Flush),
            ([Ace, Two, Three, Four, Five], StraightFlush),
            ([Two, Two, Two, Three, Three], FlushHouse),
            ([Two; 5], FlushFive),
        ] {
            assert_eq!(evaluate(&hand(&ranks, &[Clubs; 5])), Ok(expected));
        }
    }
    #[test]
    fn deck_and_sort_are_deterministic() {
        let mut a = standard_deck();
        assert_eq!(a.len(), 52);
        let mut b = a.clone();
        b.reverse();
        for mode in [SortMode::Rank, SortMode::Suit] {
            sort_cards(&mut a, mode);
            sort_cards(&mut b, mode);
            assert_eq!(a, b);
        }
    }
    #[test]
    fn rng_has_stable_sequence() {
        let mut a = GameRng::new(0);
        assert_eq!(a.next_u64(), 0xe220a8397b1dcdaf);
        assert_eq!(a.next_u64(), 0x6e789e6aa1b965f4);
    }
    #[test]
    fn invalid_hand_sizes() {
        assert_eq!(evaluate(&[]), Err(HandError::Empty));
        assert_eq!(
            evaluate(&standard_deck()[..6]),
            Err(HandError::TooManyCards)
        );
    }
}
