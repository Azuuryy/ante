use super::*;

#[test]
fn test_pack_spectral_select_then_use_round_trip() {
    use crate::card::Seal;
    use crate::pack::{Pack, PackCategory, PackContent, PackSize};
    use crate::spectral::Spectral;

    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 100;
    let pack = Pack {
        category: PackCategory::Spectral,
        size: PackSize::Normal,
        contents: vec![PackContent::Spectral(Spectral::Talisman)],
    };
    g.shop.packs = vec![pack.clone()];
    g.handle_action(Action::BuyPack(pack)).unwrap();
    let talisman = Action::PickPackCard(PackContent::Spectral(Spectral::Talisman));
    assert!(!g.gen_actions().any(|a| a == talisman));

    let card = g.available.cards()[0];
    g.handle_action(Action::SelectCard(card)).unwrap();
    assert!(g.gen_actions().any(|a| a == talisman));

    g.handle_action(talisman).unwrap();
    // The Normal pack's single pick is used, so finish_pack returned to the Shop and
    // sent the drawn hand back to the deck.
    assert_eq!(g.stage, Stage::Shop());
    assert_eq!(g.available.cards().len(), 0);
    let updated = g
        .deck
        .cards()
        .into_iter()
        .find(|c| c.id == card.id)
        .unwrap();
    assert_eq!(updated.seal, Some(Seal::Gold));
}

#[test]
// Test executing a full game using the gen_action_space (vector) api
fn test_game_action_space() {
    let mut g = Game::default();

    g.start();
    while !g.is_over() {
        // Get action space and vector
        let space = g.gen_action_space();
        let space_vec = space.to_vec();
        if space.is_empty() {
            break;
        }

        // Pick a random move and ensure its unmasked
        let mut i: usize;
        loop {
            i = rand::thread_rng().gen_range(0..space_vec.len());
            if space_vec[i] == 1 {
                break;
            }
        }
        let action = space.to_action(i, &g).expect("valid index to action");
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
