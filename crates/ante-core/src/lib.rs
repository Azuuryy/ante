mod card;
pub mod contract;
mod hand_eval;
pub mod native;
mod rng;
pub mod score;

pub use card::{Card, CardId, Rank, SortMode, Suit, sort_cards, standard_deck};
pub use hand_eval::{Hand, HandError, evaluate};
pub use rng::GameRng;

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
