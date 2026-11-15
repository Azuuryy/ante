use super::tally::{DeckTally, Tally};
use ante_core::contract::DeckCardView;

fn card(rank: char, suit: usize) -> DeckCardView {
    DeckCardView {
        rank,
        suit,
        edition: "Base",
        enhancement: None,
        seal: None,
        debuffed: false,
        remaining: true,
        effective_suits: 1 << suit,
    }
}

#[test]
fn tallies_count_base_and_effective_cards() {
    let wild = DeckCardView {
        enhancement: Some("Wild"),
        effective_suits: 0b1111,
        ..card('A', 0)
    };
    let debuffed = DeckCardView {
        debuffed: true,
        effective_suits: 0,
        ..card('K', 1)
    };
    let stone = DeckCardView {
        enhancement: Some("Stone"),
        ..card('5', 2)
    };
    let played = DeckCardView {
        remaining: false,
        ..card('7', 3)
    };
    let cards = [wild, debuffed, stone, played, card('T', 3)];

    let full = DeckTally::count(&cards, false);
    assert_eq!(
        full.aces,
        Tally {
            base: 1,
            effective: 1
        }
    );
    assert_eq!(
        full.faces,
        Tally {
            base: 1,
            effective: 0
        }
    );
    assert_eq!(
        full.numbers,
        Tally {
            base: 2,
            effective: 2
        }
    );
    assert_eq!(
        full.suits[0],
        Tally {
            base: 1,
            effective: 1
        }
    );
    assert_eq!(
        full.suits[1],
        Tally {
            base: 1,
            effective: 1
        }
    );
    assert_eq!(
        full.suits[3],
        Tally {
            base: 2,
            effective: 3
        }
    );
    assert_eq!(
        full.ranks[1],
        Tally {
            base: 1,
            effective: 0
        }
    );
    assert!(full.modded());

    let remaining = DeckTally::count(&cards, true);
    assert_eq!(remaining.numbers.base, 1);
    assert_eq!(remaining.suits[3].base, 1);
}
