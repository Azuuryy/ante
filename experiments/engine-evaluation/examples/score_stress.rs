use std::{
    hint::black_box,
    sync::{Arc, Mutex},
    time::Instant,
};

use balatro_rs::{
    card::{Card, Edition, Enhancement, Seal, Suit, Value},
    config::{Config, RngMode},
    effect::Effects,
    game::Game,
    hand::{MadeHand, SelectHand},
    joker::Jokers,
    score::ScoreSource,
};

const BATCHES: usize = 9;

fn fixture(enhancement: Enhancement, extra_retriggers: usize) -> (Game, MadeHand) {
    let mut game = Game::new(Config {
        seed: Some(0),
        rng_mode: RngMode::Fast,
        ..Config::default()
    });
    let mut joker = Jokers::TheJoker(Default::default());
    joker.set_edition(Edition::Polychrome);
    game.jokers.push(joker);
    if extra_retriggers != 0 {
        game.effect_registry
            .trigger_count_played
            .push(Effects::TriggerCountPlayed(Arc::new(Mutex::new(
                move |_: &mut Game, _: Card, _: bool| extra_retriggers,
            ))));
    }

    let cards = [
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
    ]
    .map(|value| {
        let mut card = Card::new(value, Suit::Spade);
        card.enhancement = Some(enhancement);
        card.seal = Some(Seal::Red);
        card
    });
    let hand = SelectHand::new(cards.to_vec()).best_hand().unwrap();
    (game, hand)
}

fn median_micros(iterations: usize, mut score: impl FnMut()) -> f64 {
    for _ in 0..16 {
        score();
    }
    let mut times = [0.0; BATCHES];
    for time in &mut times {
        let start = Instant::now();
        for _ in 0..iterations {
            score();
        }
        *time = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    }
    times.sort_by(f64::total_cmp);
    times[BATCHES / 2]
}

fn benchmark(enhancement: Enhancement, extra_retriggers: usize, iterations: usize) {
    let (mut game, hand) = fixture(enhancement, extra_retriggers);
    let (score, trace) = game.calc_score_traced(hand.clone());
    let card_steps = trace.0.iter().filter(|step| {
        matches!(step.source, ScoreSource::PlayedCard(_)) && step.mult_before != step.mult_after
    });
    let triggers = 5 * (2 + extra_retriggers);
    assert_eq!(card_steps.clone().count(), triggers);
    assert_eq!(
        card_steps.filter(|step| step.retrigger).count(),
        triggers - 5
    );
    let chips = 100 + 20 * (2 + extra_retriggers);
    let mult = match enhancement {
        Enhancement::Mult => 8 + 4 * triggers,
        Enhancement::Glass => 8 * 2usize.pow(triggers.try_into().unwrap()),
        _ => unreachable!(),
    } + 4;
    assert_eq!(score, chips * (mult + mult / 2));
    assert_eq!(game.calc_score(hand.clone()), score);

    let plain = median_micros(iterations, || {
        black_box(game.calc_score(black_box(hand.clone())));
    });
    let traced = median_micros(iterations, || {
        black_box(game.calc_score_traced(black_box(hand.clone())));
    });
    println!(
        "{enhancement:?}: {triggers} card triggers, {} trace steps, score {score} | {plain:.2} us plain, {traced:.2} us traced",
        trace.0.len(),
    );

    if enhancement == Enhancement::Glass {
        println!(
            "1000 hands, {} x2 effects: {:.2} ms plain, {:.2} ms traced",
            1000 * triggers,
            plain,
            traced,
        );
    }
}

fn main() {
    benchmark(Enhancement::Mult, 0, 4096);
    benchmark(Enhancement::Mult, 2_000, 128);
    benchmark(Enhancement::Mult, 10_000, 32);
    benchmark(Enhancement::Mult, 20_000, 16);
    benchmark(Enhancement::Glass, 0, 1000);
}
