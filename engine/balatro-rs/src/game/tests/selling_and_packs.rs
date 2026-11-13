use super::*;

#[cfg(feature = "serde")]
#[test]
fn test_from_json_invalid() {
    let result = Game::from_json("not valid json");
    assert!(result.is_err());
}

#[cfg(feature = "serde")]
#[test]
fn test_from_json_advances_id_counter_past_loaded_ids() {
    let mut g = Game::default();
    let mut high_id_card = Card::new(Value::King, Suit::Spade);
    high_id_card.id = 999_999;
    g.deck.push(high_id_card);

    let json = g.to_json().expect("serialize");
    let _g2 = Game::from_json(&json).expect("deserialize");

    // Card::new() after loading must not collide with the loaded id.
    let new_card = Card::new(Value::Two, Suit::Heart);
    assert!(new_card.id > 999_999);
}

#[test]
fn test_glass_shatter_preserves_hand_size() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);

    // Replace available with controlled cards so we have exactly one glass card.
    // Use mixed values/suits to ensure the hand scores below the blind threshold
    // so clear_blind() / deal() is not triggered mid-test.
    g.available.empty();
    // Glass card is ace (highest) so it's selected as the high card and scores.
    // high card ace scores (5+11)*2 = 32, below 300 small blind threshold.
    let mut glass = Card::new(Value::Ace, Suit::Heart);
    glass.enhancement = Some(Enhancement::Glass);
    let glass_id = glass.id;
    let others = vec![
        Card::new(Value::Two, Suit::Club),
        Card::new(Value::Four, Suit::Diamond),
        Card::new(Value::Six, Suit::Spade),
        Card::new(Value::Eight, Suit::Club),
        Card::new(Value::Ten, Suit::Diamond),
        Card::new(Value::Queen, Suit::Spade),
    ];
    let mut hand_cards = vec![glass];
    hand_cards.extend(others);
    g.available.extend(hand_cards);

    let hand_size_before = g.available.cards().len();

    // Force 100% shatter probability (only needs to be 4 but overkill is ok)
    g.prob_mult = 100;

    // Select glass + 4 others (5-card hand)
    let to_select: Vec<Card> = g.available.cards().into_iter().take(5).collect();
    for card in to_select {
        g.available.select_card(card).expect("select card");
    }

    g.play_selected().expect("play selected");

    // Hand size must be unchanged
    assert_eq!(g.available.cards().len(), hand_size_before);
    // Glass card must be permanently gone (not hiding in discarded to return next deal)
    assert!(g.discarded.iter().all(|c| c.id != glass_id));
}

#[test]
fn test_targeting_tarot_in_blind_invalid_selection_preserves_consumable() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    // no cards selected — Magician needs at least 1
    g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
    let res = g.use_consumable(Consumable::Tarot(Tarot::Magician));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables.len(), 1);
    assert!(g.consumables.contains(&Consumable::Tarot(Tarot::Magician)));
}

#[test]
fn test_targeting_tarot_in_blind_too_many_selected_preserves_consumable() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    // Lovers accepts exactly 1 — select 2 to exceed max
    let cards = g.available.cards();
    g.available.select_card(cards[0]).unwrap();
    g.available.select_card(cards[1]).unwrap();
    g.consumables = vec![Consumable::Tarot(Tarot::Lovers)];
    let res = g.use_consumable(Consumable::Tarot(Tarot::Lovers));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables.len(), 1);
}

#[test]
fn test_sell_joker() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);
    assert_eq!(g.jokers.len(), 1);

    g.sell_joker(0).expect("sell joker");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_joker_invalid_index() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.sell_joker(0);
    assert!(matches!(res, Err(GameError::InvalidAction)));
}

#[test]
fn test_sell_joker_invalid_stage() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::End(crate::stage::End::Win);
    g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    let res = g.sell_joker(0);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_sell_joker_during_blind() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);

    g.sell_joker(0).expect("sell joker during blind");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_luchador_disables_boss_only_for_itself() {
    use crate::joker::Jokers;
    use balatro_types::joker::{Luchador, TheJoker};
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    g.jokers.push(Jokers::Luchador(Luchador::default()));

    // Selling an unrelated joker must not trigger Luchador's OnSell effect.
    g.sell_joker(0).expect("sell plain joker");
    assert!(!g.boss_disabled_by_luchador);

    g.sell_joker(0).expect("sell luchador");
    assert!(g.boss_disabled_by_luchador);
}

