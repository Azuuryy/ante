use super::*;

#[test]
fn test_planetarium_level_up_changes_scoring() {
    let mut g = Game::default();
    let before = g.planetarium.level(HandRank::OnePair);
    g.planetarium.level_up(HandRank::OnePair);
    let after = g.planetarium.level(HandRank::OnePair);
    assert_eq!(after.chips, before.chips + 15);
    assert_eq!(after.mult, before.mult + 1);
    assert_eq!(after.level, before.level + 1);
}

#[test]
fn test_buy_consumable() {
    let mut g = Game::default();
    g.start();
    g.money = 100;
    g.stage = Stage::Shop();
    g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Mercury)))
        .unwrap();
    assert_eq!(g.consumables.len(), 1);
    assert_eq!(g.money, 97);
    assert!(g.shop.consumables.is_empty());
}

#[test]
fn test_use_consumable_planet() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let before = g.planetarium.level(HandRank::OnePair);
    g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)))
        .unwrap();
    assert!(g.consumables.is_empty());
    let after = g.planetarium.level(HandRank::OnePair);
    assert_eq!(after.chips, before.chips + 15);
    assert_eq!(after.mult, before.mult + 1);
}

#[test]
fn test_use_consumable_in_postblind() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::PostBlind();
    g.consumables = vec![Consumable::Planet(Planets::Jupiter)];
    let before = g.planetarium.level(HandRank::Flush);
    g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Jupiter)))
        .unwrap();
    assert!(g.consumables.is_empty());
    let after = g.planetarium.level(HandRank::Flush);
    assert_eq!(after.chips, before.chips + 15);
    assert_eq!(after.mult, before.mult + 2);
}

#[test]
fn test_buy_consumable_slots_full() {
    let mut g = Game::default();
    g.start();
    g.money = 100;
    g.stage = Stage::Shop();
    g.consumables = vec![
        Consumable::Planet(Planets::Pluto),
        Consumable::Planet(Planets::Mercury),
    ];
    g.shop.consumables = vec![Consumable::Planet(Planets::Venus)];
    let res = g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Venus)));
    assert!(matches!(res, Err(GameError::NoAvailableSlot)));
}

#[test]
fn test_buy_consumable_insufficient_funds() {
    let mut g = Game::default();
    g.start();
    g.money = 0;
    g.stage = Stage::Shop();
    g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let res = g.handle_action(Action::BuyConsumable(Consumable::Planet(Planets::Mercury)));
    assert!(matches!(res, Err(GameError::InvalidBalance)));
}

#[test]
fn test_use_consumable_valid_in_blind() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(crate::stage::Blind::Small);
    g.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let before = g.planetarium.level(HandRank::OnePair);
    let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)));
    assert!(res.is_ok());
    assert_eq!(
        g.planetarium.level(HandRank::OnePair).chips,
        before.chips + 15
    );
}

#[test]
fn test_use_consumable_valid_in_preblind() {
    let mut g = Game::default();
    g.start();
    g.consumables = vec![Consumable::Planet(Planets::Saturn)];
    let before = g.planetarium.level(HandRank::Straight);
    let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Saturn)));
    assert!(res.is_ok());
    assert_eq!(
        g.planetarium.level(HandRank::Straight).chips,
        before.chips + 30
    );
}

#[test]
fn test_use_consumable_invalid_at_end() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::End(crate::stage::End::Win);
    g.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let res = g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)));
    assert!(matches!(res, Err(GameError::InvalidAction)));
}

#[test]
fn test_gen_actions_buy_consumable_in_shop() {
    let mut g = Game::default();
    g.start();
    g.money = 100;
    g.stage = Stage::Shop();
    g.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(actions.contains(&Action::BuyConsumable(Consumable::Planet(Planets::Mercury))));
}

#[test]
fn test_gen_actions_use_consumable_in_shop() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.consumables = vec![Consumable::Planet(Planets::Earth)];
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(actions.contains(&Action::UseConsumable(Consumable::Planet(Planets::Earth))));
}

#[test]
fn test_gen_actions_use_consumable_in_blind() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(crate::stage::Blind::Small);
    g.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(actions
        .iter()
        .any(|a| matches!(a, Action::UseConsumable(_))));
    // buy is still shop-only
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::BuyConsumable(_))));
}

#[test]
// Test executing a full game using the gen_actions api
fn test_game_gen_actions() {
    let mut g = Game::default();

    g.start();
    while !g.is_over() {
        // Get all available actions
        let actions: Vec<Action> = g.gen_actions().collect();
        if actions.is_empty() {
            break;
        }

        // Pick a random move and execute it
        let i = rand::thread_rng().gen_range(0..actions.len());
        let action = actions[i].clone();
        dbg!("game state:\n{}", g.clone());
        dbg!("play action: {}", action.clone());
        let action_res = g.handle_action(action.clone());
        dbg!(action);
        assert!(action_res.is_ok());
    }
    let result = g.result();
    // Ensure game is over at end
    assert!(result.is_some());
    // Check game state at end
    assert!(matches!(g.stage, Stage::End(_)));
    dbg!("game action history: {:?}", g.action_history);
}

