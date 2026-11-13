use super::*;
use balatro_rs::{
    consumable::Consumable, joker::Jokers, planet::Planets, spectral::Spectral, stage::Stage,
    tarot::Tarot,
};

#[test]
fn emperor_used_with_full_slots_creates_one_tarot() {
    let mut run = shop();
    let slots = run.game.consumable_slots();
    run.game.consumables = vec![Consumable::Tarot(Tarot::Emperor)];
    while run.game.consumables.len() < slots {
        run.game
            .consumables
            .push(Consumable::Planet(Planets::Mercury));
    }
    let revision = run.view().revision;
    run.apply(Command::UseConsumable { index: 0, revision })
        .unwrap();
    assert_eq!(run.game.consumables.len(), slots);
    assert!(
        run.game
            .consumables
            .iter()
            .any(|card| matches!(card, Consumable::Tarot(_)))
    );
}

#[test]
fn buy_and_use_applies_a_planet_without_a_slot() {
    let mut run = shop();
    let slots = run.game.consumable_slots();
    run.game.consumables = vec![Consumable::Tarot(Tarot::Hermit); slots];
    run.game.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    let view = run.view();
    assert!(view.consumable_offers[0].can_buy_and_use);
    assert!(!view.consumable_offers[0].can_buy);
    run.apply(Command::BuyAndUseConsumable {
        index: 0,
        revision: view.revision,
    })
    .unwrap();
    assert!(run.game.shop.consumables.is_empty());
    assert_eq!(run.game.consumables.len(), slots);
}

fn shop() -> EngineRun {
    let mut run = EngineRun::new(7);
    run.game.stage = Stage::Shop();
    run.game.money = 30;
    run.game.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    run.game.shop.jokers = vec![Jokers::TheJoker(Default::default())];
    run
}

#[test]
fn every_supported_joker_has_an_effect_description() {
    use balatro_rs::joker::{JokerEffects, joker_catalog};
    for joker in joker_catalog().filter(|joker| joker.is_implemented()) {
        assert_ne!(
            joker_description(joker.id()),
            "Joker effect: see supported rules in the engine.",
            "{}",
            joker.id()
        );
    }
}

#[test]
fn consumable_use_availability_follows_targets_and_stage() {
    let mut run = EngineRun::new(7);
    run.game.stage = Stage::Shop();
    run.game.consumables = vec![
        Consumable::Planet(Planets::Mercury),
        Consumable::Tarot(Tarot::Hermit),
        Consumable::Tarot(Tarot::Magician),
        Consumable::Spectral(Spectral::BlackHole),
        Consumable::Spectral(Spectral::Familiar),
    ];
    let view = run.view();
    let can_use: Vec<_> = view
        .consumables
        .iter()
        .map(|card| (card.kind, card.can_use))
        .collect();
    assert_eq!(
        can_use,
        vec![
            ("c_mercury", true),
            ("c_hermit", true),
            ("c_magician", false),
            ("c_black_hole", true),
            ("c_familiar", false),
        ],
        "non-target consumables stay usable outside a blind; hand-touching ones wait for a target"
    );
}

#[test]
fn purchases_use_sell_and_reroll_follow_engine() {
    let mut run = shop();
    let initial = run.view();
    assert_eq!((initial.joker_slots, initial.consumable_slots), (5, 2));
    assert!(initial.consumable_offers[0].can_buy);
    run.apply(Command::BuyConsumable {
        index: 0,
        revision: initial.revision,
    })
    .unwrap();
    let bought = run.view();
    assert_eq!(
        bought.money,
        initial.money - initial.consumable_offers[0].price
    );
    assert_eq!(bought.consumables[0].kind, "c_mercury");
    assert!(
        run.apply(Command::UseConsumable {
            index: 0,
            revision: initial.revision
        })
        .is_err()
    );
    run.apply(Command::UseConsumable {
        index: 0,
        revision: bought.revision,
    })
    .unwrap();
    assert!(run.view().consumables.is_empty());
    let joker = &run.view().offers[0];
    run.apply(Command::BuyJoker(joker.id)).unwrap();
    let with_joker = run.view();
    assert_eq!(with_joker.held_jokers.len(), 1);
    run.apply(Command::SellJoker {
        index: 0,
        revision: with_joker.revision,
    })
    .unwrap();
    assert!(run.view().held_jokers.is_empty());
    let before = run.view();
    assert!(before.can_reroll);
    run.apply(Command::Reroll).unwrap();
    let after = run.view();
    assert_eq!(after.money, before.money - before.reroll_cost);
    assert_eq!(after.reroll_cost, before.reroll_cost + 1);
}

#[test]
fn view_uses_current_inventory_capacities() {
    let mut run = shop();
    run.game.config.joker_slots = 3;
    run.game.config.consumable_slots = 4;
    let view = run.view();
    assert_eq!((view.joker_slots, view.consumable_slots), (3, 4));
}

#[test]
fn tarot_requires_selected_card_and_view_reports_enhancement() {
    use balatro_rs::{card::Enhancement, tarot::Tarot};
    let mut run = EngineRun::new(0);
    run.apply(Command::StartBlind).unwrap();
    run.game
        .consumables
        .push(Consumable::Tarot(Tarot::Magician));
    let before = run.view();
    assert!(!before.consumables[0].can_use);
    run.apply(Command::ToggleCard(before.cards[0].id)).unwrap();
    let selected = run.view();
    assert!(selected.consumables[0].can_use);
    run.apply(Command::UseConsumable {
        index: 0,
        revision: selected.revision,
    })
    .unwrap();
    assert!(
        run.game
            .available
            .cards()
            .iter()
            .any(|card| card.enhancement == Some(Enhancement::Lucky))
    );
    assert!(
        run.view()
            .cards
            .iter()
            .any(|card| card.enhancement == "Lucky")
    );
}

#[test]
fn full_slots_insufficient_money_and_stale_indexes_are_refused() {
    let mut run = shop();
    run.game.money = 0;
    assert!(!run.view().consumable_offers[0].can_buy);
    assert!(
        run.apply(Command::BuyConsumable {
            index: 0,
            revision: run.view().revision
        })
        .is_err()
    );
    assert!(run.apply(Command::Reroll).is_err());
    run.game.money = 30;
    run.game.consumables = vec![
        Consumable::Planet(Planets::Earth),
        Consumable::Planet(Planets::Mars),
    ];
    let full = run.view();
    assert!(!full.consumable_offers[0].can_buy);
    assert!(
        run.apply(Command::BuyConsumable {
            index: 0,
            revision: full.revision
        })
        .is_err()
    );
    run.apply(Command::SellConsumable {
        index: 0,
        revision: full.revision,
    })
    .unwrap();
    let after = run.view();
    assert_eq!(after.consumables[0].kind, "c_mars");
    assert!(
        run.apply(Command::SellConsumable {
            index: 0,
            revision: full.revision
        })
        .is_err()
    );
    assert_eq!(run.view(), after);
}
