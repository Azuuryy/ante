//! Lot 3, batch 2a: Jokers that grow or wear out.

use super::*;
use crate::consumable::Consumable;
use crate::event::{GameEvent, JokerNote};
use crate::planet::Planets;
use crate::tarot::Tarot;

/// A started game in `stage` with `jokers` bought (distinct instances), events cleared.
fn game_with(jokers: Vec<Jokers>, stage: Stage) -> Game {
    let mut g = Game::default();
    g.start();
    g.blind = Some(Blind::Small);
    g.money += 1000;
    for mut joker in jokers {
        joker.set_instance_id(crate::joker::mint_joker_id());
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
    }
    g.stage = stage;
    g.take_events();
    g
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

fn counter(g: &Game) -> f32 {
    g.jokers[0].state().counter
}

fn score(g: &mut Game, cards: Vec<Card>) -> usize {
    g.calc_score(SelectHand::new(cards).best_hand().unwrap())
}

fn cards(values: &[Value]) -> Vec<Card> {
    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let suits = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond];
            Card::new(*value, suits[i % 4])
        })
        .collect()
}

#[test]
fn square_joker_grows_on_four_card_hands_only() {
    let mut g = game_with(
        vec![Jokers::SquareJoker(Default::default())],
        Stage::Blind(Blind::Small),
    );
    let four = cards(&[Value::Two, Value::Two, Value::Nine, Value::Nine]);
    let plain = {
        let mut bare = game_with(vec![], Stage::Blind(Blind::Small));
        score(&mut bare, four.clone())
    };
    // Two Pair 20 chips × 2 Mult: 4 more chips → 8 more points.
    assert_eq!(score(&mut g, four.clone()), plain + 8);
    assert_eq!(counter(&g), 4.0);
    score(&mut g, cards(&[Value::Ace]));
    assert_eq!(counter(&g), 4.0, "a one-card hand does not grow it");
}

#[test]
fn runner_grows_on_straights() {
    let mut g = game_with(
        vec![Jokers::Runner(Default::default())],
        Stage::Blind(Blind::Small),
    );
    score(
        &mut g,
        cards(&[
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Six,
        ]),
    );
    assert_eq!(counter(&g), 15.0);
}

#[test]
fn throwback_and_fortune_teller_read_the_run() {
    let mut g = game_with(
        vec![Jokers::Throwback(Default::default())],
        Stage::Blind(Blind::Small),
    );
    g.skipped_count = 2;
    // High Card Ace: 16 × X1.5.
    assert_eq!(score(&mut g, cards(&[Value::Ace])), 24);
    let mut g = game_with(
        vec![Jokers::FortuneTeller(Default::default())],
        Stage::Blind(Blind::Small),
    );
    g.tarots_used = 3;
    assert_eq!(score(&mut g, cards(&[Value::Ace])), 16 * 4);
}

#[test]
fn using_cards_grows_fortune_teller_and_constellation() {
    let mut g = game_with(
        vec![
            Jokers::FortuneTeller(Default::default()),
            Jokers::Constellation(Default::default()),
        ],
        Stage::Shop(),
    );
    g.consumables.push(Consumable::Tarot(Tarot::Hermit));
    g.use_consumable(Consumable::Tarot(Tarot::Hermit)).unwrap();
    assert_eq!(g.tarots_used, 1);
    assert_eq!(notes(&mut g), [JokerNote::MultPlus(1)]);
    g.consumables.push(Consumable::Planet(Planets::Pluto));
    g.use_consumable(Consumable::Planet(Planets::Pluto))
        .unwrap();
    assert_eq!(notes(&mut g), [JokerNote::XMult(110)]);
}

#[test]
fn flash_card_grows_on_rerolls() {
    let mut g = game_with(vec![Jokers::FlashCard(Default::default())], Stage::Shop());
    g.reroll().unwrap();
    assert_eq!(counter(&g), 2.0);
    assert_eq!(notes(&mut g), [JokerNote::MultPlus(2)]);
}

#[test]
fn popcorn_wears_out_then_is_eaten() {
    let mut g = game_with(
        vec![Jokers::Popcorn(Default::default())],
        Stage::Blind(Blind::Small),
    );
    assert_eq!(score(&mut g, cards(&[Value::Ace])), 16 * 21);
    g.handle_round_end();
    assert_eq!(notes(&mut g), [JokerNote::MultMinus(4)]);
    for _ in 0..3 {
        g.handle_round_end();
    }
    g.take_events();
    g.handle_round_end();
    assert!(g.jokers.is_empty(), "20 Mult lasts five rounds");
    assert_eq!(notes(&mut g), [JokerNote::Eaten]);
}

#[test]
fn turtle_bean_widens_the_hand_less_each_round() {
    let bare = game_with(vec![], Stage::Blind(Blind::Small)).hand_size();
    let mut g = game_with(
        vec![Jokers::TurtleBean(Default::default())],
        Stage::Blind(Blind::Small),
    );
    assert_eq!(g.hand_size(), bare + 5);
    g.handle_round_end();
    assert_eq!(g.hand_size(), bare + 4);
    assert_eq!(notes(&mut g), [JokerNote::HandSizeMinus(1)]);
}

#[test]
fn ramen_loses_a_hundredth_per_card_discarded() {
    let mut g = game_with(
        vec![Jokers::Ramen(Default::default())],
        Stage::Blind(Blind::Small),
    );
    g.deal();
    g.take_events();
    for card in g.available.cards().into_iter().take(3) {
        g.available.select_card(card).unwrap();
    }
    g.discard_selected().unwrap();
    assert!((counter(&g) - 0.03).abs() < 1e-6);
    assert_eq!(notes(&mut g), [JokerNote::XMultMinus(1); 3]);
}

#[test]
fn lucky_cat_grows_when_a_lucky_card_triggers() {
    let mut g = game_with(
        vec![Jokers::LuckyCat(Default::default())],
        Stage::Blind(Blind::Small),
    );
    g.prob_mult = 15; // Both Lucky rolls always hit.
    let mut lucky = Card::new(Value::Ace, Suit::Spade);
    lucky.enhancement = Some(Enhancement::Lucky);
    score(&mut g, vec![lucky]);
    assert_eq!(counter(&g), 0.25);
}
