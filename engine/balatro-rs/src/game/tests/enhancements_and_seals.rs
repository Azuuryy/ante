use super::*;

#[test]
fn test_manacle_boss_deals_reduced_hand() {
    let mut g = Game {
        current_boss: Some(BossBlind::Manacle),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    assert_eq!(g.available.cards().len(), 7);
}

#[test]
fn test_manacle_boss_no_effect_outside_boss_round() {
    let g = Game {
        current_boss: Some(BossBlind::Manacle),
        blind: Some(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.hand_size(), 8);
}

#[test]
fn test_serpent_boss_grows_hand_when_fewer_than_three_removed() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(2).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 9);
}

#[test]
fn test_serpent_boss_shrinks_hand_when_more_than_three_removed() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(5).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 6);
}

#[test]
fn test_serpent_boss_applies_to_play_selected_too() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let card = g.available.cards()[0];
    g.select_card(card).expect("can select card");
    g.play_selected().expect("can play selected");
    assert_eq!(g.available.cards().len(), 10);
}

#[test]
fn test_serpent_boss_no_effect_outside_boss_round() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: None,
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Small)
        .expect("can select small blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(2).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 8);
}

#[test]
fn test_play_selected() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    for card in g.available.cards().iter().take(5) {
        g.available.select_card(*card).expect("can select card");
    }

    assert_eq!(g.available.selected().len(), 5);
    // Artifically set score so blind passes
    g.score += g.required_score();
    g.play_selected().expect("can play selected");

    // Should have cleared blind
    assert_eq!(g.stage, Stage::PostBlind());
    // Score should reset to 0
    assert_eq!(g.score, g.config.base_score);
    // Plays and discards should reset
    assert_eq!(g.plays, g.config.plays);
    assert_eq!(g.discards_remaining, g.config.discards);
    // All cards back in deck after clear_blind, available is empty
    assert_eq!(g.deck.len(), 52);
    // Discarded should be length 0
    assert_eq!(g.discarded.len(), 0);
    // Available is empty until next blind starts
    assert_eq!(g.available.cards().len(), 0);
}

#[test]
fn test_buy_joker() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    let j1 = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![j1.clone()];
    g.buy_joker(j1.clone()).expect("buy joker");
    assert_eq!(g.money, 10 - j1.cost());
    assert_eq!(g.jokers.len(), 1);
}

#[test]
fn test_enhancement_bonus() {
    // High card Ace with Bonus: (5 + 11 + 30) * 1 = 46
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Bonus);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 46);
}

#[test]
fn test_enhancement_mult() {
    // High card Ace with Mult: (5 + 11) * (1 + 4) = 80
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Mult);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 80);
}

#[test]
fn test_enhancement_glass() {
    // High card Ace with Glass: (5 + 11) * (1 * 2) = 32
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Glass);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 32);
}

#[test]
fn test_enhancement_stone_as_kicker() {
    // [Ace, Stone] -> High Card Ace; Stone kicker always scores +50
    // (5 + 11 + 50) * 1 = 66
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 66);
}

#[test]
fn test_enhancement_steel_held() {
    // Pair of Kings scored; Steel King held (not played)
    // Pair level 1: chips=10, mult=2; both Kings score: chips=10+10+10=30
    // Steel held: mult = floor(2 * 1.5) = 3 -> score = 30 * 3 = 90
    let mut g = Game::default();
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_king = Card::new(Value::King, Suit::Spade);
    steel_king.enhancement = Some(Enhancement::Steel);
    g.available.extend(vec![steel_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 90);
}

#[test]
fn test_enhancement_gold_held() {
    // Gold card held in unplayed hand gives $3 per hand scored
    let mut g = Game::default();
    let mut gold_card = Card::new(Value::Two, Suit::Heart);
    gold_card.enhancement = Some(Enhancement::Gold);
    g.available.extend(vec![gold_card]);
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, 3);
}

#[test]
fn test_seal_gold_scored() {
    // Gold Seal: $3 when this card is played and scores (score itself unaffected)
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Spade);
    ace.seal = Some(Seal::Gold);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 16);
    assert_eq!(g.money, 3);
}

