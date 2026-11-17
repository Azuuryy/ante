//! Stakes, cumulative from White (1) to Gold (8), and the stickers they put on shop and
//! pack Jokers: Red takes the Small Blind's reward, Green and Purple raise blind amounts
//! (`get_blind_amount` scaling 2 and 3), Black adds Eternal, Blue takes a discard, Orange
//! adds Perishable and Gold adds Rental.

use super::*;
use crate::event::JokerNote;
use crate::pack::PackContent;
use balatro_types::joker::Stickers;

/// Rounds a Perishable Joker works for (`G.GAME.perishable_rounds`).
pub(crate) const PERISHABLE_ROUNDS: u8 = 5;
/// What a Rental Joker costs each round (`G.GAME.rental_rate`).
pub(crate) const RENTAL_RATE: usize = 3;

impl Game {
    /// Sets the stake for a new run, after the deck: Blue Stake takes a discard.
    pub fn apply_stake(&mut self, stake: usize) {
        self.stake = stake.clamp(1, 8);
        if self.stake >= 5 {
            self.config.discards = self.config.discards.saturating_sub(1);
            self.discards_remaining = self.discards_per_round();
        }
    }

    /// `G.GAME.modifiers.scaling`: 1, 2 from Green Stake, 3 from Purple Stake.
    fn scaling(&self) -> usize {
        match self.stake {
            6.. => 3,
            3.. => 2,
            _ => 1,
        }
    }

    /// `get_blind_amount` for the current ante under the stake's scaling.
    pub(crate) fn blind_amount(&self) -> usize {
        let amounts: [usize; 8] = match self.scaling() {
            1 => return self.ante_current.base(),
            2 => [300, 900, 2600, 8000, 20000, 36000, 60000, 100000],
            _ => [300, 1000, 3200, 9000, 25000, 60000, 110000, 200000],
        };
        let ante = self.ante_current.number();
        if ante < 1 {
            return 100;
        }
        if ante <= 8 {
            return amounts[ante - 1];
        }
        // Endless: `a × (b + (k·c)^d)^c`, rounded down to two significant figures.
        let (a, b, k) = (amounts[7] as f64, 1.6, 0.75);
        let c = (ante - 8) as f64;
        let d = 1.0 + 0.2 * c;
        let amount = (a * (b + (k * c).powf(d)).powf(c)).floor();
        let unit = 10f64.powf((amount.log10() - 1.0).floor());
        (amount - amount % unit) as usize
    }

    /// The blind's own reward: Red Stake and up, the Small Blind pays nothing.
    pub fn blind_reward(&self, blind: Blind) -> usize {
        if blind == Blind::Small && self.stake >= 2 {
            0
        } else {
            blind.reward()
        }
    }

    /// `create_card` for `shop_jokers` and `pack_cards`: one roll for Eternal (Black
    /// Stake, above 0.7) or Perishable (Orange Stake, 0.4 to 0.7), another for Rental
    /// (Gold Stake, above 0.7).
    pub(crate) fn roll_stickers(&mut self, joker: &mut Jokers) {
        if self.stake < 4 {
            return;
        }
        let mut stickers = joker.stickers();
        let poll = self.backend.roll_unit();
        if poll > 0.7 && joker.eternal_compatible() {
            stickers.eternal = true;
        } else if self.stake >= 7 && poll > 0.4 && poll <= 0.7 && joker.perishable_compatible() {
            stickers.perishable = true;
            let mut state = joker.state();
            state.perish_rounds = PERISHABLE_ROUNDS;
            joker.set_state(state);
        }
        if self.stake >= 8 && self.backend.roll_unit() > 0.7 {
            stickers.rental = true;
        }
        joker.set_stickers(stickers);
    }

    /// Stickers on the shop's Jokers, after it is stocked or rerolled.
    pub(super) fn sticker_shop(&mut self) {
        let mut jokers = std::mem::take(&mut self.shop.jokers);
        for joker in &mut jokers {
            self.roll_stickers(joker);
        }
        self.shop.jokers = jokers;
    }

    /// Stickers on the Jokers of a pack just opened.
    pub(super) fn sticker_pack(&mut self) {
        let Some(mut open) = self.open_pack.take() else {
            return;
        };
        for content in &mut open.contents {
            if let PackContent::Joker(joker) = content {
                self.roll_stickers(joker);
            }
        }
        self.open_pack = Some(open);
    }

    /// End of round: each Rental Joker costs $3, each Perishable one counts down and is
    /// disabled at 0 ("Disabled!").
    pub(super) fn stickers_end_of_round(&mut self) {
        let jokers: Vec<(usize, Stickers, u8)> = self
            .jokers
            .iter()
            .map(|j| (j.instance_id(), j.stickers(), j.state().perish_rounds))
            .collect();
        for (id, stickers, left) in jokers {
            if stickers.rental {
                self.money = self.money.saturating_sub(RENTAL_RATE);
                self.joker_message(id, JokerNote::Rent(RENTAL_RATE));
            }
            if stickers.perishable && left > 0 {
                if let Some(state) = self.joker_state_mut(id) {
                    state.perish_rounds -= 1;
                }
                let note = if left == 1 {
                    JokerNote::Disabled
                } else {
                    JokerNote::Remaining(usize::from(left - 1))
                };
                self.joker_message(id, note);
            }
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }
}
