use super::*;

impl Game {
    pub fn calc_score(&mut self, hand: MadeHand) -> usize {
        self.calc_score_inner(hand, None)
    }

    /// Same scoring pass as `calc_score`, plus an ordered `ScoreTrace` of
    /// every chips/mult contribution along the way.
    pub fn calc_score_traced(&mut self, hand: MadeHand) -> (usize, ScoreTrace) {
        let mut recorder = TraceRecorder {
            money: Some(self.money),
            ..TraceRecorder::default()
        };
        let score = self.calc_score_inner(hand, Some(&mut recorder));
        (score, recorder.trace)
    }

    /// Each Joker's per-card effect on `card`, in Joker order, like the
    /// `calculate_joker{individual = true}` loop of `evaluate_play`.
    fn joker_card_effects(
        &mut self,
        trace: &mut Option<&mut TraceRecorder>,
        card: Card,
        hand: &MadeHand,
        held: bool,
        retrigger: bool,
    ) {
        for joker in self.jokers.clone() {
            for effect in joker.effects(self) {
                let apply = match (effect, held) {
                    (Effects::OnCardScored(f), false) | (Effects::OnCardHeld(f), true) => f,
                    _ => continue,
                };
                let chips_before = self.chips;
                let mult_before = self.mult;
                let grown_from = self.joker_counter(&joker);
                apply.lock().unwrap()(self, card, hand);
                if self.joker_counter(&joker) > grown_from {
                    self.record_message(trace, ScoreSource::JokerUpgrade(joker.clone()));
                }
                self.record_step(
                    trace,
                    ScoreSource::JokerOnCard(joker.clone(), card),
                    chips_before,
                    mult_before,
                    retrigger,
                );
            }
        }
    }

