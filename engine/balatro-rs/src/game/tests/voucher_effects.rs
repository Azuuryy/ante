use super::*;

#[test]
fn test_hieroglyph_lowers_the_ante_and_costs_a_hand() {
    let mut g = game_in_shop(100);
    g.ante_current = Ante::Three;
    let hands = g.plays_per_round();

    redeem(&mut g, Voucher::Hieroglyph);
    assert_eq!(g.ante_current, Ante::Two);
    assert_eq!(g.plays_per_round(), hands - 1);
    assert_eq!(g.discards_per_round(), g.config.discards);
}

#[test]
fn test_seed_money_raises_the_interest_cap() {
    let mut g = game_in_shop(100);
    assert_eq!(g.interest_max(), g.config.interest_max);
    redeem(&mut g, Voucher::SeedMoney);
    assert_eq!(g.interest_max(), g.config.interest_max + 5);
    redeem(&mut g, Voucher::MoneyTree);
    assert_eq!(g.interest_max(), g.config.interest_max + 15);
}

#[test]
fn test_observatory_multiplies_mult_for_held_planets() {
    use crate::planet::Planets;

    // Two Kings = OnePair: 10 chips + 10 per King, 2 mult -> 30 * 2 = 60.
    let cards = vec![
        Card::new(Value::King, Suit::Heart),
        Card::new(Value::King, Suit::Spade),
    ];
    let hand = SelectHand::new(cards).best_hand().unwrap();

    let mut g = Game::default();
    assert_eq!(g.calc_score(hand.clone()), 60);

    // Mercury is OnePair's planet, so Observatory applies: mult 2 -> 3.
    g.consumables.push(Consumable::Planet(Planets::Mercury));
    g.vouchers.redeem(Voucher::Observatory);
    assert_eq!(g.calc_score(hand.clone()), 90);

    // A planet for a different hand does nothing.
    g.consumables = vec![Consumable::Planet(Planets::Jupiter)];
    assert_eq!(g.calc_score(hand), 60);
}

#[test]
fn test_telescope_puts_the_most_played_planet_in_celestial_packs() {
    use crate::pack::{PackCategory, PackContent};
    use crate::planet::Planets;

    let mut g = game_in_shop(100);
    // Make Flush the most played hand, then take Telescope.
    for _ in 0..5 {
        g.planetarium.play(HandRank::Flush);
    }
    redeem(&mut g, Voucher::Telescope);
    assert_eq!(g.most_played_hand_rank(), HandRank::Flush);

    let mut saw_celestial = false;
    for _ in 0..200 {
        g.stage = Stage::PostBlind();
        g.handle_action(Action::CashOut(0)).expect("cash out");
        for pack in &g.shop.packs {
            if pack.category != PackCategory::Celestial {
                continue;
            }
            saw_celestial = true;
            assert!(
                pack.contents
                    .contains(&PackContent::Planet(Planets::Jupiter)),
                "Celestial pack without the most played hand's planet: {:?}",
                pack.contents
            );
        }
    }
    assert!(saw_celestial, "no Celestial pack generated in 200 shops");
}

#[test]
fn test_magic_trick_puts_buyable_playing_cards_in_the_shop() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::MagicTrick);

    // Playing cards are one of four outcomes per slot; reroll until seen.
    let mut card = None;
    for _ in 0..500 {
        if let Some(c) = g.shop.cards.first().copied() {
            card = Some(c);
            break;
        }
        g.money = 100;
        g.reroll().expect("reroll");
    }
    let card = card.expect("Magic Trick never produced a shop playing card");
    // Without Illusion, shop cards are plain.
    assert!(card.enhancement.is_none() && card.seal.is_none());
    assert_eq!(card.edition, Edition::Base);

    let deck_before = g.deck.len();
    g.money = 100;
    g.handle_action(Action::BuyPlayingCard(card))
        .expect("buy playing card");
    assert_eq!(g.deck.len(), deck_before + 1);
    assert!(g.deck.cards().iter().any(|c| c.id == card.id));
    assert!(!g.shop.cards.iter().any(|c| c.id == card.id));
    assert_eq!(g.money, 100 - card.shop_cost());
}

// regression: `buy_playing_card` didn't check `deck_max` itself
#[test]
fn test_buy_playing_card_rejects_a_full_deck() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::MagicTrick);

    let mut card = None;
    for _ in 0..500 {
        if let Some(c) = g.shop.cards.first().copied() {
            card = Some(c);
            break;
        }
        g.money = 100;
        g.reroll().expect("reroll");
    }
    let card = card.expect("Magic Trick never produced a shop playing card");

    // Cap the deck at its current size so there's no room left.
    g.config.deck_max = g.deck.len() + g.available.cards().len() + g.discarded.len();
    g.money = 100;
    assert!(matches!(
        g.handle_action(Action::BuyPlayingCard(card)),
        Err(GameError::NoAvailableSlot)
    ));
}

#[test]
fn test_buy_voucher_round_trips_through_the_action_space() {
    let mut g = game_in_shop(100);
    let offered = g.shop.voucher.expect("voucher offered");
    let space = g.gen_action_space();
    let index = space
        .to_vec()
        .iter()
        .enumerate()
        .find_map(|(i, _)| match space.to_action(i, &g) {
            Ok(Action::BuyVoucher(v)) if v == offered => Some(i),
            _ => None,
        })
        .expect("BuyVoucher is unmasked in the shop");
    g.handle_action_index(index).expect("handle action index");
    assert!(g.vouchers.has(offered));
}

#[test]
fn test_vouchers_survive_a_save_load_round_trip() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::CrystalBall);
    redeem(&mut g, Voucher::Grabber);
    let slots = g.consumable_slots();
    let plays = g.plays_per_round();

    let json = g.to_json().expect("serialize");
    let loaded = Game::from_json(&json).expect("deserialize");
    assert_eq!(loaded.vouchers.owned(), g.vouchers.owned());
    // Derived, not baked into Config — so no double application.
    assert_eq!(loaded.consumable_slots(), slots);
    assert_eq!(loaded.plays_per_round(), plays);
}
