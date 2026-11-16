//! The Hook, The Tooth and The Ox: the boss abilities applied when a hand is played.
use super::*;

/// A Boss Blind of `boss` in progress, with a dealt hand and its first `played` cards
/// selected.
fn boss_round(boss: BossBlind, played: usize) -> Game {
    let mut g = Game {
        current_boss: Some(boss),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    for card in g.available.cards()[0..played].to_vec() {
        g.select_card(card).expect("can select card");
    }
    g
}

#[test]
fn the_hook_discards_two_held_cards_and_refills_the_hand() {
    let mut g = boss_round(BossBlind::Hook, 1);
    let held_before = g.available.not_selected();
    let size = g.available.cards().len();
    g.play_selected().expect("can play");
    assert_eq!(g.discarded_this_round.len(), 2);
    for card in &g.discarded_this_round {
        assert!(held_before.iter().any(|held| held.id == card.id));
    }
    assert_eq!(g.discards(), g.discards_per_round(), "no discard is spent");
    assert_eq!(g.available.cards().len(), size, "the hand is refilled");
}

#[test]
fn the_tooth_costs_a_dollar_per_card_played() {
    let mut g = boss_round(BossBlind::Tooth, 3);
    g.money = 10;
    g.play_selected().expect("can play");
    assert_eq!(g.money, 7);
}

#[test]
fn the_hook_and_the_tooth_pay_matador() {
    for boss in [BossBlind::Hook, BossBlind::Tooth] {
        let mut g = boss_round(boss, 1);
        g.jokers = vec![Jokers::Matador(crate::joker::Matador::default())];
        g.money = 10;
        let spent = if boss == BossBlind::Tooth { 1 } else { 0 };
        g.play_selected().expect("can play");
        assert_eq!(g.money, 10 - spent + 8, "{boss:?}");
    }
}

#[test]
fn the_ox_empties_the_wallet_on_the_most_played_hand() {
    let mut g = boss_round(BossBlind::Ox, 0);
    g.ox_hand = Some(HandRank::OnePair);
    g.money = 12;
    let pair = SelectHand::new(vec![
        Card::new(Value::King, Suit::Heart),
        Card::new(Value::King, Suit::Club),
    ])
    .best_hand()
    .unwrap();
    g.calc_score(pair);
    assert_eq!(g.money, 0);
    assert!(g.boss_triggered_this_hand);

    g.money = 12;
    let high = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();
    g.calc_score(high);
    assert_eq!(g.money, 12);
}

#[test]
fn the_ox_reads_high_card_until_a_boss_records_the_most_played_hand() {
    let mut g = Game {
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.planetarium.play(HandRank::Flush);
    g.planetarium.play(HandRank::OnePair);
    g.planetarium.play(HandRank::OnePair);
    g.planetarium.play(HandRank::Flush);
    assert_eq!(g.ox_hand, None);
    g.remember_most_played_hand();
    // A tie goes to the stronger hand, like the lower `order` in `end_round`.
    assert_eq!(g.ox_hand, Some(HandRank::Flush));
    // `handle_score` records it when a Boss Blind is beaten: see `boss_runs`.
}
