use super::*;

impl Game {
    pub(super) fn select_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        // can only set blind if stage is pre blind
        if self.stage != Stage::PreBlind() {
            return Err(GameError::InvalidStage);
        }
        // provided blind must be expected next blind
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else {
            // if game just started, blind will be None, in which case
            // we can only set it to small.
            if blind != Blind::Small {
                return Err(GameError::InvalidBlind);
            }
        }
        self.blind = Some(blind);
        self.stage = Stage::Blind(blind);
        // Per-round resources are set here, at the start of the round, not
        // only in `clear_blind` at the end of the previous one — anything
        // bought in the shop in between (Grabber, Wasteful, Hieroglyph...)
        // has to count for the round it was bought for.
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
        while self.take_tag(Tag::Juggle) {
            self.temporary_hand_size += 3;
        }
        self.deal();
        Ok(())
    }

    pub(super) fn skip_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        if self.stage != Stage::PreBlind() {
            return Err(GameError::InvalidStage);
        }
        if blind == Blind::Boss {
            return Err(GameError::InvalidBlind);
        }
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else if blind != Blind::Small {
            return Err(GameError::InvalidBlind);
        }
        self.blind = Some(blind);
        self.stage = Stage::PreBlind();
        let tag = self
            .skip_tag(blind)
            .expect("blind is not Boss, checked above");
        self.skipped_count += 1;
        let orbital = self.orbital_targets[usize::from(blind == Blind::Big)];
        self.acquire_tag(tag, orbital);
        self.resolve_blind_choice_tags();
        Ok(())
    }

    pub(super) fn draw_ante_tags(&mut self) {
        let (small, big) = self.backend.draw_ante_tags();
        self.small_blind_tag = small;
        self.big_blind_tag = big;
        self.orbital_targets = [small, big].map(|tag| {
            if tag != Tag::Orbital {
                return None;
            }
            match self.backend.roll_random_planet(&self.planetarium, &[]) {
                Consumable::Planet(planet) => Some(planet.hand_rank()),
                _ => None,
            }
        });
    }

    pub(super) fn draw_ante_boss(&mut self) {
        self.current_boss = Some(self.backend.draw_boss(self.ante_current.number() as i32));
    }

    pub fn skip_tag(&self, blind: Blind) -> Option<Tag> {
        match blind {
            Blind::Small => Some(self.small_blind_tag),
            Blind::Big => Some(self.big_blind_tag),
            Blind::Boss => None,
        }
    }

    pub(super) fn next_round(&mut self) -> Result<(), GameError> {
        self.stage = Stage::PreBlind();
        self.round += 1;
        self.resolve_blind_choice_tags();
        Ok(())
    }

    // Returns true if should clear blind after, false if not.
    pub(super) fn handle_score(&mut self, score: usize) -> Result<bool, GameError> {
        // can only handle score if stage is blind
        if !self.stage.is_blind() {
            return Err(GameError::InvalidStage);
        }

        self.score += score;
        let required = self.required_score();

        // blind not passed
        if self.score < required {
            // no more hands to play -> lose
            if self.plays == 0 {
                self.stage = Stage::End(End::Lose);
                return Ok(false);
            } else {
                // more hands to play, carry on
                return Ok(false);
            }
        }

        let blind = self.blind.expect("stage is blind");
        self.last_score = self.score;

        // score exceeds blind (blind passed).
        // handle reward then progress to next stage.
        let reward = self.calc_reward(blind)?;
        self.unused_discards += self.discards_remaining;
        self.reward = reward;
        if blind == Blind::Boss {
            while self.take_tag(Tag::Investment) {
                self.reward += 25;
            }
        }

        if blind == Blind::Boss {
            let next = if self.endless_mode {
                self.ante_current.next_supported()
            } else {
                self.ante_current.next(self.ante_end)
            };
            if let Some(ante) = next {
                self.advance_ante(ante);
            } else {
                self.campaign_won |= self.ante_current == Ante::Eight;
                self.stage = Stage::End(End::Win);
                return Ok(true);
            }
        }

        // finish blind, proceed to post blind
        self.stage = Stage::PostBlind();
        Ok(true)
    }

    pub(super) fn advance_ante(&mut self, ante: Ante) {
        self.ante_current = ante;
        self.boss_rerolled = false;
        self.blind = None;
        self.draw_ante_tags();
        self.draw_ante_boss();
    }

    pub fn can_continue_endless(&self) -> bool {
        self.stage == Stage::End(End::Win)
            && self.campaign_won
            && !self.endless_mode
            && self.ante_current == Ante::Eight
            && self.ante_current.next_supported().is_some()
    }

    /// Debug: scores `amount` as a played hand would, clearing the blind when reached.
    pub fn debug_add_score(&mut self, amount: usize) -> Result<(), GameError> {
        if self.handle_score(amount)? {
            self.handle_round_end();
            self.clear_blind();
        }
        Ok(())
    }

    /// Debug: moves to the next ante and draws its tags and boss.
    pub fn debug_next_ante(&mut self) -> Result<(), GameError> {
        let next = self
            .ante_current
            .next_supported()
            .ok_or(GameError::InvalidAction)?;
        self.advance_ante(next);
        Ok(())
    }
    pub fn continue_endless(&mut self) -> Result<(), GameError> {
        if !self.can_continue_endless() {
            return Err(GameError::InvalidAction);
        }
        let next = self
            .ante_current
            .next_supported()
            .ok_or(GameError::InvalidAction)?;
        self.endless_mode = true;
        self.advance_ante(next);
        self.stage = Stage::PostBlind();
        Ok(())
    }
}