#[test]
fn test_seal_red_retrigger_played() {
    // Red Seal: retrigger this card's played abilities.
    // High card Ace, chips scored twice: (5 + 11 + 11) * 1 = 27
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Spade);
    ace.seal = Some(Seal::Red);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 27);
}

#[test]
fn test_seal_red_retrigger_gold_held() {
    // Red Seal retriggers held-card abilities too, not just played ones:
    // a held Gold+Red card earns $6 (2x $3), not $3.
    let mut g = Game::default();
    let mut gold_red = Card::new(Value::Two, Suit::Heart);
    gold_red.enhancement = Some(Enhancement::Gold);
    gold_red.seal = Some(Seal::Red);
    g.available.extend(vec![gold_red]);
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, 6);
}

#[test]
fn test_seal_red_retrigger_steel_held() {
    // Pair of Kings scored; held Steel+Red King retriggers its Mult bonus twice.
    // Pair level 1: chips=10, mult=2; both Kings score: chips=30
    // Steel triggers twice: mult = 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5))
    // score = 30 * 4 = 120
    let mut g = Game::default();
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_red_king = Card::new(Value::King, Suit::Spade);
    steel_red_king.enhancement = Some(Enhancement::Steel);
    steel_red_king.seal = Some(Seal::Red);
    g.available.extend(vec![steel_red_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // Steel held twice: 2 * 1.5 * 1.5 = 4.5; (10 + 20) * 4.5 = 135
    assert_eq!(g.calc_score(hand), 135);
}

#[test]
fn test_stone_kicker_red_seal_retrigger() {
    // [Ace, Stone+Red Seal] -> High Card Ace; stone kicker retriggers twice.
    // (5 + 11 + 50 + 50) * 1 = 116
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    stone.seal = Some(Seal::Red);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 116);
}

#[test]
fn test_hanging_chad_retriggers_first_played_card() {
    use crate::joker::*;
    // [Ace, Stone], Ace first -> Ace retriggers 3x, stone kicker only 1x.
    // (5 + 11*3 + 50) * 1 = 88
    let mut g = Game::default();
    g.jokers.push(Jokers::HangingChad(HangingChad::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 88);
}

#[test]
fn test_hanging_chad_stone_kicker_competes_for_first() {
    use crate::joker::*;
    // Same cards as above, stone selected first this time -> the stone
    // kicker gets HangingChad's retrigger instead of the Ace.
    // (5 + 11 + 50*3) * 1 = 166
    let mut g = Game::default();
    g.jokers.push(Jokers::HangingChad(HangingChad::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![stone, ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 166);
}

#[test]
fn test_dusk_no_retrigger_with_plays_remaining() {
    use crate::joker::*;
    let mut g = Game::default();
    g.jokers.push(Jokers::Dusk(Dusk::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    g.plays = 1;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 16);
}

#[test]
fn test_dusk_retriggers_on_final_hand() {
    use crate::joker::*;
    // Same hand as above, but this is the last play of the round:
    // (5 + 11 + 11) * 1 = 27
    let mut g = Game::default();
    g.jokers.push(Jokers::Dusk(Dusk::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    g.plays = 0;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 27);
}

#[test]
fn test_hack_retriggers_only_low_ranks() {
    use crate::joker::*;
    // Two Pair (Threes + Kings): Hack retriggers each Three, not the Kings.
    let cards = vec![
        Card::new(Value::Three, Suit::Heart),
        Card::new(Value::Three, Suit::Diamond),
        Card::new(Value::King, Suit::Spade),
        Card::new(Value::King, Suit::Club),
    ];

    let mut base_game = Game::default();
    let base_hand = SelectHand::new(cards.clone()).best_hand().unwrap();
    let base = base_game.calc_score(base_hand);

    let mut g = Game::default();
    g.jokers.push(Jokers::Hack(Hack::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let hand = SelectHand::new(cards).best_hand().unwrap();
    // Each Three retriggers once more: +3 chips * 2 cards = +6 chips,
    // scaled by TwoPair's X2 mult -> +12 final score.
    assert_eq!(g.calc_score(hand), base + 12);
}
