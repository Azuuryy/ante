use crate::action::{Action, MoveDirection, SortBy};
use crate::consumable::Consumable;
use crate::game::Game;
use crate::pack::PackContent;
use crate::space::ActionSpace;
use crate::stage::{Blind, BlindExt, Stage};

impl Game {
    // Get all legal SelectCard actions that can be executed given current state
    fn gen_actions_select_card(&self) -> Option<impl Iterator<Item = Action>> {
        // Can only select card during blinds
        if !self.stage.is_blind() {
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
        if !self.stage.is_blind() {
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
        if matches!(
            self.stage,
            Stage::End(_) | Stage::TarotHand(_) | Stage::SpectralHand(_) | Stage::PackOpen()
        ) {
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

    // Get pick pack card actions (active while a pack is open)
    fn gen_actions_pick_pack_card(&self) -> Option<impl Iterator<Item = Action>> {
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
    fn gen_actions_skip_pack(&self) -> Option<impl Iterator<Item = Action>> {
        if self.stage != Stage::PackOpen() && !self.targeting_pack() {
            return None;
        }
        Some(vec![Action::SkipPack()].into_iter())
    }

    fn gen_actions_reroll(&self) -> Option<impl Iterator<Item = Action>> {
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
    fn gen_actions_tarot_hand(&self) -> Option<impl Iterator<Item = Action>> {
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
    fn gen_actions_spectral_hand(&self) -> Option<impl Iterator<Item = Action>> {
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

    fn unmask_action_space_select_cards(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_play_and_discard(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_move_cards(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_cash_out(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PostBlind() {
            return;
        }
        space.unmask_cash_out();
    }

    fn unmask_action_space_next_round(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        space.unmask_next_round();
    }

    fn unmask_action_space_select_blind(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PreBlind() {
            return;
        }
        space.unmask_select_blind();
    }

    fn unmask_action_space_skip_blind(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PreBlind() {
            return;
        }
        let next_blind = self.blind.map(|b| b.next()).unwrap_or(Blind::Small);
        if next_blind == Blind::Boss {
            return;
        }
        space.unmask_skip_blind();
    }

    fn unmask_action_space_buy_joker(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_buy_consumable(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_use_consumable(&self, space: &mut ActionSpace) {
        if matches!(
            self.stage,
            Stage::End(_) | Stage::TarotHand(_) | Stage::SpectralHand(_)
        ) {
            return;
        }
        let selected_count = self.available.selected().len();
        self.consumables.iter().enumerate().for_each(|(i, c)| {
            let valid = match c {
                Consumable::Planet(_) => true,
                Consumable::Tarot(t) => {
                    if !t.requires_targets() {
                        true
                    } else if self.stage.is_blind() {
                        selected_count >= t.min_targets() && selected_count <= t.max_targets()
                    } else {
                        true
                    }
                }
                // Same eligibility shape as Tarot above.
                Consumable::Spectral(s) => {
                    if !s.requires_targets() {
                        true
                    } else if self.stage.is_blind() {
                        selected_count >= s.min_targets() && selected_count <= s.max_targets()
                    } else {
                        true
                    }
                }
            };
            if valid {
                space
                    .unmask_use_consumable(i)
                    .expect("valid index for use consumable");
            }
        });
    }

    fn unmask_action_space_sell_joker(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.jokers.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_joker(i)
                .expect("valid index for sell joker");
        });
    }

    fn unmask_action_space_sell_consumable(&self, space: &mut ActionSpace) {
        if matches!(self.stage, Stage::End(_)) {
            return;
        }
        self.consumables.iter().enumerate().for_each(|(i, _)| {
            space
                .unmask_sell_consumable(i)
                .expect("valid index for sell consumable");
        });
    }

    fn unmask_action_space_buy_pack(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_buy_voucher(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.shop.voucher.is_none_or(|v| v.cost() > self.money) {
            return;
        }
        space.unmask_buy_voucher();
    }

    fn unmask_action_space_buy_playing_card(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_pick_pack_card(&self, space: &mut ActionSpace) {
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
            .filter(|(_, c)| match c {
                PackContent::Joker(joker) => self.can_add_joker(joker),
                _ => true,
            })
            .for_each(|(i, _)| {
                space
                    .unmask_pick_pack_card(i)
                    .expect("valid index for pick pack card");
            });
    }

    fn unmask_action_space_skip_pack(&self, space: &mut ActionSpace) {
        if self.stage != Stage::PackOpen() {
            return;
        }
        space.unmask_skip_pack();
    }

    fn unmask_action_space_sort_hand(&self, space: &mut ActionSpace) {
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

    fn unmask_action_space_reroll(&self, space: &mut ActionSpace) {
        if self.stage != Stage::Shop() {
            return;
        }
        if self.money < self.reroll_cost {
            return;
        }
        space.unmask_reroll();
    }

    fn unmask_action_space_tarot_hand(&self, space: &mut ActionSpace) {
        let Stage::TarotHand(t) = self.stage else {
            return;
        };
        // select cards
        let selected_count = self.available.selected().len();
        if selected_count < t.max_targets() {
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
        // move cards
        self.available
            .cards()
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_left(i)
                    .expect("valid index for move left");
            });
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
                    .expect("valid index for move right");
            });
        if selected_count >= t.min_targets() && selected_count <= t.max_targets() {
            space.unmask_apply_tarot();
        }
    }

    fn unmask_action_space_spectral_hand(&self, space: &mut ActionSpace) {
        let Stage::SpectralHand(s) = self.stage else {
            return;
        };
        // select cards
        let selected_count = self.available.selected().len();
        if selected_count < s.max_targets() {
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
        // move cards
        self.available
            .cards()
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, _)| {
                space
                    .unmask_move_card_left(i)
                    .expect("valid index for move left");
            });
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
                    .expect("valid index for move right");
            });
        if selected_count >= s.min_targets() && selected_count <= s.max_targets() {
            space.unmask_apply_spectral();
        }
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
        self.unmask_action_space_tarot_hand(&mut space);
        self.unmask_action_space_spectral_hand(&mut space);
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

#[cfg(test)]
mod tests;
