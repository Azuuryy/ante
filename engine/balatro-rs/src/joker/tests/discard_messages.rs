//! Jokers that act on a discard or at the end of the round, and the messages they leave
//! as events (`card_eval_status_text` in the original's `context.discard`).

use super::*;
use crate::event::{GameEvent, JokerNote};
use balatro_types::joker::SelectorValue;

/// A small blind with `joker` bought and exactly `hand` in hand, events cleared.
fn game_with(joker: Jokers, hand: Vec<Card>) -> (Game, usize) {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(joker.clone());
    g.buy_joker(joker).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    g.available.empty();
    g.available.extend(hand);
    g.take_events();
    let id = g.jokers[0].instance_id();
    (g, id)
}

fn discard_all(g: &mut Game) {
    for card in g.available.cards() {
        g.available.select_card(card).expect("can select");
    }
    g.discard_selected().expect("can discard");
}

fn notes(g: &mut Game) -> Vec<JokerNote> {
    g.take_events()
        .into_iter()
        .filter_map(|event| match event {
            GameEvent::JokerMessage { note, .. } => Some(note),
            _ => None,
        })
        .collect()
}

fn cards(values: &[Value], suit: Suit) -> Vec<Card> {
    values.iter().map(|value| Card::new(*value, suit)).collect()
}

#[test]
fn faceless_joker_pays_for_three_face_cards_not_for_five_cards() {
    let faces = cards(
        &[Value::King, Value::Queen, Value::Jack, Value::Two],
        Suit::Heart,
    );
    let (mut g, _) = game_with(Jokers::FacelessJoker(Default::default()), faces);
    let money = g.money;
    discard_all(&mut g);
    assert_eq!(g.money, money + 5);
    assert_eq!(notes(&mut g), [JokerNote::Dollars(5)]);

    let plain = cards(
        &[
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::King,
        ],
        Suit::Heart,
    );
    let (mut g, _) = game_with(Jokers::FacelessJoker(Default::default()), plain);
    let money = g.money;
    discard_all(&mut g);
    assert_eq!(g.money, money, "five cards, one face");
    assert!(notes(&mut g).is_empty());
}

#[test]
fn castle_says_upgrade_for_each_card_of_its_suit() {
    let hand = vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Three, Suit::Heart),
        Card::new(Value::Four, Suit::Spade),
    ];
    let (mut g, id) = game_with(Jokers::Castle(Default::default()), hand);
    g.joker_state_mut(id).unwrap().selector = Some(SelectorValue::Suit(Suit::Heart));
    discard_all(&mut g);
    assert_eq!(g.joker_state_mut(id).unwrap().counter, 6.0);
    assert_eq!(notes(&mut g), [JokerNote::Upgrade, JokerNote::Upgrade]);
}

#[test]
fn mail_in_rebate_skips_debuffed_cards() {
    let hand = vec![
        Card::new(Value::King, Suit::Club),
        Card::new(Value::King, Suit::Heart),
    ];
    let (mut g, id) = game_with(Jokers::MailInRebate(Default::default()), hand);
    g.blind = Some(Blind::Boss);
    g.current_boss = Some(BossBlind::Club);
    g.joker_state_mut(id).unwrap().selector = Some(SelectorValue::Value(Value::King));
    let money = g.money;
    discard_all(&mut g);
    assert_eq!(g.money, money + 5, "the King of Clubs is debuffed");
    assert_eq!(notes(&mut g), [JokerNote::Dollars(5)]);
}

#[test]
fn hit_the_road_counts_each_jack_then_resets_at_the_end_of_the_round() {
    let jacks = cards(&[Value::Jack, Value::Jack, Value::Two], Suit::Spade);
    let (mut g, _) = game_with(Jokers::HitTheRoad(Default::default()), jacks);
    discard_all(&mut g);
    assert_eq!(
        notes(&mut g),
        [JokerNote::XMult(150), JokerNote::XMult(200)]
    );
    g.handle_round_end();
    assert_eq!(notes(&mut g), [JokerNote::Reset]);
}

#[test]
fn green_joker_loses_mult_only_when_it_has_some() {
    let hand = cards(&[Value::Two], Suit::Spade);
    let (mut g, id) = game_with(Jokers::GreenJoker(Default::default()), hand.clone());
    discard_all(&mut g);
    assert!(notes(&mut g).is_empty(), "already at +0 Mult");
    g.joker_state_mut(id).unwrap().counter = 2.0;
    g.available.extend(hand);
    discard_all(&mut g);
    assert_eq!(notes(&mut g), [JokerNote::MultMinus(1)]);
}

#[test]
fn a_debuffed_purple_seal_makes_no_tarot() {
    let mut seal = Card::new(Value::Two, Suit::Club);
    seal.seal = Some(crate::card::Seal::Purple);
    let (mut g, _) = game_with(Jokers::TheJoker(Default::default()), vec![seal]);
    g.blind = Some(Blind::Boss);
    g.current_boss = Some(BossBlind::Club);
    discard_all(&mut g);
    assert!(g.consumables.is_empty());
}