#[test]
fn test_sell_luchador_restores_water_boss_discards_via_handle_action() {
    use crate::joker::Jokers;
    use balatro_types::joker::Luchador;
    let mut g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Boss),
        stage: Stage::Blind(Blind::Boss),
        discards_remaining: 4,
        jokers: vec![Jokers::Luchador(Luchador::default())],
        ..Default::default()
    };
    assert_eq!(g.discards(), 0);

    g.handle_action(Action::SellJoker(0))
        .expect("sell luchador via handle_action");
    assert_eq!(g.discards(), 4);
}

#[test]
fn test_sell_consumable() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let c = Consumable::Planet(Planets::Mercury);
    let sell_value = c.sell_value();
    g.consumables.push(c);
    assert_eq!(g.consumables.len(), 1);

    g.sell_consumable(0).expect("sell consumable");
    assert_eq!(g.consumables.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_consumable_invalid_index() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.sell_consumable(0);
    assert!(matches!(res, Err(GameError::InvalidAction)));
}

#[test]
fn test_sell_consumable_invalid_stage() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::End(crate::stage::End::Win);
    g.consumables
        .push(Consumable::Tarot(crate::tarot::Tarot::Fool));
    let res = g.sell_consumable(0);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_sell_joker_during_tarot_hand() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::TarotHand(Tarot::Magician);
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);

    g.sell_joker(0).expect("sell joker during tarot hand");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_consumable_during_tarot_hand() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::TarotHand(Tarot::Magician);
    g.money = 0;
    let c = Consumable::Planet(Planets::Mercury);
    let sell_value = c.sell_value();
    g.consumables.push(c);

    g.sell_consumable(0)
        .expect("sell consumable during tarot hand");
    assert_eq!(g.consumables.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_joker_removes_effects() {
    use crate::joker::Jokers;
    use balatro_types::joker::HangingChad;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let joker = Jokers::HangingChad(HangingChad::default());
    g.jokers.push(joker);
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    assert!(!g.effect_registry.trigger_count_played.is_empty());

    g.sell_joker(0).expect("sell joker");
    assert!(g.effect_registry.trigger_count_played.is_empty());
}

#[test]
fn test_buy_pack() {
    use crate::pack::{Pack, PackCategory, PackSize};
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 100;
    let pack = Pack {
        category: PackCategory::Celestial,
        size: PackSize::Normal,
        contents: vec![PackContent::Planet(Planets::Mercury)],
    };
    let cost = pack.cost();
    let picks = pack.picks_allowed();
    g.shop.packs.push(pack.clone());
    g.buy_pack(pack).expect("buy pack");
    assert_eq!(g.stage, Stage::PackOpen());
    assert_eq!(g.money, 100 - cost);
    let state = g.open_pack.as_ref().expect("open pack state");
    assert_eq!(state.picks_remaining, picks);
}

#[test]
fn test_skip_pack() {
    let mut g = Game::default();
    g.start();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![],
    });
    g.stage = Stage::PackOpen();
    g.skip_pack().expect("skip pack");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_skip_blind_small() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Investment;
    assert!(g.tags.is_empty());
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.blind, Some(Blind::Small));
    assert_eq!(g.stage, Stage::PreBlind());
    assert_eq!(g.tags.len(), 1);
}

#[test]
fn test_skip_blind_wrong_blind() {
    let mut g = Game::default();
    g.start();
    // Game just started, next expected blind is Small, not Big
    let res = g.skip_blind(Blind::Big);
    assert!(matches!(res, Err(GameError::InvalidBlind)));
}

#[test]
fn test_skip_blind_boss_rejected() {
    let mut g = Game::default();
    g.start();
    g.blind = Some(Blind::Big);
    let res = g.skip_blind(Blind::Boss);
    assert!(matches!(res, Err(GameError::InvalidBlind)));
}
