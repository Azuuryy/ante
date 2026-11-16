use super::*;

impl Game {
    pub(super) fn unmask_action_space_select_cards(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // Cannot select more if max already selected
        if self.available.selected().len() >= self.config.selected_max {
            return;
        }
        self.available
            .cards_and_selected()
            .iter()
            .enumerate()
            .filter(|(_, (_, a))| !*a)
            .for_each(|(i, _)| {
                space
                    .unmask_select_card(i)
                    .expect("valid index for selecting");
            });
    }

    pub(super) fn unmask_action_space_play_and_discard(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // Cannot play/discard if no cards selected
        if self.available.selected().is_empty() {
            return;
        }
        // Can only play/discard is have remaining
        if self.plays != 0 {
            space.unmask_play();
        }
        if self.discards() != 0 {
            space.unmask_discard();
        }
    }

    pub(super) fn unmask_action_space_move_cards(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        // move left
        // every available card except the first can move left
        self.available
            .cards()
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_left(i)
                    .expect("valid index for move left")
            });
        // move right
        // every available card except the last can move right
        self.available
            .cards()
            .iter()
            .rev()
            .skip(1)
            .rev()
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_right(i)
                    .expect("valid index for move right")
            });
    }

    pub(super) fn unmask_action_space_cash_out(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PostBlind() {
            return;
        }
        space.unmask_cash_out();
    }

    pub(super) fn unmask_action_space_next_round(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        space.unmask_next_round();
    }

    pub(super) fn unmask_action_space_select_blind(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PreBlind() {
            return;
        }
        space.unmask_select_blind();
    }

    pub(super) fn unmask_action_space_skip_blind(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PreBlind() {
            return;
        }
        let next_blind = self.blind.map(|b| b.next()).unwrap_or(Blind::Small);
        if next_blind == Blind::Boss {
            return;
        }
        space.unmask_skip_blind();
    }

    pub(super) fn unmask_action_space_buy_joker(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        self.shop
            .jokers
            .iter()
            .enumerate()
            .filter(|(_i, j)| {
                self.can_add_joker(j) && self.shop.joker_price(j, &self.vouchers) <= self.money
            })
            .for_each(|(i, _j)| {
                space
                    .unmask_buy_joker(i)
                    .expect("valid index for buy joker")
            });
    }

    pub(super) fn unmask_action_space_buy_consumable(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.consumables.len() >= self.consumable_slots() {
            return;
        }
        self.shop
            .consumables
            .iter()
            .enumerate()
            .filter(|(_i, c)| self.shop.consumable_price(c, &self.vouchers) <= self.money)
            .for_each(|(i, _c)| {
                space
                    .unmask_buy_consumable(i)
                    .expect("valid index for buy consumable")
            });
    }

    pub(super) fn unmask_action_space_use_consumable(&self, space: &mut ActionSpace) {
        // Same rules as gen_actions_use_consumable.
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.consumables.iter().enumerate().for_each(|(i, c)| {
            if self.consumable_usable(c) {
                space
                    .unmask_use_consumable(i)
                    .expect("valid index for use consumable");
            }
        });
    }

    pub(super) fn unmask_action_space_sell_joker(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.jokers.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_joker(i)
                .expect("valid index for sell joker");
        });
    }

    pub(super) fn unmask_action_space_sell_consumable(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.consumables.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_consumable(i)
                .expect("valid index for sell consumable");
        });
    }

    pub(super) fn unmask_action_space_buy_pack(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        self.shop
            .packs
            .iter()
            .enumerate()
            .filter(|(_, p)| self.shop.pack_price(p, &self.vouchers) <= self.money)
            .for_each(|(i, _)| {
                space.unmask_buy_pack(i).expect("valid index for buy pack");
            });
    }

    pub(super) fn unmask_action_space_buy_voucher(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.shop.voucher.is_none_or(|v| v.cost() > self.money) {
            return;
        }
        space.unmask_buy_voucher();
    }

    pub(super) fn unmask_action_space_buy_playing_card(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.deck.len() + self.available.cards().len() + self.discarded.len()
            >= self.config.deck_max
        {
            return;
        }
        self.shop
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| self.shop.card_price(c, &self.vouchers) <= self.money)
            .for_each(|(i, _)| {
                space
                    .unmask_buy_playing_card(i)
                    .expect("valid index for buy playing card");
            });
    }

    pub(super) fn unmask_action_space_pick_pack_card(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PackOpen() {
            return;
        }
        let Some(state) = self.open_pack.as_ref() else {
            return;
        };
        state
            .contents
            .iter()
            .enumerate()
            .filter(|(_, c)| self.pack_card_usable(c))
            .for_each(|(i, _)| {
                space
                    .unmask_pick_pack_card(i)
                    .expect("valid index for pick pack card");
            });
    }

    pub(super) fn unmask_action_space_skip_pack(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PackOpen() {
            return;
        }
        space.unmask_skip_pack();
    }

    pub(super) fn unmask_action_space_sort_hand(&self, space: &mut ActionSpace) {
        if !self.stage.is_blind() {
            return;
        }
        space
            .unmask_sort_hand(0)
            .expect("valid index for sort rank");
        space
            .unmask_sort_hand(1)
            .expect("valid index for sort suit");
    }

    pub(super) fn unmask_action_space_reroll(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.money < self.current_reroll_cost() {
            return;
        }
        space.unmask_reroll();
    }

    // Get an action space, masked for legal actions only
    pub fn gen_action_space(&self) -> ActionSpace {
        let mut space = ActionSpace::from(self.config.clone());
        self.unmask_action_space_select_cards(&mut space);
        self.unmask_action_space_play_and_discard(&mut space);
        self.unmask_action_space_move_cards(&mut space);
        self.unmask_action_space_cash_out(&mut space);
        self.unmask_action_space_next_round(&mut space);
        self.unmask_action_space_select_blind(&mut space);
        self.unmask_action_space_skip_blind(&mut space);
        self.unmask_action_space_buy_joker(&mut space);
        self.unmask_action_space_buy_consumable(&mut space);
        self.unmask_action_space_buy_voucher(&mut space);
        self.unmask_action_space_buy_playing_card(&mut space);
        self.unmask_action_space_use_consumable(&mut space);
        self.unmask_action_space_sell_joker(&mut space);
        self.unmask_action_space_sell_consumable(&mut space);
        self.unmask_action_space_buy_pack(&mut space);
        self.unmask_action_space_pick_pack_card(&mut space);
        self.unmask_action_space_skip_pack(&mut space);
        self.unmask_action_space_sort_hand(&mut space);
        self.unmask_action_space_reroll(&mut space);
        space
    }
}