#[test]
fn test_use_consumable_hand_touching_tarot_illegal_from_shop() {
    use crate::tarot::Tarot;
    let mut g = Game::default();
    g.start();
    // at Shop, available is empty (clear_blind returns cards to deck without re-dealing)
    g.stage = Stage::Shop();
    assert_eq!(g.available.cards().len(), 0);
    g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
    let res = g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Magician)));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    // Consumable is not consumed on a failed use, and no substitute hand is dealt.
    assert_eq!(g.consumables, vec![Consumable::Tarot(Tarot::Magician)]);
    assert_eq!(g.stage, Stage::Shop());
    assert_eq!(g.available.cards().len(), 0);
}

#[test]
fn test_use_consumable_hand_touching_spectral_illegal_from_shop() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    assert_eq!(g.available.cards().len(), 0);
    g.consumables = vec![Consumable::Spectral(Spectral::Talisman)];
    let res = g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Talisman,
    )));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(
        g.consumables,
        vec![Consumable::Spectral(Spectral::Talisman)]
    );
    assert_eq!(g.stage, Stage::Shop());
    assert_eq!(g.available.cards().len(), 0);
}

// Fool copies the last Tarot or Planet used, never a Spectral (matches its
// own description text and real Balatro). Using a Spectral in between must
// not overwrite last_consumable_used, and Fool must never duplicate it.
#[test]
fn test_fool_does_not_copy_spectral_used_via_consumable() {
    use crate::spectral::Spectral;
    use crate::tarot::Tarot;

    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.config.consumable_slots = 3;

    g.consumables = vec![Consumable::Planet(Planets::Mercury)];
    g.handle_action(Action::UseConsumable(Consumable::Planet(Planets::Mercury)))
        .unwrap();
    assert_eq!(
        g.last_consumable_used,
        Some(Consumable::Planet(Planets::Mercury))
    );

    // Wraith is a 0-target Spectral that doesn't touch the hand, so it's
    // legal from Shop.
    g.consumables.push(Consumable::Spectral(Spectral::Wraith));
    g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Wraith,
    )))
    .unwrap();
    // Using the Spectral must not overwrite what Fool remembers.
    assert_eq!(
        g.last_consumable_used,
        Some(Consumable::Planet(Planets::Mercury))
    );

    g.consumables.push(Consumable::Tarot(Tarot::Fool));
    g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Fool)))
        .unwrap();
    assert!(g
        .consumables
        .contains(&Consumable::Planet(Planets::Mercury)));
    assert!(
        !g.consumables
            .iter()
            .any(|c| matches!(c, Consumable::Spectral(_))),
        "Fool must never duplicate a Spectral"
    );
}

// A 0-target Spectral that still destroys a random card from the hand
// (see requires_hand()) must not be usable from Shop, where the hand is
// empty — this is the crash this gate exists to prevent.
#[test]
fn test_use_consumable_zero_target_hand_touching_spectral_illegal_from_shop() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    assert_eq!(g.available.cards().len(), 0);
    g.consumables = vec![Consumable::Spectral(Spectral::Familiar)];
    let res = g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Familiar,
    )));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(
        g.consumables,
        vec![Consumable::Spectral(Spectral::Familiar)]
    );

    let actions: Vec<Action> = g.gen_actions().collect();
    assert!(
        !actions.contains(&Action::UseConsumable(Consumable::Spectral(
            Spectral::Familiar
        )))
    );
}

#[test]
fn test_gen_actions_use_consumable_spectral_in_shop() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.consumables = vec![Consumable::Spectral(Spectral::Sigil)];
    let actions: Vec<Action> = g.gen_actions().collect();
    // Sigil is 0-target but still touches the hand, so it's not offered from Shop.
    assert!(
        !actions.contains(&Action::UseConsumable(Consumable::Spectral(
            Spectral::Sigil
        )))
    );
}

#[test]
fn test_pack_tarot_is_offered_only_for_a_target_count_it_accepts() {
    // The hand keeps its own highlight limit in a pack; The Magician (one or two
    // targets) is offered only while the selection fits, like can_use_consumeable.
    use crate::pack::{Pack, PackCategory, PackContent, PackSize};
    use crate::tarot::Tarot;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 100;
    let pack = Pack {
        category: PackCategory::Arcana,
        size: PackSize::Normal,
        contents: vec![PackContent::Tarot(Tarot::Magician)],
    };
    g.shop.packs = vec![pack.clone()];
    g.handle_action(Action::BuyPack(pack)).unwrap();
    assert_eq!(g.stage, Stage::PackOpen());
    let magician = Action::PickPackCard(PackContent::Tarot(Tarot::Magician));
    assert!(!g.gen_actions().any(|a| a == magician));
    let cards = g.available.cards();
    g.handle_action(Action::SelectCard(cards[0])).unwrap();
    g.handle_action(Action::SelectCard(cards[1])).unwrap();
    assert!(g.gen_actions().any(|a| a == magician));
    g.handle_action(Action::SelectCard(cards[2])).unwrap();
    assert!(!g.gen_actions().any(|a| a == magician));
    assert!(matches!(
        g.handle_action(magician.clone()),
        Err(GameError::InvalidAction)
    ));
    g.handle_action(Action::DeselectCard(cards[2])).unwrap();
    g.handle_action(magician).unwrap();
    assert_eq!(g.stage, Stage::Shop());
}
