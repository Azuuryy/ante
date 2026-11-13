use super::*;

impl ActionSpace {
    // Not all actions are always legal, by default all actions
    // are masked out, but provide methods to unmask valid.
    pub(crate) fn unmask_select_card(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.select_card.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.select_card[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_move_card_left(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_left.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_left[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_move_card_right(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.move_card_right.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.move_card_right[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_play(&mut self) {
        self.play[0] = 1;
    }

    pub(crate) fn unmask_discard(&mut self) {
        self.discard[0] = 1;
    }

    pub(crate) fn unmask_cash_out(&mut self) {
        self.cash_out[0] = 1;
    }

    pub(crate) fn unmask_buy_joker(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_joker.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_joker[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_next_round(&mut self) {
        self.next_round[0] = 1;
    }

    pub(crate) fn unmask_select_blind(&mut self) {
        self.select_blind[0] = 1;
    }

    pub(crate) fn unmask_skip_blind(&mut self) {
        self.skip_blind[0] = 1;
    }

    pub(crate) fn unmask_buy_consumable(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_consumable.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_consumable[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_buy_voucher(&mut self) {
        self.buy_voucher[0] = 1;
    }

    pub(crate) fn unmask_buy_playing_card(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_playing_card.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_playing_card[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_use_consumable(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.use_consumable.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.use_consumable[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_apply_tarot(&mut self) {
        self.apply_tarot[0] = 1;
    }

    pub(crate) fn unmask_sell_joker(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.sell_joker.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.sell_joker[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_sell_consumable(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.sell_consumable.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.sell_consumable[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_buy_pack(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.buy_pack.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.buy_pack[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_pick_pack_card(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.pick_pack_card.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.pick_pack_card[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_skip_pack(&mut self) {
        self.skip_pack[0] = 1;
    }

    pub(crate) fn unmask_sort_hand(&mut self, i: usize) -> Result<(), ActionSpaceError> {
        if i >= self.sort_hand.len() {
            return Err(ActionSpaceError::InvalidIndex);
        }
        self.sort_hand[i] = 1;
        Ok(())
    }

    pub(crate) fn unmask_reroll(&mut self) {
        self.reroll[0] = 1;
    }

    pub(crate) fn unmask_apply_spectral(&mut self) {
        self.apply_spectral[0] = 1;
    }
}
