use super::*;
use crate::card::{Card, Suit, Value};

#[test]
fn test_gen_moves_play() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // nothing selected, nothing to play
    assert!(g.gen_actions_discard().is_none());

    g.available.extend(vec![ace]);
    g.select_card(ace).unwrap();
    let moves: Vec<Action> = g.gen_actions_play().expect("are plays").collect();
    assert_eq!(moves.len(), 1);

    g.available.extend(vec![ace, king]);
    g.select_card(ace).unwrap();
    g.select_card(king).unwrap();
    let moves: Vec<Action> = g.gen_actions_play().expect("are plays").collect();
    assert_eq!(moves.len(), 1);
}

#[test]
fn test_gen_moves_discard() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // nothing selected, nothing to discard
    assert!(g.gen_actions_discard().is_none());

    g.available.extend(vec![ace, king]);
    g.select_card(ace).unwrap();
    g.select_card(king).unwrap();
    let moves: Vec<Action> = g.gen_actions_discard().expect("are discards").collect();
    assert_eq!(moves.len(), 1);
}

#[test]
fn test_unmask_action_space_select_cards() {
    let mut g = Game::default();
    g.deal();
    g.stage = Stage::Blind(Blind::Small);
    let mut space = ActionSpace::from(g.config.clone());

    // Default action space everything should be masked
    assert!(space.select_card[0] == 0);

    // Unmask card selects, we have all selects available
    g.unmask_action_space_select_cards(&mut space);
    assert!(space.select_card[0] == 1);

    // Make a fresh space
    space = ActionSpace::from(g.config.clone());
    // Select 2 cards, regenerate action space
    for _ in 0..2 {
        g.select_card(*g.available.not_selected().first().expect("is first card"))
            .expect("can select");
    }
    g.unmask_action_space_select_cards(&mut space);
    // Cannot select first and second, can select third
    assert!(space.select_card[0] == 0);
    assert!(space.select_card[1] == 0);
    assert!(space.select_card[2] == 1);
}

#[test]
fn test_unmask_action_space_select_cards_max() {
    let mut g = Game::default();
    g.deal();
    g.stage = Stage::Blind(Blind::Small);
    let mut space = ActionSpace::from(g.config.clone());

    // Default action space everything should be masked
    assert!(space.select_card[0] == 0);

    // Unmask card selects, we have all selects available
    g.unmask_action_space_select_cards(&mut space);
    assert!(space.select_card[0] == 1);

    // Make a fresh space
    space = ActionSpace::from(g.config.clone());
    // Now select 5 cards, no more selects available, regenerate action space
    for _ in 0..g.config.selected_max {
        g.select_card(*g.available.not_selected().first().expect("is first card"))
            .expect("can select");
    }
    g.unmask_action_space_select_cards(&mut space);
    for i in 0..space.select_card.len() - 1 {
        assert!(space.select_card[i] == 0);
    }

    // If stage is not blind, don't alter space
    g.stage = Stage::Shop();
    space = ActionSpace::from(g.config.clone());
    space.select_card[0] = 1;
    assert!(space.select_card[0] == 1);
    g.unmask_action_space_select_cards(&mut space);
    assert!(space.select_card[0] == 1);
}

#[test]
fn test_unmask_action_space_play_and_discard() {
    let mut g = Game::default();
    g.deal();
    g.stage = Stage::Blind(Blind::Small);
    let mut space = ActionSpace::from(g.config.clone());

    // Default action space everything should be masked
    assert!(space.play[0] == 0);
    assert!(space.discard[0] == 0);

    for c in g.available.cards()[0..5].to_vec() {
        g.select_card(c).unwrap();
    }
    // Unmask play/discard
    g.unmask_action_space_play_and_discard(&mut space);
    assert!(space.play[0] == 1);
    assert!(space.discard[0] == 1);
}

#[test]
fn test_unmask_action_space_move_cards() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut space = ActionSpace::from(g.config.clone());

    // Default action space everything should be masked, since no cards available yet
    assert_eq!(g.available.cards().len(), 0);
    for i in 0..space.move_card_left.len() {
        assert!(space.move_card_left[i] == 0);
    }
    for i in 0..space.move_card_right.len() {
        assert!(space.move_card_right[i] == 0);
    }

    // deal and make available
    g.deal();
    // Unmask play/discard
    g.unmask_action_space_move_cards(&mut space);

    // Should be able to move left every available card except leftmost
    let available = g.available.cards().len();
    for i in 0..available - 1 {
        assert!(space.move_card_left[i] == 1);
    }
    for i in 0..available - 1 {
        assert!(space.move_card_right[i] == 1);
    }

    // Even when selected, we can still move cards
    let not_selected = g.available.not_selected();
    for c in &not_selected[0..5] {
        g.select_card(*c).unwrap();
    }

    // Get fresh action space and mask
    space = ActionSpace::from(g.config.clone());
    g.unmask_action_space_move_cards(&mut space);
    for i in 0..available - 1 {
        assert!(space.move_card_left[i] == 1);
    }
    for i in 0..available - 1 {
        assert!(space.move_card_right[i] == 1);
    }
}

#[test]
fn test_skip_blind_offered_when_next_is_small_or_big() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = crate::tag::Tag::Investment;

    // Fresh game: next blind is Small, skip should be offered by both APIs
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(actions.contains(&Action::SkipBlind(Blind::Small)));
    let space = g.gen_action_space();
    assert!(space.skip_blind[0] == 1);

    // Skip Small: next blind is Big, still offerable
    g.handle_action(Action::SkipBlind(Blind::Small))
        .expect("skip small blind");
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(actions.contains(&Action::SkipBlind(Blind::Big)));
    let space = g.gen_action_space();
    assert!(space.skip_blind[0] == 1);
}

#[test]
fn test_skip_blind_never_offered_for_boss() {
    let mut g = Game::default();
    g.start();
    g.blind = Some(Blind::Big);

    // Next blind is Boss: neither API should offer SkipBlind
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(!actions.iter().any(|a| matches!(a, Action::SkipBlind(_))));
    let space = g.gen_action_space();
    assert!(space.skip_blind[0] == 0);
}
