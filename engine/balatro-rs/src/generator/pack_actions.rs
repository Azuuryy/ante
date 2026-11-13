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
            .filter(|c| self.pack_card_pickable(c))
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
        if self.stage != Stage::PackOpen() && !self.targeting_pack() {
            return None;
        }
        Some(vec![Action::SkipPack()].into_iter())
    }

    pub(super) fn gen_actions_reroll(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        if self.money < self.reroll_cost {
            return None;
        }
        Some(vec![Action::Reroll()].into_iter())
    }

    // are we in the temp tarot hand stage?
    // if so, we draw a temp hand and give options for applying the tarot.
    pub(super) fn gen_actions_tarot_hand(&self) -> Option<impl Iterator<Item = Action>> {
        let Stage::TarotHand(t) = self.stage else {
            return None;
        };
        let selected_count = self.available.selected().len();

        let select_cards: Vec<Action> = if selected_count < t.max_targets() {
            self.available
                .not_selected()
                .into_iter()
                .map(Action::SelectCard)
                .collect()
        } else {
            vec![]
        };

        let deselect_cards: Vec<Action> = self
            .available
            .selected()
            .into_iter()
            .map(Action::DeselectCard)
            .collect();

        let move_left: Vec<Action> = self
            .available
            .cards()
            .into_iter()
            .skip(1)
            .map(|c| Action::MoveCard(MoveDirection::Left, c))
            .collect();

        let move_right: Vec<Action> = {
            let cards = self.available.cards();
            let len = cards.len();
            if len > 1 {
                cards
                    .into_iter()
                    .take(len - 1)
                    .map(|c| Action::MoveCard(MoveDirection::Right, c))
                    .collect()
            } else {
                vec![]
            }
        };

        let apply: Vec<Action> =
            if selected_count >= t.min_targets() && selected_count <= t.max_targets() {
                vec![Action::ApplyTarot()]
            } else {
                vec![]
            };

        let all: Vec<Action> = select_cards
            .into_iter()
            .chain(deselect_cards)
            .chain(move_left)
            .chain(move_right)
            .chain(apply)
            .collect();

        Some(all.into_iter())
    }

    // are we in the temp spectral hand stage?
    // if so, we draw a temp hand and give options for applying the spectral.
    pub(super) fn gen_actions_spectral_hand(&self) -> Option<impl Iterator<Item = Action>> {
        let Stage::SpectralHand(s) = self.stage else {
            return None;
        };
        let selected_count = self.available.selected().len();

        let select_cards: Vec<Action> = if selected_count < s.max_targets() {
            self.available
                .not_selected()
                .into_iter()
                .map(Action::SelectCard)
                .collect()
        } else {
            vec![]
        };

        let deselect_cards: Vec<Action> = self
            .available
            .selected()
            .into_iter()
            .map(Action::DeselectCard)
            .collect();

        let move_left: Vec<Action> = self
            .available
            .cards()
            .into_iter()
            .skip(1)
            .map(|c| Action::MoveCard(MoveDirection::Left, c))
            .collect();

        let move_right: Vec<Action> = {
            let cards = self.available.cards();
            let len = cards.len();
            if len > 1 {
                cards
                    .into_iter()
                    .take(len - 1)
                    .map(|c| Action::MoveCard(MoveDirection::Right, c))
                    .collect()
            } else {
                vec![]
            }
        };

        let apply: Vec<Action> =
            if selected_count >= s.min_targets() && selected_count <= s.max_targets() {
                vec![Action::ApplySpectral()]
            } else {
                vec![]
            };

        let all: Vec<Action> = select_cards
            .into_iter()
            .chain(deselect_cards)
            .chain(move_left)
            .chain(move_right)
            .chain(apply)
            .collect();

        Some(all.into_iter())
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
        let tarot_hand = self.gen_actions_tarot_hand();
        let spectral_hand = self.gen_actions_spectral_hand();
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
            .chain(tarot_hand.into_iter().flatten())
            .chain(spectral_hand.into_iter().flatten())
            .chain(sell_jokers.into_iter().flatten())
            .chain(sell_consumables.into_iter().flatten())
            .chain(buy_packs.into_iter().flatten())
            .chain(pick_pack_cards.into_iter().flatten())
            .chain(skip_packs.into_iter().flatten())
            .chain(rerolls.into_iter().flatten())
            .chain(self.boss_reroll_available().then_some(Action::RerollBoss()))
    }
}
