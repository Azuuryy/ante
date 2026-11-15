use crate::action::{Action, MoveDirection, SortBy};
use crate::config::Config;
use crate::error::ActionSpaceError;
use crate::game::Game;
use crate::stage::{Blind, BlindExt};

mod conversion;
mod unmask;

const PACK_SLOTS: usize = 2;
const PACK_CONTENTS_MAX: usize = 5;

// Hard code a bounded action space.
// Given constraints:
// available max = 24
// store consumable slots max = 4
// consumable slots = 2
// joker slots = 5
//
// 0-23: select card
// 24-46: move card (left)
// 47-69: move card (right)
// 70: play
// 71: discard
// 72: cashout
// 73-76: buy joker
// 77: next round
// 78: select blind
// 79: skip blind
// 80-81: buy consumable
// 82: buy voucher
// 83-86: buy playing card
// 87-88: use consumable
// 89: apply tarot
// 90-94: sell joker
// 95-96: sell consumable
// 97-98: buy pack
// 99-103: pick pack card
// 104: skip pack
// 105: sort hand (rank)
// 106: sort hand (suit)
// 107: reroll
// 108: apply spectral
//
// We end up with a vector of length 109 where each index
// represents a potential action.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ActionSpace {
    pub select_card: Vec<usize>,
    pub move_card_left: Vec<usize>,
    pub move_card_right: Vec<usize>,
    pub play: Vec<usize>,
    pub discard: Vec<usize>,
    pub cash_out: Vec<usize>,
    pub buy_joker: Vec<usize>,
    pub next_round: Vec<usize>,
    pub select_blind: Vec<usize>,
    pub skip_blind: Vec<usize>,
    pub buy_consumable: Vec<usize>,
    pub buy_voucher: Vec<usize>,
    pub buy_playing_card: Vec<usize>,
    pub use_consumable: Vec<usize>,
    pub sell_joker: Vec<usize>,
    pub sell_consumable: Vec<usize>,
    pub buy_pack: Vec<usize>,
    pub pick_pack_card: Vec<usize>,
    pub skip_pack: Vec<usize>,
    pub sort_hand: Vec<usize>,
    pub reroll: Vec<usize>,
}

impl ActionSpace {
    pub fn size(&self) -> usize {
        self.select_card.len()
            + self.move_card_left.len()
            + self.move_card_right.len()
            + self.play.len()
            + self.discard.len()
            + self.cash_out.len()
            + self.buy_joker.len()
            + self.next_round.len()
            + self.select_blind.len()
            + self.skip_blind.len()
            + self.buy_consumable.len()
            + self.buy_voucher.len()
            + self.buy_playing_card.len()
            + self.use_consumable.len()
            + self.sell_joker.len()
            + self.sell_consumable.len()
            + self.buy_pack.len()
            + self.pick_pack_card.len()
            + self.skip_pack.len()
            + self.sort_hand.len()
            + self.reroll.len()
    }

    fn select_card_min(&self) -> usize {
        0
    }

    fn select_card_max(&self) -> usize {
        self.select_card_min() + self.select_card.len() - 1
    }

    fn move_card_left_min(&self) -> usize {
        self.select_card_max() + 1
    }

    fn move_card_left_max(&self) -> usize {
        self.move_card_left_min() + self.select_card.len() - 2
    }

    fn move_card_right_min(&self) -> usize {
        self.move_card_left_max() + 1
    }

    fn move_card_right_max(&self) -> usize {
        self.move_card_right_min() + self.select_card.len() - 2
    }

    fn play_min(&self) -> usize {
        self.move_card_right_max() + 1
    }

    fn play_max(&self) -> usize {
        self.play_min() + self.play.len() - 1
    }

    fn discard_min(&self) -> usize {
        self.play_max() + 1
    }

    fn discard_max(&self) -> usize {
        self.discard_min() + self.discard.len() - 1
    }

    fn cash_out_min(&self) -> usize {
        self.discard_max() + 1
    }

    fn cash_out_max(&self) -> usize {
        self.cash_out_min() + self.cash_out.len() - 1
    }

    fn buy_joker_min(&self) -> usize {
        self.cash_out_max() + 1
    }

    fn buy_joker_max(&self) -> usize {
        self.buy_joker_min() + self.buy_joker.len() - 1
    }

    fn next_round_min(&self) -> usize {
        self.buy_joker_max() + 1
    }

    fn next_round_max(&self) -> usize {
        self.next_round_min() + self.next_round.len() - 1
    }

    fn select_blind_min(&self) -> usize {
        self.next_round_max() + 1
    }

    fn select_blind_max(&self) -> usize {
        self.select_blind_min() + self.select_blind.len() - 1
    }

    fn skip_blind_min(&self) -> usize {
        self.select_blind_max() + 1
    }

