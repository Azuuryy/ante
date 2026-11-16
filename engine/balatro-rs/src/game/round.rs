use super::*;

impl Game {
    // Fires when the round that just ended cleared the blind, before played
    // cards are swapped out and replacements drawn in. `self.available` still
    // reflects what was actually held in hand at the moment the round ended.
    pub(super) fn handle_round_end(&mut self) {
        for e in self.effect_registry.on_round_end.clone() {
            if let Effects::OnRoundEnd(f) = e {
                f.lock().unwrap()(self)
            }
        }

        let planetarium = self.planetarium.clone();
        for card in self.available.not_selected() {
            if card.seal == Some(Seal::Blue) && self.consumables.len() < self.consumable_slots() {
                let planet = self.backend.roll_random_planet(&planetarium, &[]);
                self.consumables.push(planet);
            }
        }
    }

    pub(crate) fn clear_blind(&mut self) {
        self.temporary_hand_size = 0;
        self.score = self.config.base_score;
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
        self.hand_ranks_played_this_round.clear();
        self.mouth_hand = None;
        self.discarded_this_round.clear();
        self.boss_disabled_by_luchador = false;
        self.deck.append(&mut self.discarded);
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.backend.shuffle_deck(&mut self.deck);
        self.roll_discard_selectors();
    }

    /// The boss currently in effect, or `None` if we're not in the middle of
    /// that boss's own round, or its effect is disabled (Chicot held, or
    /// Luchador sold this Boss Blind).
    pub(crate) fn active_boss(&self) -> Option<BossBlind> {
        if self.blind != Some(Blind::Boss) {
            return None;
        }
        if self.jokers.iter().any(|j| matches!(j, Jokers::Chicot(_))) {
            return None;
        }
        if self.boss_disabled_by_luchador {
            return None;
        }
        self.current_boss
    }

    /// Discards actually usable right now
    pub fn discards(&self) -> usize {
        if self.active_boss() == Some(BossBlind::Water) {
            0
        } else {
            self.discards_remaining
        }
    }

    // Castle/MailInRebate lock onto a fresh random suit/rank each round
    // ("changes every round") - unconditional, overwrites whatever was
    // already there.
    pub(super) fn roll_discard_selectors(&mut self) {
        for j in self.jokers.iter_mut() {
            self.backend.roll_discard_selector(j);
        }
    }

    // draw from deck to available
    pub(super) fn draw(&mut self, count: usize) {
        self.available.extend(self.deck.draw(count));
    }

    // shuffle and deal new cards to available
    pub(crate) fn deal(&mut self) {
        // add discarded back to deck, emptying in process
        self.deck.append(&mut self.discarded);
        // add available back to deck and empty
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.backend.shuffle_deck(&mut self.deck);
        self.draw(self.hand_size());
    }

    /// Reshuffles discarded/held cards back into the deck and redraws a
    /// fresh hand, regardless of stage. Just a helper.
    pub fn redeal(&mut self) {
        self.deal();
    }

    pub(crate) fn select_card(&mut self, card: Card) -> Result<(), GameError> {
        if self.available.selected().len() >= self.config.selected_max {
            return Err(GameError::InvalidSelectCard);
        }
        self.available.select_card(card)
    }

    pub(crate) fn move_card(
        &mut self,
        direction: MoveDirection,
        card: Card,
    ) -> Result<(), GameError> {
        self.available.move_card(direction, card)
    }

    #[cfg(test)]
    pub(crate) fn play_selected(&mut self) -> Result<(), GameError> {
        self.play_selected_inner(None)
    }

