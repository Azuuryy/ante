//! Jokers that create cards or level up hands (Lot 3, batch 5a): the Tarot, Spectral,
//! Joker and Stone card they make, and the moments they act on that the scoring pass
//! does not cover: the blind set, a Booster Pack opened, the first discard, the hand's
//! `before` and `after`.

use super::*;
use crate::card::Value;
use crate::event::JokerNote;
use crate::spectral::Spectral;

impl Game {
    /// Room in the consumable slots (`#G.consumeables.cards + buffer < limit`).
    pub(crate) fn has_consumable_room(&self) -> bool {
        self.consumables.len() < self.consumable_slots()
    }

    /// A random Tarot card in the consumable slots, said "+1 Tarot" on Joker `id`.
    pub(crate) fn create_tarot(&mut self, id: usize) {
        let held: Vec<Tarot> = self
            .consumables
            .iter()
            .filter_map(|c| match c {
                Consumable::Tarot(t) => Some(*t),
                _ => None,
            })
            .collect();
        let tarot = self.backend.roll_random_tarot(&held);
        self.consumables.push(tarot);
        self.joker_message(id, JokerNote::PlusTarot);
    }

    /// A random Spectral card (not The Soul or Black Hole), said "+1 Spectral".
    pub(crate) fn create_spectral(&mut self, id: usize) {
        let pool: Vec<Spectral> = Spectral::iter().filter(|s| !s.is_rare()).collect();
        let spectral = pool[self.backend.roll_index(pool.len())];
        self.consumables.push(Consumable::Spectral(spectral));
        self.joker_message(id, JokerNote::PlusSpectral);
    }

    /// Diet Cola sold: a Double Tag.
    pub(crate) fn acquire_double_tag(&mut self) {
        self.acquire_tag(Tag::Double, None);
    }

