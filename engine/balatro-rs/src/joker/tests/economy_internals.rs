//! Credit Card's debt, Gift Card's sell values and Perkeo's Negative copies.

use super::*;
use crate::consumable::Consumable;
use crate::tarot::Tarot;

fn game_with(jokers: Vec<Jokers>) -> Game {
    let mut g = Game::default();
    g.start();
    g.money += 1000;
    for mut joker in jokers {
        joker.set_instance_id(crate::joker::mint_joker_id());
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
    }
    g.stage = Stage::Shop();
    g.take_events();
    g
}

#[test]
fn credit_card_lets_money_go_twenty_below_zero() {
    let mut g = game_with(vec![Jokers::CreditCard(Default::default())]);
    g.money = 3;
    assert_eq!(g.spendable(), 23);
    g.pay(10);
    assert_eq!(g.balance(), (7, true));
    assert_eq!(g.spendable(), 13);
    // Money earned pays the debt back first.
    g.money += 10;
    g.settle_debt();
    assert_eq!(g.balance(), (3, false));
    let mut plain = game_with(vec![]);
    plain.money = 3;
    assert_eq!(plain.spendable(), 3);
}

#[test]
fn gift_card_raises_every_sell_value() {
    let mut g = game_with(vec![
        Jokers::GiftCard(Default::default()),
        Jokers::TheJoker(Default::default()),
    ]);
    g.consumables.push(Consumable::Tarot(Tarot::Hermit));
    let joker = g.joker_sell_value(&g.jokers[1]);
    let card = g.consumable_sell_value(0);
    g.handle_round_end();
    assert_eq!(g.joker_sell_value(&g.jokers[1]), joker + 1);
    assert_eq!(g.consumable_sell_value(0), card + 1);
}

#[test]
fn perkeo_adds_a_negative_copy_with_its_own_slot() {
    let mut g = game_with(vec![Jokers::Perkeo(Default::default())]);
    g.consumables.push(Consumable::Tarot(Tarot::Hermit));
    let slots = g.consumable_slots();
    g.perkeo();
    assert_eq!(
        g.consumables,
        [Consumable::Tarot(Tarot::Hermit); 2],
        "a copy of the only card"
    );
    assert!(g.consumable_meta(1).negative);
    assert_eq!(g.consumable_slots(), slots + 1);
    // Negative: (3 + 5) / 2.
    assert_eq!(g.consumable_sell_value(1), 4);
    g.sell_consumable(1).unwrap();
    assert_eq!(g.consumable_slots(), slots);
}

#[test]
fn moving_a_consumable_keeps_its_state() {
    let mut g = game_with(vec![Jokers::Perkeo(Default::default())]);
    g.consumables.push(Consumable::Tarot(Tarot::Hermit));
    g.consumables.push(Consumable::Tarot(Tarot::Fool));
    g.perkeo();
    let negative = (0..3).find(|&i| g.consumable_meta(i).negative).unwrap();
    let card = g.consumables[negative];
    g.move_consumable(negative, 0);
    assert_eq!(g.consumables[0], card);
    assert!(g.consumable_meta(0).negative);
}