    fn skip_blind_max(&self) -> usize {
        self.skip_blind_min() + self.skip_blind.len() - 1
    }

    fn buy_consumable_min(&self) -> usize {
        self.skip_blind_max() + 1
    }

    fn buy_consumable_max(&self) -> usize {
        self.buy_consumable_min() + self.buy_consumable.len() - 1
    }

    fn buy_voucher_min(&self) -> usize {
        self.buy_consumable_max() + 1
    }

    fn buy_voucher_max(&self) -> usize {
        self.buy_voucher_min() + self.buy_voucher.len().saturating_sub(1)
    }

    fn buy_playing_card_min(&self) -> usize {
        self.buy_voucher_min() + self.buy_voucher.len()
    }

    fn buy_playing_card_max(&self) -> usize {
        self.buy_playing_card_min() + self.buy_playing_card.len().saturating_sub(1)
    }

    fn use_consumable_min(&self) -> usize {
        self.buy_playing_card_min() + self.buy_playing_card.len()
    }

    fn use_consumable_max(&self) -> usize {
        self.use_consumable_min() + self.use_consumable.len() - 1
    }

    fn sell_joker_min(&self) -> usize {
        self.use_consumable_max() + 1
    }

    fn sell_joker_max(&self) -> usize {
        self.sell_joker_min() + self.sell_joker.len().saturating_sub(1)
    }

    fn sell_consumable_min(&self) -> usize {
        self.sell_joker_min() + self.sell_joker.len()
    }

    fn sell_consumable_max(&self) -> usize {
        self.sell_consumable_min() + self.sell_consumable.len().saturating_sub(1)
    }

    fn buy_pack_min(&self) -> usize {
        self.sell_consumable_min() + self.sell_consumable.len()
    }

    fn buy_pack_max(&self) -> usize {
        self.buy_pack_min() + self.buy_pack.len().saturating_sub(1)
    }

    fn pick_pack_card_min(&self) -> usize {
        self.buy_pack_min() + self.buy_pack.len()
    }

    fn pick_pack_card_max(&self) -> usize {
        self.pick_pack_card_min() + self.pick_pack_card.len().saturating_sub(1)
    }

    fn skip_pack_min(&self) -> usize {
        self.pick_pack_card_min() + self.pick_pack_card.len()
    }

    fn skip_pack_max(&self) -> usize {
        self.skip_pack_min()
    }

    fn sort_hand_min(&self) -> usize {
        self.skip_pack_min() + self.skip_pack.len()
    }

    fn sort_hand_max(&self) -> usize {
        self.sort_hand_min() + self.sort_hand.len().saturating_sub(1)
    }

    fn reroll_min(&self) -> usize {
        self.sort_hand_min() + self.sort_hand.len()
    }

    fn reroll_max(&self) -> usize {
        self.reroll_min()
    }
}

impl From<Config> for ActionSpace {
    fn from(c: Config) -> Self {
        ActionSpace {
            select_card: vec![0; c.available_max],
            move_card_left: vec![0; c.available_max - 1], // every card but leftmost can move left
            move_card_right: vec![0; c.available_max - 1], // every card but rightmost can move right
            play: vec![0; 1],
            discard: vec![0; 1],
            cash_out: vec![0; 1],
            buy_joker: vec![0; c.store_consumable_slots_max],
            next_round: vec![0; 1],
            select_blind: vec![0; 1],
            skip_blind: vec![0; 1],
            buy_consumable: vec![0; c.consumable_slots],
            // One voucher on offer per ante, at most.
            buy_voucher: vec![0; 1],
            // Magic Trick can turn every shop card slot into a playing card.
            buy_playing_card: vec![0; c.store_consumable_slots_max],
            use_consumable: vec![0; c.consumable_slots],
            sell_joker: vec![0; c.joker_slots],
            sell_consumable: vec![0; c.consumable_slots],
            buy_pack: vec![0; PACK_SLOTS],
            pick_pack_card: vec![0; PACK_CONTENTS_MAX],
            skip_pack: vec![0; 1],
            sort_hand: vec![0; 2],
            reroll: vec![0; 1],
        }
    }
}

// Generate an action space vector, masked based on current state
impl From<ActionSpace> for Vec<usize> {
    fn from(a: ActionSpace) -> Vec<usize> {
        [
            a.select_card,
            a.move_card_left,
            a.move_card_right,
            a.play,
            a.discard,
            a.cash_out,
            a.buy_joker,
            a.next_round,
            a.select_blind,
            a.skip_blind,
            a.buy_consumable,
            a.buy_voucher,
            a.buy_playing_card,
            a.use_consumable,
            a.sell_joker,
            a.sell_consumable,
            a.buy_pack,
            a.pick_pack_card,
            a.skip_pack,
            a.sort_hand,
            a.reroll,
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests;
