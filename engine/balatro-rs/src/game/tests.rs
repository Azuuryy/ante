use super::*;
use crate::card::{Suit, Value};

// --- vouchers ---

/// A game sitting in a freshly-stocked shop, having cashed out (so the
/// ante's voucher offer has been drawn), with `money` to spend.
fn game_in_shop(money: usize) -> Game {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    g.money = money;
    g
}

fn redeem(g: &mut Game, voucher: Voucher) {
    g.shop.voucher = Some(voucher);
    g.money = g.money.max(voucher.cost());
    g.handle_action(Action::BuyVoucher(voucher))
        .expect("buy voucher");
}

mod boss_effects;
mod boss_play;
mod boss_runs;
mod decks;
mod editions_and_snapshots;
mod enhancements_and_seals;
mod real_rng;
mod rerolls_and_vouchers;
mod scoring_and_debuffs;
mod selling_and_packs;
mod stakes;
mod tags;
mod voucher_effects;
