use super::*;

#[test]
fn test_pick_pack_card_targeting_spectral() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    assert!(Spectral::Talisman.requires_targets());
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Spectral(Spectral::Talisman)],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Spectral(Spectral::Talisman))
        .expect("pick targeting spectral");
    assert!(matches!(g.stage, Stage::SpectralHand(Spectral::Talisman)));
    assert_eq!(g.spectral_prev_stage, Some(Stage::PackOpen()));
    // picks_remaining decremented later in apply_spectral, not here
    assert_eq!(g.open_pack.as_ref().unwrap().picks_remaining, 1);
}

#[test]
fn test_pick_pack_card_spectral_immediate_apply() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    assert!(!Spectral::Sigil.requires_targets());
    g.available.extend(vec![crate::card::Card::new(
        crate::card::Value::Two,
        crate::card::Suit::Heart,
    )]);
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Spectral(Spectral::Sigil)],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Spectral(Spectral::Sigil))
        .expect("pick immediate-apply spectral");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_reroll_invalid_outside_shop() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    let res = g.reroll();
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_reroll_insufficient_balance() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let res = g.reroll();
    assert!(matches!(res, Err(GameError::InvalidBalance)));
}

#[test]
fn test_reroll_deducts_cost() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    g.reroll().expect("reroll");
    assert_eq!(g.money, 5);
}

#[test]
fn test_reroll_increments_cost() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 20;
    g.reroll().expect("first reroll");
    assert_eq!(g.reroll_cost, 6);
    g.reroll().expect("second reroll");
    assert_eq!(g.reroll_cost, 7);
    assert_eq!(g.money, 9); // 20 - 5 - 6
}

#[test]
fn test_reroll_regenerates_shop_cards() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    let old_joker = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![old_joker.clone()];
    g.shop.consumables = Vec::new();
    g.reroll().expect("reroll");
    assert_eq!(g.shop.jokers.len() + g.shop.consumables.len(), 2);
    let still_present = g
        .shop
        .jokers
        .iter()
        .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&old_joker));
    assert!(!still_present);
}

#[test]
fn test_shop_offers_a_voucher_after_cashout() {
    let g = game_in_shop(0);
    let offered = g.shop.voucher.expect("ante 1 offers a voucher");
    // Only tier-1 vouchers can show up before anything is redeemed.
    assert_eq!(offered.requires(), None);
}

#[test]
fn test_voucher_offer_survives_reroll_but_not_purchase() {
    let mut g = game_in_shop(50);
    let offered = g.shop.voucher.expect("voucher offered");

    g.reroll().expect("reroll");
    assert_eq!(g.shop.voucher, Some(offered), "reroll must not touch it");

    g.handle_action(Action::BuyVoucher(offered))
        .expect("buy voucher");
    assert_eq!(g.shop.voucher, None);
    assert!(g.vouchers.has(offered));
    assert_eq!(g.money, 50 - 5 - offered.cost(), "reroll then voucher");
}

#[test]
fn test_voucher_offer_stays_gone_for_the_rest_of_the_ante() {
    let mut g = game_in_shop(50);
    let offered = g.shop.voucher.expect("voucher offered");
    g.handle_action(Action::BuyVoucher(offered))
        .expect("buy voucher");

    // Next round, same ante: no new offer.
    g.handle_action(Action::NextRound()).expect("next round");
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    assert_eq!(g.shop.voucher, None);

    // New ante: a fresh offer, and never the one already redeemed.
    g.ante_current = Ante::Two;
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    let next = g.shop.voucher.expect("ante 2 offers a voucher");
    assert_ne!(next, offered);
}

#[test]
fn test_cannot_buy_voucher_twice_or_without_funds() {
    let mut g = game_in_shop(0);
    let offered = g.shop.voucher.expect("voucher offered");
    assert!(matches!(
        g.handle_action(Action::BuyVoucher(offered)),
        Err(GameError::InvalidBalance)
    ));

    redeem(&mut g, offered);
    g.shop.voucher = Some(offered);
    g.money = 100;
    assert!(matches!(
        g.handle_action(Action::BuyVoucher(offered)),
        Err(GameError::VoucherAlreadyRedeemed)
    ));
}

#[test]
fn test_upgrade_voucher_only_offered_once_base_is_owned() {
    let mut g = game_in_shop(100);
    // Fast mode draws from the offerable pool, so with only the
    // upgrade's base missing it can never surface the upgrade.
    for _ in 0..50 {
        g.voucher_ante = None;
        g.ante_current = Ante::One;
        g.refresh_voucher_offer();
        let offered = g.shop.voucher.expect("voucher offered");
        assert_eq!(offered.requires(), None, "upgrade offered with no base");
    }

    redeem(&mut g, Voucher::Overstock);
    assert!(g.vouchers.is_offerable(Voucher::OverstockPlus));
}

#[test]
fn test_overstock_widens_the_shop() {
    let mut g = game_in_shop(100);
    let before = g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len();
    assert_eq!(before, 2);

    redeem(&mut g, Voucher::Overstock);
    // Applied to the shop the player is standing in, not just the next.
    let after = g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len();
    assert_eq!(after, 3);

    // ...and to every later refresh.
    g.reroll().expect("reroll");
    assert_eq!(
        g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len(),
        3
    );
}

