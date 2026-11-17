use super::*;

impl Game {
    pub fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        self.handle_action_inner(action, None)
    }

    pub fn handle_action_traced(
        &mut self,
        action: Action,
        profile_trace: bool,
    ) -> Result<(ScoreTrace, std::time::Duration), GameError> {
        let mut recorder = TraceRecorder {
            profile: profile_trace,
            ..TraceRecorder::default()
        };
        self.handle_action_inner(action, Some(&mut recorder))?;
        Ok((recorder.trace, recorder.elapsed))
    }

    pub(super) fn handle_action_inner(
        &mut self,
        action: Action,
        trace: Option<&mut TraceRecorder>,
    ) -> Result<(), GameError> {
        self.action_history.push(action.clone());
        self.settle_debt();
        match action {
            Action::SelectCard(card) => {
                if self.stage.is_blind() {
                    self.select_card(card)
                } else if self.hand_selectable() {
                    // A pack's hand keeps the hand's highlight limit.
                    if self.available.selected().len() >= self.config.selected_max {
                        return Err(GameError::InvalidAction);
                    }
                    self.select_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::DeselectCard(card) => {
                if self.hand_selectable() {
                    self.available.deselect_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::Play() => match self.stage.is_blind() {
                true => self.play_selected_inner(trace),
                false => Err(GameError::InvalidAction),
            },
            Action::Discard() => match self.stage.is_blind() {
                true => self.discard_selected(),
                false => Err(GameError::InvalidAction),
            },
            Action::MoveCard(dir, card) => {
                if self.hand_selectable() {
                    self.move_card(dir, card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::CashOut(_reward) => match self.stage {
                Stage::PostBlind() => self.cashout(),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyJoker(joker) => match self.stage {
                Stage::Shop() => self.buy_joker(joker),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyConsumable(consumable) => match self.stage {
                Stage::Shop() => self.buy_consumable(consumable),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyVoucher(voucher) => match self.stage {
                Stage::Shop() => self.buy_voucher(voucher),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyPlayingCard(card) => match self.stage {
                Stage::Shop() => self.buy_playing_card(card),
                _ => Err(GameError::InvalidAction),
            },
            Action::UseConsumable(consumable) => self.use_consumable(consumable),
            Action::NextRound() => match self.stage {
                Stage::Shop() => self.next_round(),
                _ => Err(GameError::InvalidAction),
            },
            Action::SelectBlind(blind) => match self.stage {
                Stage::PreBlind() => self.select_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::SkipBlind(blind) => match self.stage {
                Stage::PreBlind() => self.skip_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::SellJoker(idx) => self.sell_joker(idx),
            Action::SellConsumable(idx) => self.sell_consumable(idx),
            Action::BuyPack(pack) => match self.stage {
                Stage::Shop() => self.buy_pack(pack),
                _ => Err(GameError::InvalidStage),
            },
            Action::PickPackCard(content) => match self.stage {
                Stage::PackOpen() => self.pick_pack_card(content),
                _ => Err(GameError::InvalidStage),
            },
            Action::SortHand(sort_by) => {
                if self.stage.is_blind() {
                    self.available.sort(sort_by);
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::SkipPack() => self.skip_pack(),
            Action::RerollBoss() => {
                if !self.boss_reroll_available() {
                    return Err(GameError::InvalidAction);
                }
                self.pay(10);
                self.replace_boss();
                Ok(())
            }
            Action::Reroll() => match self.stage {
                Stage::Shop() => self.reroll(),
                _ => Err(GameError::InvalidAction),
            },
        }
    }

    pub fn handle_action_index(&mut self, index: usize) -> Result<(), GameError> {
        let space = self.gen_action_space();
        let action = space.to_action(index, self)?;
        self.handle_action(action)
    }
}
