use super::*;

#[test]
fn test_skip_blind_wrong_stage() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.skip_blind(Blind::Small);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_skip_blind_draws_a_real_tag() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Juggle;
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.tags[0], Tag::Juggle);
}

#[test]
fn test_ante_tags_exist_from_game_start() {
    let g = Game::default();
    assert!(g.skip_tag(Blind::Small).is_some());
    assert!(g.skip_tag(Blind::Big).is_some());
    assert!(g.skip_tag(Blind::Boss).is_none());
}

#[test]
fn test_ante_tags_redrawn_after_boss_defeated() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Boss),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ante_before = g.ante_current;
    g.handle_score(1_000_000).expect("handle score");
    assert_ne!(g.ante_current, ante_before);
    assert!(Tag::iter().any(|t| t == g.small_blind_tag));
    assert!(Tag::iter().any(|t| t == g.big_blind_tag));
}

#[test]
fn test_current_boss_set_on_game_new() {
    let g = Game::new(Config::default());
    assert!(g.current_boss.is_some());
    assert!(!g.current_boss.unwrap().is_finisher());
}

#[test]
fn test_current_boss_redrawn_after_boss_defeated() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Boss),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.handle_score(1_000_000).expect("handle score");
    let boss = g
        .current_boss
        .expect("current_boss redrawn after ante advance");
    assert_eq!(boss.is_finisher(), g.ante_current.number() as i32 % 8 == 0);
}

#[test]
fn test_skip_blind_banks_the_predrawn_tag() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Investment;
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.tags, vec![Tag::Investment]);
}

#[test]
fn test_cashout_consumes_only_shop_tags() {
    let mut g = Game {
        stage: Stage::PostBlind(),
        ..Default::default()
    };
    g.tags = vec![Tag::Juggle, Tag::Investment, Tag::D6, Tag::Coupon];
    g.cashout().unwrap();
    assert_eq!(g.tags, [Tag::Juggle, Tag::Investment]);
    assert_eq!(g.reroll_cost, 0);
    assert!(g.shop.free_packs);
    assert_eq!(g.shop.free_jokers.len(), g.shop.jokers.len());
}

#[test]
fn immediate_tags_apply_once_and_double_copies_the_orbital_target() {
    let mut g = Game::default();
    let rank = HandRank::TwoPair;
    let starting_level = g.planetarium.level(rank).level;
    g.acquire_tag(Tag::Double, None);
    g.acquire_tag(Tag::Double, None);
    assert_eq!(g.tags, [Tag::Double, Tag::Double]);
    g.acquire_tag(Tag::Orbital, Some(rank));
    assert!(g.tags.is_empty());
    assert_eq!(g.planetarium.level(rank).level, starting_level + 9);

    g.money = 12;
    g.skipped_count = 2;
    g.unused_discards = 4;
    g.planetarium.play(rank);
    g.acquire_tag(Tag::Speed, None);
    g.acquire_tag(Tag::Garbage, None);
    g.acquire_tag(Tag::Handy, None);
    assert_eq!(g.money, 27);
    g.acquire_tag(Tag::Economy, None);
    assert_eq!(g.money, 54);
    g.acquire_tag(Tag::TopUp, None);
    assert_eq!(g.jokers.len(), 2);
    assert!(g.jokers.iter().all(|j| j.rarity() == Rarity::Common));
}

#[test]
fn blind_tags_pay_on_boss_and_raise_hand_size_for_one_round() {
    let mut g = Game {
        small_blind_tag: Tag::Juggle,
        big_blind_tag: Tag::Investment,
        ..Game::default()
    };
    let base_size = g.hand_size();
    g.skip_blind(Blind::Small).unwrap();
    g.select_blind(Blind::Big).unwrap();
    assert_eq!(g.hand_size(), base_size + 3);
    assert!(g.tags.is_empty());
    g.clear_blind();
    assert_eq!(g.hand_size(), base_size);
    g.stage = Stage::Blind(Blind::Boss);
    g.blind = Some(Blind::Boss);
    g.acquire_tag(Tag::Investment, None);
    let base_reward = g.calc_reward(Blind::Boss).unwrap().total();
    g.handle_score(g.required_score()).unwrap();
    assert_eq!(g.reward, base_reward + 25);
    assert_eq!(g.round_reward.tags, vec![(Tag::Investment, 25)]);
    assert!(g.tags.is_empty());
}

#[test]
fn free_tag_packs_return_to_blinds_in_acquisition_order() {
    for (tag, category, size) in [
        (Tag::Standard, PackCategory::Standard, PackSize::Mega),
        (Tag::Charm, PackCategory::Arcana, PackSize::Mega),
        (Tag::Meteor, PackCategory::Celestial, PackSize::Mega),
        (Tag::Buffoon, PackCategory::Buffoon, PackSize::Mega),
        (Tag::Ethereal, PackCategory::Spectral, PackSize::Normal),
    ] {
        let mut g = Game::default();
        let money = g.money;
        g.small_blind_tag = tag;
        g.acquire_tag(Tag::Double, None);
        g.skip_blind(Blind::Small).unwrap();
        assert_eq!(g.stage, Stage::PackOpen());
        let pack = g.open_pack.as_ref().unwrap();
        assert!(pack.description.starts_with(&format!(
            "Choose {}",
            if size == PackSize::Mega { 2 } else { 1 }
        )));
        assert!(pack.contents.iter().all(|item| matches!(
            (category, item),
            (PackCategory::Standard, PackContent::PlayingCard(_))
                | (PackCategory::Arcana, PackContent::Tarot(_))
                | (PackCategory::Celestial, PackContent::Planet(_))
                | (PackCategory::Buffoon, PackContent::Joker(_))
                | (PackCategory::Spectral, PackContent::Spectral(_))
        )));
        #[cfg(feature = "serde")]
        {
            g = Game::from_json(&g.to_json().unwrap()).unwrap();
        }
        g.skip_pack().unwrap();
        assert_eq!(g.stage, Stage::PackOpen());
        g.skip_pack().unwrap();
        assert_eq!(g.stage, Stage::PreBlind());
        assert_eq!(g.blind, Some(Blind::Small));
        assert_eq!(g.money, money);
    }
}

