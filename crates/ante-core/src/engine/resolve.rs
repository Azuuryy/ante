use super::*;

impl EngineRun {
    /// Applies a Dev shortcut outside the score trace, like the original debug panel.
    pub(super) fn apply_debug(&mut self, command: DebugCommand) -> Result<Outcome, RunError> {
        let game = &mut self.game;
        let phase = phase_from(game.stage);
        let playing = phase == Phase::Playing;
        let simulation =
            |error: balatro_rs::error::GameError| RunError::Simulation(error.to_string());
        match command {
            DebugCommand::AddMoney(amount) => game.money += amount,
            DebugCommand::AddHand if playing => game.plays += 1,
            DebugCommand::AddDiscard if playing => game.discards_remaining += 1,
            DebugCommand::AddRound => game.round += 1,
            DebugCommand::AddAnte if !playing && !matches!(phase, Phase::Won | Phase::Lost) => {
                game.debug_next_ante().map_err(simulation)?;
                self.skipped_blinds.clear();
            }
            DebugCommand::AddScore(amount) if playing => {
                game.debug_add_score(amount).map_err(simulation)?;
            }
            DebugCommand::WinBlind if playing => {
                let missing = game.required_score().saturating_sub(game.score);
                game.debug_add_score(missing).map_err(simulation)?;
            }
            DebugCommand::LoseRun if !matches!(phase, Phase::Won | Phase::Lost) => {
                game.stage = Stage::End(End::Lose);
            }
            _ => return Err(RunError::Unavailable),
        }
        self.debug = true;
        self.revision = self.revision.wrapping_add(1);
        let phase = phase_from(game.stage);
        Ok(Outcome {
            phase,
            round_score: visible_score(game, phase),
            campaign_won: game.campaign_won,
            hand_score: self.last_hand_score,
            trace: None,
            trace_time: Duration::ZERO,
            discovered_item: None,
        })
    }

    /// The shop's "Buy & Use": pays and applies a consumable without taking a slot.
    pub(super) fn buy_and_use(&mut self, index: usize, revision: u64) -> Result<Outcome, RunError> {
        if revision != self.revision {
            return Err(RunError::Unavailable);
        }
        let game = &mut self.game;
        let card = *game
            .shop
            .consumables
            .get(index)
            .ok_or(RunError::Unavailable)?;
        game.buy_and_use_consumable(card)
            .map_err(|_| RunError::Unavailable)?;
        self.revision = self.revision.wrapping_add(1);
        let phase = phase_from(game.stage);
        Ok(Outcome {
            phase,
            round_score: visible_score(game, phase),
            campaign_won: game.campaign_won,
            hand_score: self.last_hand_score,
            trace: None,
            trace_time: Duration::ZERO,
            discovered_item: Some(card.id()),
        })
    }

