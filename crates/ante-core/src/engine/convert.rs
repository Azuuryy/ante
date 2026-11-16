use super::*;

pub(super) fn map_rank(rank: HandRank) -> HandKind {
    match rank {
        HandRank::HighCard => HandKind::HighCard,
        HandRank::OnePair => HandKind::OnePair,
        HandRank::TwoPair => HandKind::TwoPair,
        HandRank::ThreeOfAKind => HandKind::ThreeOfAKind,
        HandRank::Straight => HandKind::Straight,
        HandRank::Flush => HandKind::Flush,
        HandRank::FullHouse => HandKind::FullHouse,
        HandRank::FourOfAKind => HandKind::FourOfAKind,
        HandRank::StraightFlush => HandKind::StraightFlush,
        HandRank::RoyalFlush => HandKind::RoyalFlush,
        HandRank::FiveOfAKind => HandKind::FiveOfAKind,
        HandRank::FlushHouse => HandKind::FlushHouse,
        HandRank::FlushFive => HandKind::FlushFive,
    }
}

pub(super) fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

pub(super) fn suit_name(suit: Suit) -> &'static str {
    SUITS[suit_index(suit)]
}

pub(super) fn edition_name(value: Edition) -> &'static str {
    match value {
        Edition::Base => "Base",
        Edition::Foil => "Foil",
        Edition::Holographic => "Holographic",
        Edition::Polychrome => "Polychrome",
        Edition::Negative => "Negative",
    }
}

pub(super) fn enhancement_name(value: Enhancement) -> &'static str {
    match value {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Gold => "Gold",
        Enhancement::Lucky => "Lucky",
    }
}

pub(super) fn seal_name(value: Seal) -> &'static str {
    match value {
        Seal::Gold => "Gold",
        Seal::Red => "Red",
        Seal::Blue => "Blue",
        Seal::Purple => "Purple",
    }
}

pub(super) fn card_art(card: balatro_rs::card::Card) -> ShopArt {
    ShopArt::Card {
        rank: char::from(card.value),
        suit: suit_name(card.suit),
        look: crate::contract::CardLook {
            enhancement: card.enhancement.map(enhancement_name),
            edition: edition_name(card.edition),
            seal: card.seal.map(seal_name),
        },
    }
}

pub(super) fn pack_art(pack: &balatro_rs::pack::Pack) -> ShopArt {
    use balatro_rs::pack::{PackCategory as Category, PackSize as Size};
    ShopArt::Pack {
        family: match pack.category {
            Category::Arcana => PackFamily::Arcana,
            Category::Celestial => PackFamily::Celestial,
            Category::Spectral => PackFamily::Spectral,
            Category::Standard => PackFamily::Standard,
            Category::Buffoon => PackFamily::Buffoon,
        },
        size: match pack.size {
            Size::Normal => PackSize::Normal,
            Size::Jumbo => PackSize::Jumbo,
            Size::Mega => PackSize::Mega,
        },
    }
}

/// The engine keeps no category for an open pack; its contents tell it. Tarots mark an
/// Arcana pack even with an Omen Globe spectral among them.
pub(super) fn open_pack_family(contents: &[PackContent]) -> PackFamily {
    let has = |test: fn(&PackContent) -> bool| contents.iter().any(test);
    if has(|c| matches!(c, PackContent::Tarot(_))) {
        PackFamily::Arcana
    } else if has(|c| matches!(c, PackContent::Planet(_))) {
        PackFamily::Celestial
    } else if has(|c| matches!(c, PackContent::PlayingCard(_))) {
        PackFamily::Standard
    } else if has(|c| matches!(c, PackContent::Joker(_))) {
        PackFamily::Buffoon
    } else {
        PackFamily::Spectral
    }
}

pub(super) fn pack_content_art(content: &PackContent) -> ShopArt {
    match content {
        PackContent::PlayingCard(card) => card_art(*card),
        PackContent::Joker(joker) => ShopArt::Joker(joker.id()),
        PackContent::Tarot(card) => ShopArt::Consumable(card.id()),
        PackContent::Planet(card) => ShopArt::Consumable(card.id()),
        PackContent::Spectral(card) => ShopArt::Consumable(card.id()),
    }
}