    // Extension point for retrigger (jokers + red seal)
    pub(super) fn trigger_count_played(&mut self, card: &Card, is_first: bool) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_played.clone() {
            if let Effects::TriggerCountPlayed(f) = e {
                n += f.lock().unwrap()(self, *card, is_first);
            }
        }
        n
    }

    // Extension point for retrigger (jokers + red seal)
    pub(super) fn trigger_count_held(&mut self, card: &Card) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_held.clone() {
            if let Effects::TriggerCountHeld(f) = e {
                n += f.lock().unwrap()(self, *card, false);
            }
        }
        n
    }

    pub(super) fn calc_score_inner(
        &mut self,
        mut hand: MadeHand,
        mut trace: Option<&mut TraceRecorder>,
    ) -> usize {
        // The Hook and The Tooth trigger when the hand is played, before it scores.
        self.boss_triggered_this_hand = std::mem::take(&mut self.boss_play_triggered);
        // `evaluate_play` records the hand type before the boss can refuse it.
        self.last_hand_played = Some(hand.rank);
        if self.hand_not_allowed(&hand) {
            return self.score_not_allowed(&hand, trace);
        }
        self.remember_mouth_hand(&hand);
        self.jokers_before_scoring(&mut hand);
        // Splash: every played card scores.
        if self.count_jokers(|j| matches!(j, Jokers::Splash(_))) > 0 {
            hand.hand = SelectHand::new(hand.all.clone());
        }
        self.apply_ox(&hand);

        // compute chips and mult from hand level
        let chips_before = self.chips;
        let mult_before = self.mult;
        if self.active_boss() == Some(BossBlind::Arm) && self.planetarium.level(hand.rank).level > 1
        {
            self.planetarium.level_down(hand.rank);
            self.boss_triggered_this_hand = true;
        }
        let level = self.planetarium.play(hand.rank);
        let (level_chips, level_mult) = if self.active_boss() == Some(BossBlind::Flint) {
            self.boss_triggered_this_hand = true;
            // Halved rounding to nearest, Mult at least 1, like `Blind:modify_hand`.
            (level.chips.div_ceil(2), level.mult.div_ceil(2).max(1))
        } else {
            (level.chips, level.mult)
        };
        self.chips += level_chips;
        self.mult += level_mult as f64;
        self.record_step(
            &mut trace,
            ScoreSource::HandLevel(hand.rank),
            chips_before,
            mult_before,
            false,
        );

        // Observatory: every held Planet card whose own hand is the one
        // being scored multiplies Mult by 1.5. Applied straight after the
        // hand level, before any card or joker contribution.
        if self.vouchers.observatory() {
            let matching = self
                .consumables
                .iter()
                .filter(|c| matches!(c, Consumable::Planet(p) if p.hand_rank() == hand.rank))
                .count();
            for _ in 0..matching {
                let chips_before = self.chips;
                let mult_before = self.mult;
                self.mult *= 1.5;
                self.note_xmult(1.5);
                self.record_step(
                    &mut trace,
                    ScoreSource::Voucher(Voucher::Observatory),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }

        // first card in original play order that's actually used in scoring:
        // either part of the made hand, or a stone kicker (which always scores).
        // ordinary kickers that never score don't count, even if played first.
        let hand_ids: Vec<usize> = hand.hand.cards().iter().map(|c| c.id).collect();
        let first_played_id = hand
            .all
            .iter()
            .find(|c| hand_ids.contains(&c.id) || c.enhancement == Some(Enhancement::Stone))
            .map(|c| c.id);

        // per-scored-card, rank chips, enhancements, editions
        for card in hand.hand.cards() {
            if self.is_card_debuffed(&card) {
                self.boss_triggered_this_hand = true;
                continue;
            }
            let is_first = Some(card.id) == first_played_id;
            for trig in 0..self.trigger_count_played(&card, is_first) {
                let chips_before = self.chips;
                let mult_before = self.mult;

                // stone has no rank, skip normal chip value
                if card.enhancement != Some(Enhancement::Stone) {
                    self.chips += card.chips();
                }
                // `get_chip_bonus` adds the permanent bonus, Stone cards included.
                self.chips += card.perma_bonus;
                match card.enhancement {
                    Some(Enhancement::Bonus) => self.chips += 30,
                    Some(Enhancement::Mult) => self.mult += 4.0,
                    Some(Enhancement::Glass) => {
                        self.mult *= 2.0;
                        self.note_xmult(2.0);
                    }
                    Some(Enhancement::Lucky) => {
                        let mult = self.prob_roll(1, 5);
                        if mult {
                            self.mult += 20.0;
                        }
                        let money = self.prob_roll(1, 15);
                        if money {
                            self.money += 20;
                        }
                        // `lucky_trigger`, read by Lucky Cat.
                        self.lucky_trigger = (mult || money).then_some(card.id);
                    }
                    _ => {}
                }
                match card.edition {
                    Edition::Foil => self.chips += 50,
                    Edition::Holographic => self.mult += 10.0,
                    Edition::Polychrome => {
                        self.mult *= 1.5;
                        self.note_xmult(1.5);
                    }
                    _ => {}
                }
                if card.seal == Some(Seal::Gold) {
                    self.money += 3;
                }

                self.record_step(
                    &mut trace,
                    ScoreSource::PlayedCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
                self.joker_card_effects(&mut trace, card, &hand, false, trig > 0);
            }
        }

        // stone cards always score +50 chips, even as kickers outside the played
        // hand's own scoring subset.
        // still subject to the same retrigger rules as any other played card.
        for card in &hand.all {
            if card.enhancement == Some(Enhancement::Stone) {
                if self.is_card_debuffed(card) {
                    self.boss_triggered_this_hand = true;
                    continue;
                }
                let is_first = Some(card.id) == first_played_id;
                for trig in 0..self.trigger_count_played(card, is_first) {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    self.chips += 50;
                    self.record_step(
                        &mut trace,
                        ScoreSource::StoneKicker(*card),
                        chips_before,
                        mult_before,
                        trig > 0,
                    );
                }
            }
        }

        // held in hand effects (cards not played this hand)
        for card in self.available.not_selected() {
            if self.is_card_debuffed(&card) {
                // Held-card Matador trigger unconfirmed, real-game check needed.
                if matches!(
                    card.enhancement,
                    Some(Enhancement::Steel) | Some(Enhancement::Gold)
                ) {
                    self.boss_triggered_this_hand = true;
                }
                continue;
            }
            for trig in 0..self.trigger_count_held(&card) {
                let chips_before = self.chips;
                let mult_before = self.mult;
                // A Gold card pays at the end of the round (`handle_round_end`).
                if card.enhancement == Some(Enhancement::Steel) {
                    self.mult *= 1.5;
                    self.note_xmult(1.5);
                }
                self.record_step(
                    &mut trace,
                    ScoreSource::HeldCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
                self.joker_card_effects(&mut trace, card, &hand, true, trig > 0);
            }
        }

        // run hand modifiers (e.g. Pareidolia) before joker scoring effects
        for e in self.effect_registry.on_modify_hand.clone() {
            if let Effects::OnModifyHand(f) = e {
                f.lock().unwrap()(self, &mut hand)
            }
        }

        // apply joker effects with per-joker edition ordering.
        // effects recomputed per joker, not read from EffectRegistry. Each joker's edition
        // bonus must sandwich its own OnScore closures in self.jokers order.
        for joker in self.jokers.clone() {
            let chips_before = self.chips;
            let mult_before = self.mult;
            match joker.edition() {
                Edition::Foil => self.chips += 50,
                Edition::Holographic => self.mult += 10.0,
                _ => {}
            }
            self.record_step(
                &mut trace,
                ScoreSource::Joker(joker.clone()),
                chips_before,
                mult_before,
                false,
            );

            for e in joker.effects(self) {
                if let Effects::OnScore(f) = e {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    let grown_from = self.joker_counter(&joker);
                    let bus_from = self.consecutive_hands_without_face_card;
                    f.lock().unwrap()(self, hand.clone());
                    // The message comes before the value it feeds.
                    let counter = self.joker_counter(&joker);
                    let reset = match joker {
                        Jokers::RideTheBus(_) => {
                            bus_from > 0 && self.consecutive_hands_without_face_card == 0
                        }
                        Jokers::Obelisk(_) => grown_from > 0.0 && counter == 0.0,
                        _ => false,
                    };
                    if counter > grown_from {
                        self.record_message(&mut trace, ScoreSource::JokerUpgrade(joker.clone()));
                    } else if reset {
                        self.record_message(&mut trace, ScoreSource::JokerReset(joker.clone()));
                    }
                    self.record_step(
                        &mut trace,
                        ScoreSource::Joker(joker.clone()),
                        chips_before,
                        mult_before,
                        false,
                    );
                }
            }

            if joker.edition() == Edition::Polychrome {
                let chips_before = self.chips;
                let mult_before = self.mult;
                self.mult *= 1.5;
                self.note_xmult(1.5);
                self.record_step(
                    &mut trace,
                    ScoreSource::Joker(joker.clone()),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }

        // record after joker loop, not before (for CardSharp)
        self.hand_ranks_played_this_round.insert(hand.rank);

        // compute score
        let score = (self.chips as f64 * self.mult).floor() as usize;

        // reset chips and mult
        self.mult = self.config.base_mult as f64;
        self.chips = self.config.base_chips;

        score
    }

    pub fn required_score(&self) -> usize {
        self.required_score_for(self.blind)
    }

    pub fn required_score_for(&self, blind: Option<Blind>) -> usize {
        let base = self.ante_current.base();
        match blind {
            None => base,
            Some(Blind::Small) => base,
            Some(Blind::Big) => base + base / 2,
            Some(Blind::Boss) => {
                let boss = if self.blind == blind {
                    self.active_boss()
                } else {
                    self.current_boss
                };
                base * Self::boss_score_multiplier(boss)
            }
        }
    }

    /// The score the upcoming Boss Blind will require, based on
    /// `current_boss`, valid before boss blind is active.
    pub fn boss_required_score(&self) -> usize {
        self.ante_current.base() * Self::boss_score_multiplier(self.current_boss)
    }

    pub(super) fn boss_score_multiplier(boss: Option<BossBlind>) -> usize {
        match boss {
            Some(BossBlind::Wall) => 4,
            Some(BossBlind::VioletVessel) => 6,
            Some(BossBlind::Needle) => 1,
            _ => 2,
        }
    }

    /// The rows of `G.FUNCS.evaluate_round` for a cleared `blind`; tags are added by the caller.
    pub(crate) fn calc_reward(
        &mut self,
        blind: Blind,
    ) -> Result<crate::reward::RoundReward, GameError> {
        // `min(floor($ / 5), cap / 5) × interest_amount`; To the Moon adds 1 to the amount.
        let chunks = ((self.money as f32 * self.config.interest_rate).floor() as usize)
            .min(self.interest_max());
        let moons = self
            .jokers
            .iter()
            .filter(|joker| matches!(joker, Jokers::ToTheMoon(_)))
            .count();
        let interest = chunks * (1 + moons);
        Ok(crate::reward::RoundReward {
            blind: blind.reward(),
            hands_left: self.plays,
            hand_dollars: self.plays * self.config.money_per_hand,
            jokers: self
                .jokers
                .iter()
                .filter_map(|joker| {
                    crate::reward::dollar_bonus(joker, self).map(|d| (joker.clone(), d))
                })
                .collect(),
            tags: Vec::new(),
            interest,
        })
    }
}