    pub(super) fn resolve(
        &mut self,
        action: Command,
        profile_trace: Option<bool>,
    ) -> Result<Outcome, RunError> {
        if let Command::Debug(command) = action {
            return self.apply_debug(command);
        }
        if let Command::BuyAndUseConsumable { index, revision } = action {
            return self.buy_and_use(index, revision);
        }
        let game = &mut self.game;
        if action == Command::ContinueEndless {
            game.continue_endless().map_err(|_| RunError::Unavailable)?;
            self.skipped_blinds.clear();
            self.revision = self.revision.wrapping_add(1);
            return Ok(Outcome {
                phase: Phase::Reward,
                round_score: game.last_score,
                campaign_won: game.campaign_won,
                hand_score: self.last_hand_score,
                trace: None,
                trace_time: Duration::ZERO,
                discovered_item: None,
            });
        }
        match action {
            Command::MoveCard { id, direction }
                if matches!(phase_from(game.stage), Phase::Playing | Phase::Pack) =>
            {
                let card = game
                    .available
                    .cards()
                    .into_iter()
                    .find(|card| card.id == id.0)
                    .ok_or(RunError::Unavailable)?;
                game.handle_action(Action::MoveCard(
                    match direction {
                        MoveDirection::Left => EngineMoveDirection::Left,
                        MoveDirection::Right => EngineMoveDirection::Right,
                    },
                    card,
                ))
                .map_err(|error| RunError::Simulation(error.to_string()))?;
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(Outcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            Command::ReorderJoker { from, to, revision }
                if revision == self.revision
                    && from < game.jokers.len()
                    && to < game.jokers.len() =>
            {
                let item = game.jokers.remove(from);
                game.jokers.insert(to, item);
                let jokers = game.jokers.clone();
                game.effect_registry.register_jokers(jokers, &game.clone());
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(Outcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            Command::ReorderConsumable { from, to, revision }
                if revision == self.revision
                    && from < game.consumables.len()
                    && to < game.consumables.len() =>
            {
                game.move_consumable(from, to);
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(Outcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            Command::ReorderJoker { .. } | Command::ReorderConsumable { .. } => {
                return Err(RunError::Unavailable);
            }
            _ => {}
        }
        let ante_before = game.ante_current;
        let selected_count = game.available.selected().len();
        let indexed = match action {
            Command::SkipBlind { revision } | Command::RedeemVoucher { revision, .. } => {
                Some(revision)
            }
            Command::BuyPlayingCard { revision, .. }
            | Command::BuyPack { revision, .. }
            | Command::PickPack { revision, .. }
            | Command::BuyConsumable { revision, .. }
            | Command::UseConsumable { revision, .. }
            | Command::SellConsumable { revision, .. }
            | Command::SellJoker { revision, .. } => Some(revision),
            _ => None,
        };
        if indexed.is_some_and(|revision| revision != self.revision) {
            return Err(RunError::Unavailable);
        }
        let native = game
            .gen_actions()
            .find(|candidate| match (action, candidate) {
                (Command::StartBlind, Action::SelectBlind(_))
                | (Command::SkipBlind { .. }, Action::SkipBlind(_))
                | (Command::SortRank, Action::SortHand(SortBy::Rank))
                | (Command::SortSuit, Action::SortHand(SortBy::Suit))
                | (Command::Play, Action::Play())
                | (Command::Discard, Action::Discard())
                | (Command::CashOut, Action::CashOut(_))
                | (Command::LeaveShop, Action::NextRound())
                | (Command::RerollBoss, Action::RerollBoss()) => true,
                (Command::RedeemVoucher { voucher, .. }, Action::BuyVoucher(offered)) => {
                    voucher.0 == offered.id()
                }
                (
                    Command::ToggleCard(id),
                    Action::SelectCard(card) | Action::DeselectCard(card),
                ) => id.0 == card.id,
                (Command::BuyPlayingCard { id, .. }, Action::BuyPlayingCard(card)) => {
                    id.0 == card.id
                }
                (Command::BuyPack { index, .. }, Action::BuyPack(pack)) => {
                    game.shop.packs.get(index) == Some(pack)
                }
                (Command::PickPack { index, .. }, Action::PickPackCard(content)) => {
                    game.open_pack
                        .as_ref()
                        .and_then(|pack| pack.contents.get(index))
                        == Some(content)
                }
                (Command::SkipPack, Action::SkipPack()) => true,
                (Command::BuyJoker(id), Action::BuyJoker(joker)) => id.0 == joker.instance_id(),
                (Command::BuyConsumable { index, .. }, Action::BuyConsumable(c)) => {
                    game.shop.consumables.get(index) == Some(c)
                }
                (Command::UseConsumable { index, .. }, Action::UseConsumable(c)) => {
                    game.consumables.get(index) == Some(c)
                }
                (Command::SellConsumable { index, .. }, Action::SellConsumable(i)) => index == *i,
                (Command::SellJoker { index, .. }, Action::SellJoker(i)) => index == *i,
                (Command::Reroll, Action::Reroll()) => true,
                _ => false,
            })
            .ok_or(RunError::Unavailable)?;
        if action == Command::SortRank {
            self.sort_by_suit = false;
        } else if action == Command::SortSuit {
            self.sort_by_suit = true;
        }
        if action == Command::Play {
            let selected = game.available.selected();
            let sorted_ids = game
                .available
                .cards()
                .into_iter()
                .filter(|card| selected.contains(card))
                .map(|card| card.id)
                .collect::<Vec<_>>();
            for card in selected {
                game.handle_action(Action::DeselectCard(card))
                    .map_err(|error| RunError::Simulation(error.to_string()))?;
            }
            for id in sorted_ids {
                let card = game
                    .available
                    .cards()
                    .into_iter()
                    .find(|card| card.id == id)
                    .ok_or(RunError::Unavailable)?;
                game.handle_action(Action::SelectCard(card))
                    .map_err(|error| RunError::Simulation(error.to_string()))?;
            }
        }
        let skipped = if let Action::SkipBlind(blind) = native {
            Some(blind)
        } else {
            None
        };
        let before = game.score;
        let discovered_item = match &native {
            Action::BuyJoker(joker) | Action::PickPackCard(PackContent::Joker(joker)) => {
                Some(joker.id())
            }
            Action::BuyVoucher(voucher) => Some(voucher.id()),
            Action::SkipBlind(blind) => game.skip_tag(*blind).map(|tag| tag.id()),
            Action::BuyConsumable(card) | Action::UseConsumable(card) => Some(card.id()),
            Action::PickPackCard(PackContent::Tarot(card)) => Some(card.id()),
            Action::PickPackCard(PackContent::Planet(card)) => Some(card.id()),
            Action::PickPackCard(PackContent::Spectral(card)) => Some(card.id()),
            Action::BuyPack(pack) => match pack_art(pack) {
                ShopArt::Pack { family, size } => Some(crate::contract::booster_id(family, size)),
                _ => None,
            },
            Action::SelectBlind(balatro_rs::stage::Blind::Boss) => {
                game.current_boss.map(|boss| boss.id())
            }
            _ => None,
        };
        let (trace, trace_time) = if let (Command::Play, Some(profile)) = (action, profile_trace) {
            let (trace, elapsed) = game
                .handle_action_traced(native, profile)
                .map_err(|error| RunError::Simulation(error.to_string()))?;
            (Some(map_trace(trace)), elapsed)
        } else {
            game.handle_action(native)
                .map_err(|error| RunError::Simulation(error.to_string()))?;
            (None, Duration::ZERO)
        };

        if matches!(
            action,
            Command::StartBlind | Command::Play | Command::Discard
        ) && phase_from(game.stage) == Phase::Playing
            && game
                .gen_actions()
                .any(|candidate| matches!(candidate, Action::SortHand(_)))
        {
            let sort = if self.sort_by_suit {
                Action::SortHand(SortBy::Suit)
            } else {
                Action::SortHand(SortBy::Rank)
            };
            game.handle_action(sort)
                .map_err(|error| RunError::Simulation(error.to_string()))?;
        }

        if ante_before != game.ante_current {
            self.skipped_blinds.clear();
        }
        if let Some(blind) = skipped {
            self.skipped_blinds.push(blind);
        }
        self.revision = self.revision.wrapping_add(1);
        match action {
            Command::StartBlind => self.last_hand_score = None,
            Command::Play => {
                self.stats.played += selected_count;
                let after = if matches!(game.stage, Stage::PostBlind() | Stage::End(End::Win)) {
                    game.last_score
                } else {
                    game.score
                };
                let hand = after.saturating_sub(before);
                self.last_hand_score = Some(hand);
                self.stats.best_hand = self.stats.best_hand.max(hand);
            }
            Command::Discard => self.stats.discarded += selected_count,
            Command::BuyJoker(_)
            | Command::BuyConsumable { .. }
            | Command::RedeemVoucher { .. } => self.stats.purchased += 1,
            Command::Reroll => self.stats.rerolls += 1,
            _ => {}
        }
        let phase = phase_from(game.stage);
        Ok(Outcome {
            phase,
            round_score: visible_score(game, phase),
            campaign_won: game.campaign_won,
            hand_score: self.last_hand_score,
            trace,
            trace_time,
            discovered_item,
        })
    }
}