pub(super) fn pack_content_description(content: &PackContent) -> String {
    use balatro_rs::consumable::Consumable;
    match content {
        PackContent::PlayingCard(card) => card_effect_summary(*card),
        PackContent::Joker(joker) => joker_description(joker.id()),
        PackContent::Tarot(card) => consumable_description(Consumable::Tarot(*card)),
        PackContent::Planet(card) => consumable_description(Consumable::Planet(*card)),
        PackContent::Spectral(card) => consumable_description(Consumable::Spectral(*card)),
    }
}

pub(super) fn trace_card(card: balatro_rs::card::Card, trigger: CardTrigger) -> ScoreSource {
    ScoreSource::Card {
        trigger,
        id: CardKey(card.id),
        rank: char::from(card.value),
        suit: suit_name(card.suit),
    }
}

pub(super) fn map_trace(trace: NativeTrace) -> ScoreTrace {
    ScoreTrace(
        trace
            .0
            .into_iter()
            .map(|step| ScoreStep {
                source: match step.source {
                    NativeSource::PlayedCard(card) => trace_card(card, CardTrigger::Played),
                    NativeSource::StoneKicker(card) => trace_card(card, CardTrigger::StoneKicker),
                    NativeSource::HeldCard(card) => trace_card(card, CardTrigger::Held),
                    NativeSource::HandLevel(rank) => ScoreSource::HandLevel(map_rank(rank)),
                    NativeSource::Joker(joker) => ScoreSource::Joker {
                        id: JokerKey(joker.instance_id()),
                        kind: ContentId(joker.id()),
                        name: joker.name(),
                    },
                    NativeSource::JokerOnCard(joker, card) => ScoreSource::JokerOnCard {
                        joker: JokerKey(joker.instance_id()),
                        kind: ContentId(joker.id()),
                        name: joker.name(),
                        card: CardKey(card.id),
                    },
                    NativeSource::Voucher(voucher) => {
                        ScoreSource::Voucher(ContentId(voucher.id()), voucher_name(voucher))
                    }
                    NativeSource::NotAllowed => ScoreSource::NotAllowed,
                },
                chips_before: step.chips_before,
                chips_after: step.chips_after,
                mult_before: step.mult_before,
                mult_after: step.mult_after,
                retrigger: step.retrigger,
                xmult: step.xmult.map(|factor| (factor * 100.0).round() as usize),
                money: step.money,
            })
            .collect(),
    )
}

pub(super) fn tag_view(tag: balatro_rs::tag::Tag, name: String) -> TagView {
    TagView {
        content: TAGS
            .iter()
            .find(|item| item.id.0 == tag.id())
            .expect("known tag"),
        name,
    }
}

/// The evaluation rows of a cleared Blind, skipping the empty ones like `evaluate_round`.
pub(super) fn reward_rows(
    reward: &balatro_rs::reward::RoundReward,
    interest_max: usize,
) -> Vec<RewardRow> {
    let mut rows = Vec::new();
    if reward.blind > 0 {
        rows.push(RewardRow {
            kind: RewardKind::Blind,
            dollars: reward.blind,
        });
    }
    if reward.hands_left > 0 {
        rows.push(RewardRow {
            kind: RewardKind::Hands {
                count: reward.hands_left,
            },
            dollars: reward.hand_dollars,
        });
    }
    for (joker, dollars) in &reward.jokers {
        rows.push(RewardRow {
            kind: RewardKind::Joker {
                id: JokerKey(joker.instance_id()),
                kind: ContentId(joker.id()),
                name: joker.name(),
            },
            dollars: *dollars,
        });
    }
    for (tag, dollars) in &reward.tags {
        rows.push(RewardRow {
            kind: RewardKind::Tag(ContentId(tag.id())),
            dollars: *dollars,
        });
    }
    if reward.interest > 0 {
        rows.push(RewardRow {
            kind: RewardKind::Interest { max: interest_max },
            dollars: reward.interest,
        });
    }
    rows
}