    /// `setting_blind` for the card makers: Cartomancer a Tarot card, Riff-raff up to two
    /// Common Jokers, Marble Joker a Stone card in the deck.
    pub(super) fn card_makers_set_blind(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::Cartomancer(_))) {
            if self.has_consumable_room() {
                self.create_tarot(id);
            }
        }
        for id in self.joker_ids(|j| matches!(j, Jokers::RiffRaff(_))) {
            let room = self.joker_slots().saturating_sub(self.jokers.len()).min(2);
            if room == 0 {
                continue;
            }
            for _ in 0..room {
                let mut joker = self.backend.gen_joker_of_rarity(
                    self.ante_current.number() as i32,
                    self.prob_mult,
                    &self.jokers,
                    Rarity::Common,
                );
                joker.set_edition(Edition::Base);
                self.add_acquired_joker(joker);
            }
            self.joker_message(id, JokerNote::PlusJoker);
        }
        for id in self.joker_ids(|j| matches!(j, Jokers::MarbleJoker(_))) {
            let mut stone = self.backend.gen_random_card(self.prob_mult, false, None);
            stone.enhancement = Some(Enhancement::Stone);
            self.deck.push(stone);
            self.playing_cards_added(1);
            self.joker_message(id, JokerNote::PlusStone);
        }
    }

    /// `open_booster`: Hallucination has 1 in 2 to create a Tarot card.
    pub(super) fn card_makers_open_booster(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::Hallucination(_))) {
            if self.has_consumable_room() && self.prob_roll(1, 2) {
                self.create_tarot(id);
            }
        }
    }

    /// `pre_discard`: on the round's first discard, Burnt Joker levels up the discarded
    /// poker hand.
    pub(super) fn card_makers_discard(&mut self, discarded: &[Card], first: bool) {
        if !first || discarded.is_empty() {
            return;
        }
        let burnt = self.joker_ids(|j| matches!(j, Jokers::BurntJoker(_)));
        if burnt.is_empty() {
            return;
        }
        let Ok(made) = SelectHand::new(discarded.to_vec()).best_hand_with(self.hand_rules()) else {
            return;
        };
        for id in burnt {
            self.planetarium.level_up(made.rank);
            self.joker_message(id, JokerNote::UpgradePlain);
        }
    }

    /// `before`: Space Joker has 1 in 4 to level up the hand played.
    pub(super) fn space_jokers(&mut self, rank: HandRank) {
        for id in self.joker_ids(|j| matches!(j, Jokers::SpaceJoker(_))) {
            if self.prob_roll(1, 4) {
                self.planetarium.level_up(rank);
                self.joker_message(id, JokerNote::LevelUp);
            }
        }
    }

    /// `after`: Loyalty Card counts the hands played since it was acquired.
    pub(super) fn count_loyalty(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::LoyaltyCard(_))) {
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += 1.0;
            }
        }
    }

    /// `first_hand_drawn`: Certificate adds a random card with a random seal to the hand.
    pub(crate) fn certificates(&mut self) {
        for _ in self.joker_ids(|j| matches!(j, Jokers::Certificate(_))) {
            let mut card = self.backend.gen_random_card(self.prob_mult, false, None);
            card.enhancement = None;
            card.edition = Edition::Base;
            let seals = [Seal::Purple, Seal::Gold, Seal::Blue, Seal::Red];
            card.seal = Some(seals[self.backend.roll_index(seals.len())]);
            self.available.extend(vec![card]);
            self.playing_cards_added(1);
        }
    }

    /// `before`, first hand of the round with one card: DNA copies it into the hand.
    pub(super) fn dna(&mut self, hand: &MadeHand) {
        if !self.hand_ranks_played_this_round.is_empty() || hand.all.len() != 1 {
            return;
        }
        for id in self.joker_ids(|j| matches!(j, Jokers::Dna(_))) {
            let original = hand.all[0];
            let mut copy = Card::new(original.value, original.suit);
            copy.enhancement = original.enhancement;
            copy.edition = original.edition;
            copy.seal = original.seal;
            copy.perma_bonus = original.perma_bonus;
            self.available.extend(vec![copy]);
            self.playing_cards_added(1);
            self.joker_message(id, JokerNote::Copied);
        }
    }

    /// `destroying_card`, first hand of the round: a single 6 played is destroyed by Sixth
    /// Sense, which creates a Spectral card if there is room.
    pub(super) fn sixth_sense(&mut self, played: &[Card], first: bool) {
        if !first || played.len() != 1 || played[0].value != Value::Six {
            return;
        }
        let senses = self.joker_ids(|j| matches!(j, Jokers::SixthSense(_)));
        if senses.is_empty() {
            return;
        }
        self.destroy_card(played[0].id);
        for id in senses {
            if self.has_consumable_room() {
                self.create_spectral(id);
            }
        }
    }

    /// End of round: Invisible Joker counts the rounds held.
    pub(super) fn age_invisible(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::InvisibleJoker(_))) {
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += 1.0;
            }
        }
    }

    /// `selling_self`: after 2 rounds, Invisible Joker duplicates a random other Joker
    /// (a Negative copy loses its edition) if there is room once it is gone.
    pub(crate) fn invisible_sold(&mut self, sold: &Jokers) {
        if !matches!(sold, Jokers::InvisibleJoker(_)) || sold.state().counter < 2.0 {
            return;
        }
        let others = self.jokers.clone();
        if others.is_empty() || self.jokers.len() >= self.joker_slots() {
            return;
        }
        let chosen = self.backend.pick_random_joker(others);
        let mut copy = self.backend.clone_joker(chosen);
        if copy.edition() == Edition::Negative {
            copy.set_edition(Edition::Base);
        }
        let mut state = copy.state();
        if matches!(copy, Jokers::InvisibleJoker(_)) {
            state.counter = 0.0;
        }
        copy.set_state(state);
        self.add_acquired_joker(copy);
    }

    /// Keeps the shop's Astronomer prices in step with the held Jokers.
    pub(crate) fn sync_shop_rules(&mut self) {
        self.shop.astronomer = self.count_jokers(|j| matches!(j, Jokers::Astronomer(_))) > 0;
    }

    fn joker_ids(&self, kind: fn(&Jokers) -> bool) -> Vec<usize> {
        self.jokers
            .iter()
            .filter(|j| kind(j))
            .map(|j| j.instance_id())
            .collect()
    }
}
