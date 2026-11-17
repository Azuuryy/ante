//! Stakes from White to Gold and the stickers they bring.

use super::*;
use crate::event::{GameEvent, JokerNote};
use balatro_types::joker::Stickers;

fn game(stake: usize) -> Game {
    let mut g = Game::new(Config {
        plays: 4,
        discards: 3,
        ..Config::default()
    });
    g.apply_deck(balatro_types::DeckVariant::Red);
    g.apply_stake(stake);
    g.start();
    g
}

#[test]
fn red_stake_takes_the_small_blinds_reward() {
    assert_eq!(game(1).blind_reward(Blind::Small), 3);
    assert_eq!(game(2).blind_reward(Blind::Small), 0);
    assert_eq!(game(2).blind_reward(Blind::Big), 4);
}

#[test]
fn green_and_purple_stakes_raise_blind_amounts() {
    let at_ante = |stake: usize, ante: usize| {
        let mut g = game(stake);
        g.ante_current = crate::ante::Ante::try_from(ante).unwrap();
        g.required_score_for(Some(Blind::Small))
    };
    assert_eq!(at_ante(1, 2), 800);
    assert_eq!(at_ante(3, 2), 900);
    assert_eq!(at_ante(6, 8), 200000);
}

#[test]
fn blue_stake_takes_a_discard() {
    assert_eq!(game(4).discards_per_round(), 4);
    assert_eq!(game(5).discards_per_round(), 3);
}

#[test]
fn black_stake_and_up_put_stickers_on_shop_jokers() {
    let mut g = game(8);
    let (mut eternal, mut perishable, mut rental) = (0, 0, 0);
    for _ in 0..400 {
        let mut joker = Jokers::TheJoker(Default::default());
        g.roll_stickers(&mut joker);
        let stickers = joker.stickers();
        eternal += usize::from(stickers.eternal);
        perishable += usize::from(stickers.perishable);
        rental += usize::from(stickers.rental);
        assert!(!(stickers.eternal && stickers.perishable));
    }
    // About 30% each.
    for count in [eternal, perishable, rental] {
        assert!(
            (70..=170).contains(&count),
            "{eternal} {perishable} {rental}"
        );
    }
    let mut white = game(1);
    let mut joker = Jokers::TheJoker(Default::default());
    white.roll_stickers(&mut joker);
    assert_eq!(joker.stickers(), Stickers::default());
}

#[test]
fn eternal_jokers_cannot_be_sold() {
    let mut g = game(4);
    let mut joker = Jokers::TheJoker(Default::default());
    joker.set_stickers(Stickers {
        eternal: true,
        ..Default::default()
    });
    g.jokers.push(joker);
    g.stage = Stage::Shop();
    assert!(g.sell_joker(0).is_err());
}

#[test]
fn rental_costs_three_dollars_a_round_and_one_to_buy() {
    let mut g = game(8);
    let mut joker = Jokers::TheJoker(Default::default());
    joker.set_stickers(Stickers {
        rental: true,
        ..Default::default()
    });
    assert_eq!(g.shop.joker_price(&joker, &g.vouchers), 1);
    g.jokers.push(joker);
    g.money = 10;
    g.handle_round_end();
    assert_eq!(g.money, 7);
}

#[test]
fn perishable_jokers_stop_after_five_rounds() {
    let mut g = game(7);
    let mut joker = Jokers::TheJoker(Default::default());
    joker.set_stickers(Stickers {
        perishable: true,
        ..Default::default()
    });
    let mut state = joker.state();
    state.perish_rounds = 5;
    joker.set_state(state);
    g.jokers.push(joker);
    g.effect_registry
        .register_jokers(g.jokers.clone(), &g.clone());
    let ace = || {
        SelectHand::new(vec![Card::new(Value::Ace, Suit::Spade)])
            .best_hand()
            .unwrap()
    };
    assert_eq!(g.calc_score(ace()), 16 * 5);
    for _ in 0..5 {
        g.handle_round_end();
    }
    let last = g
        .take_events()
        .into_iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::JokerMessage { note, .. } => Some(note),
            _ => None,
        });
    assert_eq!(last, Some(JokerNote::Disabled));
    assert!(g.jokers[0].is_disabled());
    assert_eq!(g.calc_score(ace()), 16);
}