    pub(super) fn play_selected_inner(
        &mut self,
        trace: Option<&mut TraceRecorder>,
    ) -> Result<(), GameError> {
        if self.plays == 0 {
            return Err(GameError::NoRemainingPlays);
        }
        self.plays -= 1;
        let selected = SelectHand::new(self.available.selected());
        let best = selected.best_hand()?;
        let scored = best.clone();
        let score = self.calc_score_inner(best, trace);
        let clear_blind = self.handle_score(score)?;
        // must run before the discard/redraw below: self.available still reflects
        // the cards actually held when the round ended, not their replacements.
        if clear_blind {
            self.handle_round_end();
        }
        #[cfg(feature = "ante-extra-jokers")]
        self.age_ice_cream();
        self.discarded.extend(self.available.selected());
        let removed = self.available.remove_selected();
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            removed
        };
        self.draw(n);
        for card in scored.hand.cards() {
            if card.enhancement == Some(Enhancement::Glass) && self.prob_roll(1, 4) {
                self.destroy_card(card.id);
            }
        }
        if clear_blind {
            self.clear_blind();
        }
        Ok(())
    }

    // discard selected cards from available and draw equal number back to available
    pub(crate) fn discard_selected(&mut self) -> Result<(), GameError> {
        if self.discards() == 0 {
            return Err(GameError::NoRemainingDiscards);
        }
        self.discards_remaining -= 1;
        let discarded = self.available.selected();
        self.discarded.extend(discarded.iter().copied());
        self.discarded_this_round.extend(discarded.iter().copied());
        self.total_cards_discarded += discarded.len();
        let removed = self.available.remove_selected();
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            removed
        };
        self.draw(n);

        // Check for purple seals
        for card in &discarded {
            if card.seal == Some(Seal::Purple) && self.consumables.len() < self.consumable_slots() {
                let excl: Vec<Tarot> = self
                    .consumables
                    .iter()
                    .filter_map(|c| match c {
                        Consumable::Tarot(t) => Some(*t),
                        _ => None,
                    })
                    .collect();
                let tarot = self.backend.roll_random_tarot(&excl);
                self.consumables.push(tarot);
            }
        }

        let hand = MadeHand {
            hand: SelectHand::new(discarded.clone()),
            rank: HandRank::HighCard,
            all: discarded,
        };
        for e in self.effect_registry.on_discard.clone() {
            if let Effects::OnDiscard(f) = e {
                f.lock().unwrap()(self, hand.clone())
            }
        }
        Ok(())
    }

    pub(crate) fn destroy_card(&mut self, id: usize) {
        self.available.remove_by_id(id);
        self.deck.remove_by_id(id);
        self.discarded.retain(|c| c.id != id);
    }

    pub(crate) fn mutate_card<F: Fn(&mut Card) + Copy>(&mut self, id: usize, f: F) {
        self.available.mutate_card(id, f);
        self.deck.mutate_card(id, f);
    }

    pub(crate) fn joker_state_mut(&mut self, id: usize) -> Option<&mut JokerState> {
        self.jokers
            .iter_mut()
            .find(|j| j.instance_id() == id)
            .map(|j| j.state_mut())
    }

    pub(crate) fn is_face_card(&self, card: &Card) -> bool {
        card.is_face_card()
            || self
                .effect_registry
                .rule_flags
                .contains(&RuleFlag::AllCardsAreFace)
    }

    pub(crate) fn is_even(&self, card: &Card) -> bool {
        card.is_even_impl(self.is_face_card(card))
    }

    pub(crate) fn is_odd(&self, card: &Card) -> bool {
        card.is_odd_impl(self.is_face_card(card))
    }

    /// Whether the active boss (if any) currently debuffs card
    pub fn is_card_debuffed(&self, card: &Card) -> bool {
        match self.active_boss() {
            Some(BossBlind::Club) => card.matches_suit(Suit::Club),
            Some(BossBlind::Goad) => card.matches_suit(Suit::Spade),
            Some(BossBlind::Head) => card.matches_suit(Suit::Heart),
            Some(BossBlind::Window) => card.matches_suit(Suit::Diamond),
            Some(BossBlind::Plant) => self.is_face_card(card),
            _ => false,
        }
    }

    // Filters debuffed cards out of a collection
    pub fn non_debuffed<'a>(&self, cards: impl IntoIterator<Item = &'a Card>) -> Vec<Card> {
        cards
            .into_iter()
            .filter(|c| !self.is_card_debuffed(c))
            .copied()
            .collect()
    }

    pub(crate) fn most_played_hand_rank(&self) -> HandRank {
        HandRank::iter()
            // royal flush has no storage slot of its own, its counted as straight flush
            .filter(|r| *r != HandRank::RoyalFlush)
            .max_by_key(|r| self.planetarium.level(*r).plays)
            .unwrap_or(HandRank::HighCard)
    }

    // Every card the player owns this run, regardless of whether it's
    // currently undrawn, in hand, or already discarded this round.
    // `self.deck` alone is only the undrawn remainder mid-round.
    pub fn full_deck(&self) -> Vec<Card> {
        let mut all = self.deck.cards();
        all.extend(self.available.cards());
        all.extend(self.discarded.clone());
        all
    }

    pub fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.backend
            .prob_roll(numerator.saturating_mul(self.prob_mult), denominator)
    }

    #[cfg(feature = "ante-extra-jokers")]
    pub(crate) fn roll_misprint_mult(&mut self) -> usize {
        self.backend.roll_misprint_mult()
    }

    #[cfg(feature = "ante-extra-jokers")]
    pub(super) fn age_ice_cream(&mut self) {
        let before = self.jokers.len();
        self.jokers.retain_mut(|joker| {
            if matches!(joker, Jokers::IceCream(_)) {
                let state = joker.state_mut();
                state.counter += 1.0;
                return state.counter < 20.0;
            }
            true
        });
        if self.jokers.len() != before {
            self.effect_registry
                .register_jokers(self.jokers.clone(), &self.clone());
        }
    }
}
