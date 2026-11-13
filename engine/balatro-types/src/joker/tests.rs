use super::*;
use strum::IntoEnumIterator;

#[test]
fn test_jokers_from_str_round_trip() {
    for joker in Jokers::iter() {
        let debug = format!("{joker:?}");
        let variant_name = debug.split('(').next().unwrap();
        assert_eq!(variant_name.parse::<Jokers>().unwrap(), joker);
    }
}

#[test]
fn test_jokers_id_round_trip() {
    for joker in Jokers::iter() {
        let id = joker.id();
        assert_eq!(Jokers::from_id(id).map(|j| j.id()), Some(id));
    }
}

#[test]
fn test_jokers_known_ids() {
    assert_eq!(Jokers::TheJoker(TheJoker::default()).id(), "j_joker");
    assert_eq!(
        Jokers::GluttonousJoker(GluttonousJoker::default()).id(),
        "j_gluttenous_joker"
    );
    assert_eq!(
        Jokers::from_id("j_scary_face"),
        Some(Jokers::ScaryFace(ScaryFace::default()))
    );
    assert_eq!(Jokers::from_id("j_not_a_real_joker"), None);
}

#[test]
fn test_jokers_from_str_case_insensitive() {
    let expected = Jokers::HalfJoker(HalfJoker::default());
    assert_eq!("halfjoker".parse::<Jokers>(), Ok(expected.clone()));
    assert_eq!("HALFJOKER".parse::<Jokers>(), Ok(expected));
}

#[test]
fn test_jokers_from_str_invalid() {
    assert!("NotARealJoker".parse::<Jokers>().is_err());
}