#[test]
fn test_clearance_sale_discounts_cards_and_packs_but_not_vouchers() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::ClearanceSale);

    let joker = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![joker.clone()];
    g.jokers.clear();
    let full = joker.cost();
    let money_before = g.money;
    g.buy_joker(joker).expect("buy joker");
    assert_eq!(g.money, money_before - (full * 3 / 4));

    // Vouchers keep their sticker price.
    g.shop.voucher = Some(Voucher::Blank);
    let money_before = g.money;
    g.handle_action(Action::BuyVoucher(Voucher::Blank))
        .expect("buy voucher");
    assert_eq!(g.money, money_before - Voucher::Blank.cost());
}

#[test]
fn test_reroll_vouchers_cut_the_price_immediately() {
    let mut g = game_in_shop(100);
    assert_eq!(g.reroll_cost, 5);
    redeem(&mut g, Voucher::RerollSurplus);
    assert_eq!(g.reroll_cost, 3, "current shop gets the discount too");
    assert_eq!(g.base_reroll_cost(), 3);

    redeem(&mut g, Voucher::RerollGlut);
    assert_eq!(g.reroll_cost, 1);
}

#[test]
fn omen_globe_can_replace_tarots_in_free_arcana_packs() {
    let mut game = Game::new(Config {
        seed: Some(0),
        ..Default::default()
    });
    game.vouchers.redeem(Voucher::CrystalBall);
    game.vouchers.redeem(Voucher::OmenGlobe);
    let mut spectrals = 0;
    for _ in 0..20 {
        game.tags.push(Tag::Charm);
        game.resolve_blind_choice_tags();
        let pack = game.open_pack.as_ref().unwrap();
        assert_eq!(pack.contents.len(), 5);
        spectrals += pack
            .contents
            .iter()
            .filter(|content| matches!(content, PackContent::Spectral(_)))
            .count();
        game.skip_pack().unwrap();
    }
    assert!(spectrals > 0 && spectrals < 100);
}

#[test]
fn every_voucher_redeems_once_and_upgrades_require_the_base() {
    for voucher in Voucher::iter() {
        let mut g = game_in_shop(100);
        g.shop.voucher = Some(voucher);
        if let Some(required) = voucher.requires() {
            assert!(g.buy_voucher(voucher).is_err());
            assert_eq!(g.money, 100);
            g.vouchers.redeem(required);
        }
        g.buy_voucher(voucher).unwrap();
        assert!(g.vouchers.has(voucher));
        assert_eq!(g.money, 100 - voucher.cost());
        assert!(g.buy_voucher(voucher).is_err());
        assert_eq!(g.money, 100 - voucher.cost());
    }
}

#[test]
fn test_slot_and_resource_vouchers() {
    let mut g = game_in_shop(100);
    let slots = g.consumable_slots();
    redeem(&mut g, Voucher::CrystalBall);
    assert_eq!(g.consumable_slots(), slots + 1);

    let joker_slots = g.joker_slots();
    redeem(&mut g, Voucher::Blank);
    redeem(&mut g, Voucher::Antimatter);
    assert_eq!(g.joker_slots(), joker_slots + 1);

    let hand_size = g.hand_size();
    redeem(&mut g, Voucher::PaintBrush);
    assert_eq!(g.hand_size(), hand_size + 1);

    // Per-round resources take effect at the start of the next round.
    redeem(&mut g, Voucher::Grabber);
    redeem(&mut g, Voucher::Wasteful);
    assert_eq!(g.plays_per_round(), g.config.plays + 1);
    assert_eq!(g.discards_per_round(), g.config.discards + 1);
    g.clear_blind();
    assert_eq!(g.plays, g.config.plays + 1);
    assert_eq!(g.discards(), g.config.discards + 1);
}

// Regression: `clear_blind` resets the per-round counters when a blind
// is *finished*, which happens before the shop opens. A voucher bought
// in that shop was landing on an already-set counter and appeared to do
// nothing until the round after next.
#[test]
fn test_hand_vouchers_apply_to_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_plays = g.config.plays;
    let base_discards = g.config.discards;

    redeem(&mut g, Voucher::Grabber);
    redeem(&mut g, Voucher::Wasteful);

    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.plays, base_plays + 1, "Grabber must count this round");
    assert_eq!(
        g.discards(),
        base_discards + 1,
        "Wasteful must count this round"
    );
}

// Same path, but for the vouchers that *cost* a resource.
#[test]
fn test_hieroglyph_costs_a_hand_on_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_plays = g.config.plays;
    g.ante_current = Ante::Three;

    redeem(&mut g, Voucher::Hieroglyph);
    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.plays, base_plays - 1);
    assert_eq!(g.ante_current, Ante::Two);
}

#[test]
fn test_paint_brush_deals_a_bigger_hand_on_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_hand = g.config.available;

    redeem(&mut g, Voucher::PaintBrush);
    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.available.cards().len(), base_hand + 1);
}
