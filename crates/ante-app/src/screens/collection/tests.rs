use super::layout::{blind_amount, number_format, page_layout};
use ante_core::contract::ContentGroup;

#[test]
fn pages_follow_the_original_collection_areas() {
    let jokers = page_layout(ContentGroup::Jokers).unwrap();
    assert_eq!(jokers.per_page(), 15);
    assert_eq!(jokers.pages(150), 10);
    let tarots = page_layout(ContentGroup::Tarots).unwrap();
    assert_eq!(
        tarots.rows.iter().map(|r| r.count).collect::<Vec<_>>(),
        [5, 6]
    );
    assert_eq!(tarots.pages(22), 2);
    let spectrals = page_layout(ContentGroup::Spectrals).unwrap();
    assert_eq!(spectrals.per_page(), 9);
    assert_eq!(spectrals.pages(18), 2);
    let planets = page_layout(ContentGroup::Planets).unwrap();
    assert_eq!(planets.pages(12), 1);
    assert!(!planets.cycle);
    assert_eq!(page_layout(ContentGroup::Vouchers).unwrap().pages(32), 4);
    assert_eq!(
        page_layout(ContentGroup::Boosters).unwrap().card_scale,
        1.27
    );
    assert!(page_layout(ContentGroup::Tags).is_none());
}

#[test]
fn blind_amounts_and_their_format_match_the_original() {
    assert_eq!(blind_amount(1), 300.0);
    assert_eq!(blind_amount(8), 50000.0);
    assert_eq!(blind_amount(9), 110000.0);
    assert_eq!(number_format(110000.0), "110,000");
    assert_eq!(number_format(300.0), "300");
    assert!(number_format(blind_amount(16)).contains('e'));
}

#[test]
fn collection_popups_match_the_run_popups() {
    use super::art::collection_tip;
    use crate::components::tooltip::Tip;
    use ante_core::contract::{Content, ContentId};
    let item = |id, name: &str, rarity| Content {
        id: ContentId(id),
        name: name.to_owned(),
        description: "Rule".to_owned(),
        cost: 0,
        supported: true,
        unlock: Some("Win a run"),
        rarity,
    };
    let joker = item("j_joker", "Joker", Some("Common"));
    assert_eq!(
        collection_tip(&joker, ContentGroup::Jokers, true, false).without_technical(),
        Tip::new("Joker", "Rule").badge("Common")
    );
    assert_eq!(
        collection_tip(
            &item("m_bonus", "Bonus Card", None),
            ContentGroup::Enhancements,
            true,
            false
        )
        .without_technical(),
        Tip::new("", "Rule").badge("Bonus Card")
    );
    assert_eq!(
        collection_tip(
            &item("e_holo", "Holographic", None),
            ContentGroup::Editions,
            true,
            false
        )
        .without_technical(),
        Tip::new("Holographic", "Rule")
            .badge("Edition")
            .badge("Holographic")
    );
    assert_eq!(
        collection_tip(
            &item("c_fool", "The Fool", None),
            ContentGroup::Tarots,
            true,
            false
        )
        .without_technical(),
        Tip::new("The Fool", "Rule").badge("Tarot")
    );
    assert_eq!(
        collection_tip(&joker, ContentGroup::Jokers, true, true).without_technical(),
        Tip::new("Locked", "Win a run")
    );
}
