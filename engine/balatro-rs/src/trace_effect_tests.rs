// Score trace steps tell XMult from +Mult and carry money earned while scoring.

use crate::card::{Card, Edition, Seal, Suit, Value};
use crate::game::Game;
use crate::hand::SelectHand;
use crate::joker::{Jokers, TheDuo};
use crate::score::ScoreSource;
use crate::stage::{Blind, Stage};

fn blind_game() -> Game {
    Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    }
}

fn pair() -> SelectHand {
    let a = Card::new(Value::Ace, Suit::Heart);
    let b = Card::new(Value::Ace, Suit::Club);
    SelectHand::new(vec![a, b])
}

#[test]
fn an_xmult_joker_step_carries_its_factor() {
    let mut g = blind_game();
    let joker = Jokers::TheDuo(TheDuo::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(joker.clone());
    g.buy_joker(joker).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    let (_, trace) = g.calc_score_traced(pair().best_hand().unwrap());
    let duo = trace
        .0
        .iter()
        .find(|step| matches!(step.source, ScoreSource::Joker(_)))
        .unwrap();
    assert_eq!(duo.xmult, Some(2.0));
    assert_eq!(duo.mult_after, duo.mult_before * 2.0);
    assert!(trace
        .0
        .iter()
        .all(|step| matches!(step.source, ScoreSource::Joker(_)) || step.xmult.is_none()));
}

#[test]
fn polychrome_cards_are_xmult_and_gold_seals_earn_money() {
    let mut g = blind_game();
    let mut a = Card::new(Value::Ace, Suit::Heart);
    a.edition = Edition::Polychrome;
    let mut b = Card::new(Value::Ace, Suit::Club);
    b.seal = Some(Seal::Gold);
    let (_, trace) = g.calc_score_traced(SelectHand::new(vec![a, b]).best_hand().unwrap());
    let cards: Vec<_> = trace
        .0
        .iter()
        .filter(|step| matches!(step.source, ScoreSource::PlayedCard(_)))
        .collect();
    assert_eq!(cards.len(), 2);
    let poly = cards.iter().find(|step| step.xmult.is_some()).unwrap();
    assert_eq!(poly.xmult, Some(1.5));
    let gold = cards.iter().find(|step| step.money != 0).unwrap();
    assert_eq!(gold.money, 3);
    assert_eq!(gold.xmult, None);
}

#[test]
fn a_hand_level_step_is_additive() {
    let mut g = blind_game();
    let (_, trace) = g.calc_score_traced(pair().best_hand().unwrap());
    assert!(matches!(trace.0[0].source, ScoreSource::HandLevel(_)));
    assert_eq!(trace.0[0].xmult, None);
    assert_eq!(trace.0[0].money, 0);
}
