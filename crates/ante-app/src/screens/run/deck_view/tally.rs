//! The counts of `G.UIDEF.view_deck`: base and effective tallies of the cards in view.

use ante_core::contract::{DeckCardView, RANKS};

/// A base count and the count the cards actually play as (debuffed cards drop out of the
/// effective ranks, faces and numbers; Wild cards count for every suit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tally {
    pub(crate) base: usize,
    pub(crate) effective: usize,
}

impl Tally {
    fn add(&mut self, base: bool, effective: bool) {
        self.base += usize::from(base);
        self.effective += usize::from(effective);
    }

    pub(crate) fn modded(self) -> bool {
        self.base != self.effective
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DeckTally {
    pub(crate) aces: Tally,
    pub(crate) faces: Tally,
    pub(crate) numbers: Tally,
    /// Spades, Hearts, Clubs, Diamonds.
    pub(crate) suits: [Tally; 4],
    /// In `RANKS` order, Ace first.
    pub(crate) ranks: [Tally; 13],
}

impl DeckTally {
    /// Counts the non-Stone cards, only those still in the deck when `remaining`.
    pub(crate) fn count(cards: &[DeckCardView], remaining: bool) -> Self {
        let mut tally = Self::default();
        let counted = cards
            .iter()
            .filter(|card| card.enhancement != Some("Stone") && (!remaining || card.remaining));
        for card in counted {
            let live = !card.debuffed;
            let Some(rank) = RANKS.find(card.rank) else {
                continue;
            };
            tally.aces.add(card.rank == 'A', card.rank == 'A' && live);
            let face = matches!(card.rank, 'K' | 'Q' | 'J');
            tally.faces.add(face, face && live);
            let number = !face && card.rank != 'A';
            tally.numbers.add(number, number && live);
            tally.ranks[rank].add(true, live);
            for (suit, count) in tally.suits.iter_mut().enumerate() {
                count.add(card.suit == suit, card.effective_suits & (1 << suit) != 0);
            }
        }
        tally
    }

    /// `modded`: faces or suits differ, which adds "Effective" and the footnote.
    pub(crate) fn modded(&self) -> bool {
        self.faces.modded() || self.suits.iter().any(|suit| suit.modded())
    }
}
