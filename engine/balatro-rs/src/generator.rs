use crate::action::{Action, MoveDirection, SortBy};
use crate::game::Game;
use crate::space::ActionSpace;
use crate::stage::{Blind, BlindExt, Stage};

mod action_space;
mod pack_actions;

impl Game {
    // Get all legal SelectCard actions that can be executed given current state
    fn gen_actions_select_card(&self) -> Option<impl Iterator<Item = Action>> {
        if !self.hand_selectable() {
            return None;
        }
        // Cannot select more than max
        if self.available.selected().len() >= self.config.selected_max {
            return None;
        }
        let combos = self
            .available
            .not_selected()
            .clone()
            .into_iter()
            .map(Action::SelectCard);
        Some(combos)
    }

    // Get all legal DeselectCard actions that can be executed given current state
    fn gen_actions_deselect_card(&self) -> Option<impl Iterator<Item = Action>> {
        if !self.hand_selectable() {
            return None;
        }
        let selected = self.available.selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.into_iter().map(Action::DeselectCard))
    }

    // Get all legal Play actions that can be executed given current state
    fn gen_actions_play(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only play hand during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no plays remaining, return None
        if self.plays == 0 {
            return None;
        }
        // If no cards selected, return None
        if self.available.selected().is_empty() {
            return None;
        }
        let combos = vec![Action::Play()].into_iter();
        Some(combos)
    }

    // Get all legal Play actions that can be executed given current state
    fn gen_actions_discard(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only discard during blinds
        if !self.stage.is_blind() {
            return None;
        }
        // If no discards remaining, return None
        if self.discards() == 0 {
            return None;
        }
        // If no cards selected, return None
        if self.available.selected().is_empty() {
            return None;
        }
        let combos = vec![Action::Discard()].into_iter();
        Some(combos)
    }

    fn gen_actions_sort_hand(&self) -> Option<impl Iterator<Item = Action>> {
        if !self.stage.is_blind() {
            return None;
        }
        Some(
            vec![
                Action::SortHand(SortBy::Rank),
                Action::SortHand(SortBy::Suit),
            ]
            .into_iter(),
        )
    }

    // Get all legal move card actions
    fn gen_actions_move_card(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only move cards during blinds
        if !self.stage.is_blind() {
            return None;
        }
        let left = self
            .available
            .cards()
            .clone()
            .into_iter()
            .skip(1)
            .map(|c| Action::MoveCard(MoveDirection::Left, c));
        let right = self
            .available
            .cards()
            .clone()
            .into_iter()
            .rev()
            .skip(1)
            .rev()
            .map(|c| Action::MoveCard(MoveDirection::Right, c));

        let combos = left.chain(right);
        Some(combos)
    }

    // Get cash out action
    fn gen_actions_cash_out(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not post blind, cannot cash out
        if self.stage != Stage::PostBlind() {
            return None;
        }
        Some(vec![Action::CashOut(self.reward)].into_iter())
    }

    // Get next round action
    fn gen_actions_next_round(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not shop, cannot next round
        if self.stage != Stage::Shop() {
            return None;
        }
        Some(vec![Action::NextRound()].into_iter())
    }

    // Get select blind action
    fn gen_actions_select_blind(&self) -> Option<impl Iterator<Item = Action>> {
        // If stage is not pre blind, cannot select blind
        if self.stage != Stage::PreBlind() {
            return None;
        }
        if let Some(blind) = self.blind {
            Some(vec![Action::SelectBlind(blind.next())].into_iter())
        } else {
            Some(vec![Action::SelectBlind(Blind::Small)].into_iter())
        }
    }

    // Get skip blind action (never offered for Boss)
    fn gen_actions_skip_blind(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PreBlind() {
            return None;
        }
        let next_blind = self.blind.map(|b| b.next()).unwrap_or(Blind::Small);
        if next_blind == Blind::Boss {
            return None;
        }
        Some(vec![Action::SkipBlind(next_blind)].into_iter())
    }

    // Get buy joker actions
    fn gen_actions_buy_joker(&self) -> Option<impl Iterator<Item = Action> + '_> {
        // If stage is not shop, cannot buy
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop
            .gen_moves_buy_joker(self.money, &self.vouchers)
            .map(|actions| {
                actions.filter(
                    |action| matches!(action, Action::BuyJoker(joker) if self.can_add_joker(joker)),
                )
            })
    }

    // Get buy consumable actions
    fn gen_actions_buy_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop.gen_moves_buy_consumable(
            self.money,
            self.consumable_slots(),
            self.consumables.len(),
            &self.vouchers,
        )
    }

    // Get use consumable actions
    fn gen_actions_use_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(self.stage, Stage::End(_)) {
            return None;
        }
        if self.consumables.is_empty() {
            return None;
        }
        let actions: Vec<Action> = self
            .consumables
            .iter()
            .filter(|c| self.consumable_usable(c))
            .cloned()
            .map(Action::UseConsumable)
            .collect();
        if actions.is_empty() {
            None
        } else {
            Some(actions.into_iter())
        }
    }

    fn gen_actions_sell_joker(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(self.stage, Stage::End(_)) {
            return None;
        }
        if self.jokers.is_empty() {
            return None;
        }
        let actions: Vec<Action> = (0..self.jokers.len()).map(Action::SellJoker).collect();
        Some(actions.into_iter())
    }

    fn gen_actions_sell_consumable(&self) -> Option<impl Iterator<Item = Action>> {
        if matches!(self.stage, Stage::End(_)) {
            return None;
        }
        if self.consumables.is_empty() {
            return None;
        }
        let actions: Vec<Action> = (0..self.consumables.len())
            .map(Action::SellConsumable)
            .collect();
        Some(actions.into_iter())
    }

    // Get buy voucher actions. The offer sits in its own slot, is
    // never discounted, and can only be bought once per ante.
    fn gen_actions_buy_voucher(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop.gen_moves_buy_voucher(self.money)
    }

    // Get buy playing card actions (Magic Trick)
    fn gen_actions_buy_playing_card(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        if self.deck.len() + self.available.cards().len() + self.discarded.len()
            >= self.config.deck_max
        {
            return None;
        }
        self.shop.gen_moves_buy_card(self.money, &self.vouchers)
    }

    // Get buy pack actions
    fn gen_actions_buy_pack(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::Shop() {
            return None;
        }
        self.shop.gen_moves_buy_pack(self.money, &self.vouchers)
    }
}

#[cfg(test)]
mod tests;
