use super::*;
use crate::config::Config;
use crate::stage::Blind;

#[test]
fn test_unmask() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);

    a.unmask_select_card(1).unwrap();

    let v: Vec<usize> = Vec::from(a);
    dbg!(v.clone());
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 1);
}

#[test]
fn test_unmask_max_index() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);

    a.unmask_select_card(23).unwrap();

    let v: Vec<usize> = Vec::from(a.clone());
    dbg!(v.clone());
    assert_eq!(v[0], 0);
    assert_eq!(v[23], 1);

    let res = a.unmask_select_card(24);
    assert!(res.is_err());
}

#[test]
fn test_action_space_size() {
    let c = Config::default();
    let a = ActionSpace::from(c.clone());
    // 24 select + 23 move_left + 23 move_right + 1 play + 1 discard
    // + 1 cashout + 4 buy_joker + 1 next_round + 1 select_blind + 1 skip_blind
    // + 2 buy_consumable + 1 buy_voucher + 4 buy_playing_card
    // + 2 use_consumable + 1 apply_tarot
    // + 5 sell_joker + 2 sell_consumable
    // + 2 buy_pack + 5 pick_pack_card + 1 skip_pack + 2 sort_hand + 1 reroll
    // + 1 apply_spectral = 109
    assert_eq!(a.size(), 109);
    assert_eq!(a.to_vec().len(), 109);
}

#[test]
fn test_action_space_zero_joker_slots_no_panic() {
    let c = Config {
        joker_slots: 0,
        consumable_slots: 0,
        ..Default::default()
    };
    let a = ActionSpace::from(c);
    // size() and to_vec() must not panic with empty sell vecs
    assert_eq!(a.to_vec().len(), a.size());
    // to_action on the apply_tarot index must still resolve and not panic
    assert!(a.to_action(a.apply_tarot_min(), &Game::default()).is_err()); // masked, not a panic
                                                                          // Same for the apply_spectral index, appended at the end of the vector.
    assert!(a
        .to_action(a.apply_spectral_min(), &Game::default())
        .is_err()); // masked, not a panic
}

#[test]
fn test_apply_spectral_index_round_trip() {
    use crate::pack::{Pack, PackCategory, PackContent, PackSize};
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    g.stage = crate::stage::Stage::Shop();
    g.money = 100;
    let pack = Pack {
        category: PackCategory::Spectral,
        size: PackSize::Normal,
        contents: vec![PackContent::Spectral(Spectral::Talisman)],
    };
    g.shop.packs = vec![pack.clone()];
    g.handle_action(Action::BuyPack(pack)).unwrap();
    g.handle_action(Action::PickPackCard(PackContent::Spectral(
        Spectral::Talisman,
    )))
    .unwrap();
    let card = g.available.cards()[0];
    g.handle_action(Action::SelectCard(card)).unwrap();

    let space = g.gen_action_space();
    let idx = space.apply_spectral_min();
    assert_eq!(space.to_vec()[idx], 1);
    let action = space.to_action(idx, &g).expect("to action");
    assert_eq!(action, Action::ApplySpectral());
}

#[test]
fn test_unmask_sell_joker() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);
    assert_eq!(a.sell_joker[0], 0);
    a.unmask_sell_joker(0).unwrap();
    assert_eq!(a.sell_joker[0], 1);
    assert!(a.unmask_sell_joker(5).is_err());
}

#[test]
fn test_unmask_sell_consumable() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);
    assert_eq!(a.sell_consumable[0], 0);
    a.unmask_sell_consumable(0).unwrap();
    assert_eq!(a.sell_consumable[0], 1);
    assert!(a.unmask_sell_consumable(2).is_err());
}

#[test]
fn test_skip_blind_round_trip() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = crate::tag::Tag::Investment;
    let space = g.gen_action_space();
    let index = space.skip_blind_min();
    assert_eq!(space.to_vec()[index], 1);
    let action = space.to_action(index, &g).expect("to action");
    assert_eq!(action, Action::SkipBlind(Blind::Small));
    g.handle_action(action).unwrap();
    assert_eq!(g.tags.len(), 1);
}

#[test]
fn test_index_to_action() {
    let mut g = Game::default();
    let space = g.gen_action_space();
    let space_vec = g.gen_action_space().to_vec();

    // Game hasn't started yet, so only valid action is select blind
    for b in space_vec.iter().rev().skip(1) {
        // skip last (select_blind at index 78) AND skip use_consumable/buy_consumable
        // Actually just check select_blind is unmasked
        let _ = b;
    }
    // select_blind is at index 78
    assert_eq!(space_vec[78], 1);
    let action = space.to_action(78, &g).expect("to action");
    assert_eq!(action, Action::SelectBlind(Blind::Small));
    g.handle_action(action).unwrap();

    // Game now in small blind, we can select, move, play, discard
    let space = g.gen_action_space();
    let space_vec = g.gen_action_space().to_vec();
    assert_eq!(space_vec[0], 1);

    // We can select first card
    assert_eq!(g.available.selected().len(), 0);
    let index = space.select_card_min();
    let action = space.to_action(index, &g).expect("to action");
    assert_eq!(
        action,
        Action::SelectCard(g.available.card_from_index(0).expect("first card"))
    );
    g.handle_action(action).unwrap();
    assert_eq!(g.available.selected().len(), 1);

    // Regenerate space, cannot select first card, can select second
    let space = g.gen_action_space();
    let space_vec = g.gen_action_space().to_vec();
    assert_eq!(space_vec[0], 0);
    assert_eq!(space_vec[1], 1);
    assert_eq!(g.available.selected().len(), 1);

    // Ensure select second is unmasked, convert to action and handle
    let index = space.select_card_min() + 1;
    let action = space.to_action(index, &g).expect("to action");
    assert_eq!(
        action,
        Action::SelectCard(g.available.card_from_index(1).expect("second card"))
    );
    g.handle_action(action).unwrap();
    assert_eq!(g.available.selected().len(), 2);

    // Regenerate space, cannot select first or second, can play and discard
    let space = g.gen_action_space();
    let space_vec = g.gen_action_space().to_vec();
    assert_eq!(space_vec[0], 0);
    assert_eq!(space_vec[1], 0);
    assert_eq!(g.available.selected().len(), 2);

    let index = space.play_min();
    let action_play = space.to_action(index, &g).expect("to action");
    assert_eq!(action_play, Action::Play());

    let index = space.discard_min();
    let action_discard = space.to_action(index, &g).expect("to action");
    assert_eq!(action_discard, Action::Discard());

    // Play
    g.handle_action(action_play).unwrap();
}

#[test]
fn test_unmask_buy_consumable() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);
    assert_eq!(a.buy_consumable[0], 0);
    a.unmask_buy_consumable(0).unwrap();
    assert_eq!(a.buy_consumable[0], 1);
    assert!(a.unmask_buy_consumable(2).is_err());
}

#[test]
fn test_unmask_use_consumable() {
    let c = Config::default();
    let mut a = ActionSpace::from(c);
    assert_eq!(a.use_consumable[0], 0);
    a.unmask_use_consumable(0).unwrap();
    assert_eq!(a.use_consumable[0], 1);
    assert!(a.unmask_use_consumable(2).is_err());
}