#[test]
fn shop_tags_offer_free_jokers_and_an_extra_voucher() {
    let mut g = Game {
        stage: Stage::PostBlind(),
        ..Default::default()
    };
    g.tags = vec![Tag::Uncommon, Tag::Rare, Tag::Voucher];
    g.cashout().unwrap();
    assert_eq!(g.shop.voucher_offers().count(), 2);
    for rarity in [Rarity::Uncommon, Rarity::Rare] {
        assert!(g
            .shop
            .jokers
            .iter()
            .any(|j| j.rarity() == rarity && g.shop.joker_price(j, &g.vouchers) == 0));
    }

    for (tag, edition) in [
        (Tag::Foil, Edition::Foil),
        (Tag::Holographic, Edition::Holographic),
        (Tag::Polychrome, Edition::Polychrome),
        (Tag::Negative, Edition::Negative),
    ] {
        g.shop.jokers = vec![Jokers::TheJoker(Default::default())];
        g.tags = vec![tag];
        g.resolve_shop_tags(false);
        assert!(g.tags.is_empty());
        assert_eq!(g.shop.jokers[0].edition(), edition);
        assert_eq!(g.shop.joker_price(&g.shop.jokers[0], &g.vouchers), 0);
    }
}

#[test]
fn boss_tag_and_vouchers_share_the_reroll_lifecycle() {
    let mut g = Game {
        small_blind_tag: Tag::Boss,
        ..Game::default()
    };
    let before = g.current_boss;
    let money = g.money;
    g.skip_blind(Blind::Small).unwrap();
    assert_ne!(g.current_boss, before);
    assert_eq!(g.money, money);
    g.vouchers.redeem(Voucher::DirectorsCut);
    g.money = 30;
    assert!(!g.boss_reroll_available());
    g.vouchers.redeem(Voucher::Retcon);
    let before = g.current_boss;
    g.handle_action(Action::RerollBoss()).unwrap();
    assert_ne!(g.current_boss, before);
    assert_eq!(g.money, 20);
}

#[test]
fn test_finish_pack_does_not_touch_tags() {
    let mut g = Game::default();
    g.start();
    g.tags = vec![Tag::Handy];
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![],
    });
    g.stage = Stage::PackOpen();
    g.skip_pack().expect("skip pack");
    assert_eq!(g.tags, vec![Tag::Handy]);
}

#[test]
fn test_pick_pack_card_planet() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Planet(Planets::Mercury)],
    });
    g.stage = Stage::PackOpen();
    let level_before = g.planetarium.level(HandRank::OnePair).level;
    g.pick_pack_card(PackContent::Planet(Planets::Mercury))
        .expect("pick planet");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
    assert_eq!(
        g.planetarium.level(HandRank::OnePair).level,
        level_before + 1
    );
}

#[test]
fn test_pick_pack_card_joker() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let joker = Jokers::TheJoker(TheJoker::default());
    let before_len = g.jokers.len();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(joker.clone())],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Joker(joker))
        .expect("pick joker");
    assert_eq!(g.jokers.len(), before_len + 1);
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_pick_pack_card_joker_slots_full() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let slots = g.config.joker_slots;
    for _ in 0..slots {
        g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    }
    let joker = Jokers::TheJoker(TheJoker::default());
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(joker.clone())],
    });
    g.stage = Stage::PackOpen();
    let res = g.pick_pack_card(PackContent::Joker(joker));
    assert!(matches!(res, Err(GameError::NoAvailableSlot)));
}

#[test]
fn test_negative_joker_from_pack_expands_slots() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let slots = g.config.joker_slots;
    for _ in 0..slots {
        g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    }
    let mut neg_joker = Jokers::TheJoker(TheJoker::default());
    neg_joker.set_edition(Edition::Negative);
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(neg_joker.clone())],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Joker(neg_joker))
        .expect("pick negative joker from full pack");
    assert_eq!(g.config.joker_slots, slots + 1);
    assert_eq!(g.jokers.len(), slots + 1);
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_pick_pack_card_targeting_tarot() {
    use crate::card::{Card, Enhancement, Suit, Value};
    let mut g = Game::default();
    g.start();
    assert!(Tarot::Lovers.requires_targets());
    g.available.extend(vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Three, Suit::Heart),
    ]);
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Tarot(Tarot::Lovers)],
    });
    g.stage = Stage::PackOpen();
    // No target selected: refused, and the pack keeps its card.
    let lovers = PackContent::Tarot(Tarot::Lovers);
    assert!(matches!(
        g.pick_pack_card(lovers.clone()),
        Err(GameError::InvalidAction)
    ));
    assert_eq!(g.open_pack.as_ref().unwrap().contents.len(), 1);
    // One hand card selected first, then the Tarot is used on it at once.
    let target = g.available.cards()[0];
    g.handle_action(Action::SelectCard(target)).unwrap();
    g.pick_pack_card(lovers).expect("use targeting tarot");
    assert_eq!(g.stage, Stage::Shop());
    let wild = g
        .deck
        .cards()
        .into_iter()
        .find(|c| c.id == target.id)
        .unwrap();
    assert_eq!(wild.enhancement, Some(Enhancement::Wild));
}
