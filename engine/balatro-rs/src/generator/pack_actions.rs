use super::*;

impl Game {
    // Get pick pack card actions (active while a pack is open)
    pub(super) fn gen_actions_pick_pack_card(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PackOpen() {
            return None;
        }
        let state = self.open_pack.as_ref()?;
        let picks: Vec<Action> = state
            .contents
            .iter()
            .filter(|c| self.pack_card_usable(c))
            .cloned()
            .map(Action::PickPackCard)
            .collect();
        if picks.is_empty() {
            None
        } else {
            Some(picks.into_iter())
        }
    }

    // Skip is always available while a pack is open
    pub(super) fn gen_actions_skip_pack(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PackOpen() {
            return None;
        }
        Some(vec![Action::SkipPack()].into_iter())
    }

    pub(super) fn gen_actions_reroll(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        if self.money < self.current_reroll_cost() {
            return None;
        }
        Some(vec![Action::Reroll()].into_iter())
    }

    // Get all legal actions that can be executed given current state
    pub fn gen_actions(&self) -> impl Iterator<Item = Action> + '_ {
        let select_cards = self.gen_actions_select_card();
        let deselect_cards = self.gen_actions_deselect_card();
        let plays = self.gen_actions_play();
        let discards = self.gen_actions_discard();
        let move_cards = self.gen_actions_move_card();
        let sort_hands = self.gen_actions_sort_hand();
        let cash_outs = self.gen_actions_cash_out();
        let next_rounds = self.gen_actions_next_round();
        let select_blinds = self.gen_actions_select_blind();
        let skip_blinds = self.gen_actions_skip_blind();
        let buy_jokers = self.gen_actions_buy_joker();
        let buy_consumables = self.gen_actions_buy_consumable();
        let buy_vouchers = self.gen_actions_buy_voucher();
        let buy_playing_cards = self.gen_actions_buy_playing_card();
        let use_consumables = self.gen_actions_use_consumable();
        let sell_jokers = self.gen_actions_sell_joker();
        let sell_consumables = self.gen_actions_sell_consumable();
        let buy_packs = self.gen_actions_buy_pack();
        let pick_pack_cards = self.gen_actions_pick_pack_card();
        let skip_packs = self.gen_actions_skip_pack();
        let rerolls = self.gen_actions_reroll();

        select_cards
            .into_iter()
            .flatten()
            .chain(deselect_cards.into_iter().flatten())
            .chain(plays.into_iter().flatten())
            .chain(discards.into_iter().flatten())
            .chain(move_cards.into_iter().flatten())
            .chain(sort_hands.into_iter().flatten())
            .chain(cash_outs.into_iter().flatten())
            .chain(next_rounds.into_iter().flatten())
            .chain(select_blinds.into_iter().flatten())
            .chain(skip_blinds.into_iter().flatten())
            .chain(buy_jokers.into_iter().flatten())
            .chain(buy_consumables.into_iter().flatten())
            .chain(buy_vouchers.into_iter().flatten())
            .chain(buy_playing_cards.into_iter().flatten())
            .chain(use_consumables.into_iter().flatten())
            .chain(sell_jokers.into_iter().flatten())
            .chain(sell_consumables.into_iter().flatten())
            .chain(buy_packs.into_iter().flatten())
            .chain(pick_pack_cards.into_iter().flatten())
            .chain(skip_packs.into_iter().flatten())
            .chain(rerolls.into_iter().flatten())
            .chain(self.boss_reroll_available().then_some(Action::RerollBoss()))
    }
}
