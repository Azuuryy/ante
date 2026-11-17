//! The 15 starting decks: `Back:apply_to_run` sets up the run once, and the rules the
//! deck keeps for the whole run (Green, Ghost, Anaglyph, Plasma) are read where they apply.

use super::*;
use crate::card::Value;
use balatro_types::deck::DeckVariant;

impl Game {
    /// `Back:apply_to_run`: limits, money, vouchers, starting consumables and the cards of
    /// the deck. Call once, on a new game, before it starts.
    pub fn apply_deck(&mut self, deck: DeckVariant) {
        self.deck_variant = deck;
        let shift = |value: usize, delta: isize| value.saturating_add_signed(delta);
        match deck {
            DeckVariant::Red => self.config.discards += 1,
            DeckVariant::Blue => self.config.plays += 1,
            DeckVariant::Yellow => {
                self.config.money_start += 10;
                self.money += 10;
            }
            DeckVariant::Black => {
                self.config.plays = shift(self.config.plays, -1);
                self.config.joker_slots += 1;
            }
            DeckVariant::Magic => {
                self.redeem_starting_voucher(Voucher::CrystalBall);
                for _ in 0..2 {
                    self.consumables.push(Consumable::Tarot(Tarot::Fool));
                }
            }
            DeckVariant::Nebula => {
                self.redeem_starting_voucher(Voucher::Telescope);
                self.config.consumable_slots = shift(self.config.consumable_slots, -1);
            }
            DeckVariant::Ghost => {
                self.consumables
                    .push(Consumable::Spectral(crate::spectral::Spectral::Hex));
            }
            DeckVariant::Zodiac => {
                for voucher in [
                    Voucher::TarotMerchant,
                    Voucher::PlanetMerchant,
                    Voucher::Overstock,
                ] {
                    self.redeem_starting_voucher(voucher);
                }
            }
            DeckVariant::Painted => {
                self.config.available += 2;
                self.config.joker_slots = shift(self.config.joker_slots, -1);
            }
            DeckVariant::Abandoned => {
                let cards: Vec<Card> = self
                    .deck
                    .cards()
                    .into_iter()
                    .filter(|card| !card.is_face_card())
                    .collect();
                self.deck = crate::deck::Deck::from_cards(cards);
            }
            DeckVariant::Checkered => {
                let cards: Vec<Card> = self
                    .deck
                    .cards()
                    .into_iter()
                    .map(|mut card| {
                        card.suit = match card.suit {
                            Suit::Club => Suit::Spade,
                            Suit::Diamond => Suit::Heart,
                            other => other,
                        };
                        card
                    })
                    .collect();
                self.deck = crate::deck::Deck::from_cards(cards);
            }
            DeckVariant::Erratic => {
                // Each card takes a random rank and suit (`erratic_suits_and_ranks`).
                let suits = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond];
                let values: Vec<Value> = Value::iter().collect();
                let count = self.deck.len();
                let mut cards = Vec::with_capacity(count);
                for _ in 0..count {
                    let value = values[self.backend.roll_index(values.len())];
                    let suit = suits[self.backend.roll_index(suits.len())];
                    cards.push(Card::new(value, suit));
                }
                self.deck = crate::deck::Deck::from_cards(cards);
            }
            DeckVariant::Green | DeckVariant::Anaglyph | DeckVariant::Plasma => {}
        }
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
    }

    fn redeem_starting_voucher(&mut self, voucher: Voucher) {
        self.vouchers.redeem(voucher);
        self.backend.on_voucher_bought(&voucher);
    }

    /// Plasma Deck: blinds need twice the score (`ante_scaling = 2`).
    pub(crate) fn ante_scaling(&self) -> usize {
        if self.deck_variant == DeckVariant::Plasma {
            2
        } else {
            1
        }
    }

    /// Ghost Deck: Spectral cards weigh 2 in the shop (`spectral_rate`).
    pub(crate) fn spectral_rate(&self) -> f64 {
        if self.deck_variant == DeckVariant::Ghost {
            2.0
        } else {
            0.0
        }
    }

    /// Green Deck: $2 per hand and $1 per discard left, and no interest.
    pub(crate) fn green_deck(&self) -> bool {
        self.deck_variant == DeckVariant::Green
    }

    /// Plasma Deck's `final_scoring_step`: Chips and Mult become their average.
    pub(super) fn balance_plasma(&mut self, trace: &mut Option<&mut TraceRecorder>) {
        if self.deck_variant != DeckVariant::Plasma {
            return;
        }
        let (chips_before, mult_before) = (self.chips, self.mult);
        let half = ((self.chips as f64 + self.mult) / 2.0).floor();
        self.chips = half as usize;
        self.mult = half;
        self.record_step(
            trace,
            ScoreSource::Balanced,
            chips_before,
            mult_before,
            false,
        );
    }

    /// Anaglyph Deck: a Double Tag after each Boss Blind defeated.
    pub(super) fn anaglyph_after_boss(&mut self) {
        if self.deck_variant == DeckVariant::Anaglyph {
            self.acquire_tag(Tag::Double, None);
        }
    }
}
