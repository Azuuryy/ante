use super::*;
use strum::IntoEnumIterator;

#[test]
fn test_planets_id_round_trip() {
    for p in Planets::iter() {
        assert_eq!(Planets::from_id(p.id()), Some(p));
    }
}

#[test]
fn test_planet_count() {
    assert_eq!(Planets::iter().count(), 12);
}

#[test]
fn test_planet_hand_ranks() {
    assert_eq!(Planets::Pluto.hand_rank(), HandRank::HighCard);
    assert_eq!(Planets::Neptune.hand_rank(), HandRank::StraightFlush);
    assert_eq!(Planets::Eris.hand_rank(), HandRank::FlushFive);
}

#[test]
fn test_planet_is_secret() {
    assert!(!Planets::Pluto.is_secret());
    assert!(!Planets::Neptune.is_secret());
    assert!(Planets::PlanetX.is_secret());
    assert!(Planets::Ceres.is_secret());
    assert!(Planets::Eris.is_secret());
}

#[test]
fn test_planet_cost_and_sell_value() {
    assert_eq!(Planets::Mercury.cost(), 3);
    assert_eq!(Planets::Mercury.sell_value(), 1);
}

#[test]
fn test_planetarium_level_up_highcard() {
    let mut p = Planetarium::new();
    let before = p.level(HandRank::HighCard);
    p.level_up(HandRank::HighCard);
    let after = p.level(HandRank::HighCard);
    assert_eq!(after.level, before.level + 1);
    assert_eq!(after.chips, before.chips + 10);
    assert_eq!(after.mult, before.mult + 1);
}

#[test]
fn test_planetarium_level_down_reverses_level_up() {
    let mut p = Planetarium::new();
    p.level_up(HandRank::OnePair);
    p.level_up(HandRank::OnePair);
    let before = p.level(HandRank::OnePair);
    p.level_down(HandRank::OnePair);
    let after = p.level(HandRank::OnePair);
    assert_eq!(after.level, before.level - 1);
    assert_eq!(after.chips, before.chips - 15);
    assert_eq!(after.mult, before.mult - 1);
}

#[test]
fn test_planetarium_level_down_stops_at_level_one() {
    let mut p = Planetarium::new();
    let before = p.level(HandRank::OnePair);
    assert_eq!(before.level, 1);
    p.level_down(HandRank::OnePair);
    assert_eq!(p.level(HandRank::OnePair), before);
}

#[test]
fn test_planetarium_level_down_saturates_on_inconsistent_state() {
    let mut p = Planetarium::new();
    let level = p.level_mut(HandRank::OnePair);
    level.level = 2;
    level.chips = 0;
    level.mult = 0;
    p.level_down(HandRank::OnePair);
    let after = p.level(HandRank::OnePair);
    assert_eq!(after.level, 1);
    assert_eq!(after.chips, 0);
    assert_eq!(after.mult, 0);
}

#[test]
fn test_planetarium_royal_flush_shares_straight_flush_slot() {
    let mut p = Planetarium::new();
    p.level_up(HandRank::StraightFlush);
    assert_eq!(
        p.level(HandRank::RoyalFlush),
        p.level(HandRank::StraightFlush)
    );
}

#[test]
fn test_planetarium_play_increments_count() {
    let mut p = Planetarium::new();
    assert_eq!(p.level(HandRank::OnePair).plays, 0);
    p.play(HandRank::OnePair);
    p.play(HandRank::OnePair);
    assert_eq!(p.level(HandRank::OnePair).plays, 2);
    assert_eq!(p.level(HandRank::HighCard).plays, 0);
}

#[test]
fn test_planets_from_str_round_trip() {
    for planet in Planets::iter() {
        assert_eq!(format!("{planet:?}").parse::<Planets>(), Ok(planet));
    }
}

#[test]
fn test_planets_from_str_case_insensitive() {
    assert_eq!("planetx".parse::<Planets>(), Ok(Planets::PlanetX));
    assert_eq!("PLANETX".parse::<Planets>(), Ok(Planets::PlanetX));
}

#[test]
fn test_planets_from_str_invalid() {
    assert!("NotAPlanet".parse::<Planets>().is_err());
}
