// Jokers that act on each scored or held card, after the original `evaluate_play`: every
// repetition of a card runs the card's own effects, then each Joker's
// `calculate_joker{individual = true}` on that card.

use crate::card::{Card, Suit, Value};
use crate::game::Game;
use crate::hand::SelectHand;
use crate::joker::{Baron, Jokers, Mime, Pareidolia, ScaryFace, SmileyFace, SockAndBuskin};
use crate::score::ScoreSource;
use crate::stage::{Blind, Stage};

fn blind_game(jokers: Vec<Jokers>) -> Game {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        jokers,
        ..Default::default()
    };
    let snapshot = g.clone();
    g.effect_registry
        .register_jokers(snapshot.jokers.clone(), &snapshot);
    g
}

fn heart_flush() -> SelectHand {
    SelectHand::new(
        [
            Value::Two,
            Value::Four,
            Value::Six,
            Value::Eight,
            Value::Ten,
        ]
        .into_iter()
        .map(|value| Card::new(value, Suit::Heart))
        .collect(),
    )
}

fn face_jokers() -> Vec<Jokers> {
    vec![
        Jokers::ScaryFace(ScaryFace::default()),
        Jokers::SmileyFace(SmileyFace::default()),
        Jokers::SockAndBuskin(SockAndBuskin::default()),
        Jokers::Pareidolia(Pareidolia::default()),
    ]
}

#[test]
fn face_jokers_trigger_on_every_repetition_of_every_card() {
    // Pareidolia makes all five cards faces; Sock and Buskin repeats each once: ten
    // scorings. Flush level 1 is 35 chips × 4 Mult; the cards give 2 × 30 chips,
    // Scary Face 10 × 30 chips and Smiley Face 10 × 5 Mult.
    let mut g = blind_game(face_jokers());
    let score = g.calc_score(heart_flush().best_hand().unwrap());
    assert_eq!(score, (35 + 60 + 300) * (4 + 50));
}

#[test]
fn a_joker_step_follows_the_card_it_scores() {
    let mut g = blind_game(face_jokers());
    let (_, trace) = g.calc_score_traced(heart_flush().best_hand().unwrap());
    let scary_steps = trace
        .0
        .iter()
        .filter(|step| {
            matches!(
                step.source,
                ScoreSource::JokerOnCard(Jokers::ScaryFace(_), _)
            )
        })
        .count();
    assert_eq!(scary_steps, 10);
    let first = trace
        .0
        .iter()
        .position(|step| matches!(step.source, ScoreSource::PlayedCard(_)))
        .unwrap();
    let ScoreSource::PlayedCard(card) = trace.0[first].source else {
        unreachable!()
    };
    assert!(matches!(
        trace.0[first + 1].source,
        ScoreSource::JokerOnCard(Jokers::ScaryFace(_), on) if on.id == card.id
    ));
    // The repetition comes after both Jokers scored the first pass of that card.
    assert!(matches!(
        trace.0[first + 3].source,
        ScoreSource::PlayedCard(again) if again.id == card.id
    ));
    assert!(trace.0[first + 3].retrigger);
}

#[test]
fn mime_repeats_baron_on_a_held_king() {
    // High Card Two: 5 + 2 chips × 1 Mult; Baron × 1.5 per held King, twice with Mime.
    let mut g = blind_game(vec![
        Jokers::Baron(Baron::default()),
        Jokers::Mime(Mime::default()),
    ]);
    g.available
        .extend(vec![Card::new(Value::King, Suit::Spade)]);
    let two = SelectHand::new(vec![Card::new(Value::Two, Suit::Heart)]);
    let score = g.calc_score(two.best_hand().unwrap());
    assert_eq!(score, (7.0_f64 * 1.5 * 1.5).floor() as usize);
}
