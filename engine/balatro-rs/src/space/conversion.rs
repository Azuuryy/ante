use super::*;

impl ActionSpace {
    pub fn to_action(&self, index: usize, game: &Game) -> Result<Action, ActionSpaceError> {
        let vec = self.to_vec();
        if let Some(v) = vec.get(index) {
            if *v == 0 {
                return Err(ActionSpaceError::MaskedAction);
            }
        } else {
            return Err(ActionSpaceError::InvalidIndex);
        }
        match index {
            // Cannot reference runtime values in patterns, so this is workaround
            n if (self.select_card_min()..=self.select_card_max()).contains(&n) => game
                .available
                .card_from_index(index)
                .map(Action::SelectCard)
                .ok_or(ActionSpaceError::InvalidActionConversion),
            n if (self.move_card_left_min()..=self.move_card_left_max()).contains(&n) => {
                // Index shifted to right (+1), since leftmost card cannot move left
                let n_offset = n - self.move_card_left_min() + 1;
                game.available
                    .card_from_index(n_offset)
                    .map(|card| Action::MoveCard(MoveDirection::Left, card))
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.move_card_right_min()..=self.move_card_right_max()).contains(&n) => {
                let n_offset = n - self.move_card_right_min();
                game.available
                    .card_from_index(n_offset)
                    .map(|card| Action::MoveCard(MoveDirection::Right, card))
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.play_min()..=self.play_max()).contains(&n) => Ok(Action::Play()),
            n if (self.discard_min()..=self.discard_max()).contains(&n) => Ok(Action::Discard()),
            n if (self.cash_out_min()..=self.cash_out_max()).contains(&n) => {
                Ok(Action::CashOut(game.reward))
            }
            n if (self.buy_joker_min()..=self.buy_joker_max()).contains(&n) => {
                let n_offset = n - self.buy_joker_min();
                game.shop
                    .joker_from_index(n_offset)
                    .map(Action::BuyJoker)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.next_round_min()..=self.next_round_max()).contains(&n) => {
                Ok(Action::NextRound())
            }
            n if (self.select_blind_min()..=self.select_blind_max()).contains(&n) => {
                match game.blind {
                    Some(blind) => Ok(Action::SelectBlind(blind.next())),
                    None => Ok(Action::SelectBlind(Blind::Small)),
                }
            }
            n if (self.skip_blind_min()..=self.skip_blind_max()).contains(&n) => match game.blind {
                Some(blind) => Ok(Action::SkipBlind(blind.next())),
                None => Ok(Action::SkipBlind(Blind::Small)),
            },
            n if (self.buy_consumable_min()..=self.buy_consumable_max()).contains(&n) => {
                let n_offset = n - self.buy_consumable_min();
                game.shop
                    .consumable_from_index(n_offset)
                    .map(Action::BuyConsumable)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if !self.buy_voucher.is_empty()
                && (self.buy_voucher_min()..=self.buy_voucher_max()).contains(&n) =>
            {
                game.shop
                    .voucher
                    .map(Action::BuyVoucher)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if !self.buy_playing_card.is_empty()
                && (self.buy_playing_card_min()..=self.buy_playing_card_max()).contains(&n) =>
            {
                let n_offset = n - self.buy_playing_card_min();
                game.shop
                    .card_from_index(n_offset)
                    .map(Action::BuyPlayingCard)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.use_consumable_min()..=self.use_consumable_max()).contains(&n) => {
                let n_offset = n - self.use_consumable_min();
                game.consumables
                    .get(n_offset)
                    .cloned()
                    .map(Action::UseConsumable)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if (self.apply_tarot_min()..=self.apply_tarot_max()).contains(&n) => {
                Ok(Action::ApplyTarot())
            }
            n if !self.sell_joker.is_empty()
                && (self.sell_joker_min()..=self.sell_joker_max()).contains(&n) =>
            {
                Ok(Action::SellJoker(n - self.sell_joker_min()))
            }
            n if !self.sell_consumable.is_empty()
                && (self.sell_consumable_min()..=self.sell_consumable_max()).contains(&n) =>
            {
                Ok(Action::SellConsumable(n - self.sell_consumable_min()))
            }
            n if !self.buy_pack.is_empty()
                && (self.buy_pack_min()..=self.buy_pack_max()).contains(&n) =>
            {
                let n_offset = n - self.buy_pack_min();
                game.shop
                    .pack_from_index(n_offset)
                    .map(Action::BuyPack)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if !self.pick_pack_card.is_empty()
                && (self.pick_pack_card_min()..=self.pick_pack_card_max()).contains(&n) =>
            {
                let n_offset = n - self.pick_pack_card_min();
                game.open_pack
                    .as_ref()
                    .and_then(|s| s.contents.get(n_offset))
                    .cloned()
                    .map(Action::PickPackCard)
                    .ok_or(ActionSpaceError::InvalidActionConversion)
            }
            n if !self.skip_pack.is_empty()
                && (self.skip_pack_min()..=self.skip_pack_max()).contains(&n) =>
            {
                Ok(Action::SkipPack())
            }
            n if !self.sort_hand.is_empty()
                && (self.sort_hand_min()..=self.sort_hand_max()).contains(&n) =>
            {
                match n - self.sort_hand_min() {
                    0 => Ok(Action::SortHand(SortBy::Rank)),
                    _ => Ok(Action::SortHand(SortBy::Suit)),
                }
            }
            n if !self.reroll.is_empty()
                && (self.reroll_min()..=self.reroll_max()).contains(&n) =>
            {
                Ok(Action::Reroll())
            }
            n if !self.apply_spectral.is_empty()
                && (self.apply_spectral_min()..=self.apply_spectral_max()).contains(&n) =>
            {
                Ok(Action::ApplySpectral())
            }
            _ => Err(ActionSpaceError::InvalidActionConversion),
        }
    }

    /// Gym index for `action` in this space, before the action is applied.
    /// Masked slots are skipped. `None` means `action` is not in the 0–N space
    /// (for example `Action::DeselectCard`, which `gen_actions` can still emit).
    pub fn index_of(&self, action: &Action, game: &Game) -> Option<usize> {
        let mask = self.to_vec();
        (0..self.size()).find(|&index| {
            mask.get(index).copied() == Some(1)
                && self.to_action(index, game).ok().as_ref() == Some(action)
        })
    }

    pub fn to_vec(&self) -> Vec<usize> {
        [
            self.select_card.clone(),
            self.move_card_left.clone(),
            self.move_card_right.clone(),
            self.play.clone(),
            self.discard.clone(),
            self.cash_out.clone(),
            self.buy_joker.clone(),
            self.next_round.clone(),
            self.select_blind.clone(),
            self.skip_blind.clone(),
            self.buy_consumable.clone(),
            self.buy_voucher.clone(),
            self.buy_playing_card.clone(),
            self.use_consumable.clone(),
            self.apply_tarot.clone(),
            self.sell_joker.clone(),
            self.sell_consumable.clone(),
            self.buy_pack.clone(),
            self.pick_pack_card.clone(),
            self.skip_pack.clone(),
            self.sort_hand.clone(),
            self.reroll.clone(),
            self.apply_spectral.clone(),
        ]
        .concat()
    }

    // True is all elements are masked
    pub fn is_empty(&self) -> bool {
        let vec = self.to_vec();
        (*vec.iter().min().unwrap() == 0) && (*vec.iter().max().unwrap() == 0)
    }
}
