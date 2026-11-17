//! Why a command changed the state, in words, so a log can be read without replaying it:
//! the blind, the hand and the effects that scored it, the notes of the engine, the Jokers
//! whose value moved, and what the shop or a pack offers.

use serde::{Deserialize, Serialize};

use crate::contract::{
    CardTrigger, Command, CurrentValue, HandKind, Note, Phase, RunUpdate, RunView, ScoreSource,
    ScoreStep,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explain {
    /// The blind just started: name, rule, score to reach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blind: Option<String>,
    /// The hand played: type, final Chips × Mult and score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand: Option<String>,
    /// The score trace, one entry per source and effect, repeats counted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Jokers whose current value changed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jokers: Vec<String>,
    /// The shop's offers after entering it or rerolling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shop: Vec<String>,
    /// The cards of a pack just opened.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pack: Vec<String>,
}

impl Explain {
    pub fn of(
        before: &RunView,
        command: Command,
        update: Option<&RunUpdate>,
        after: &RunView,
    ) -> Option<Self> {
        let update = update?;
        let mut explain = Self::default();
        if command == Command::StartBlind && after.phase == Phase::Playing {
            explain.blind = Some(blind(after));
        }
        if let Some(trace) = update.outcome.trace.as_ref() {
            explain.hand = hand(&trace.0, update.outcome.hand_score);
            explain.effects = effects(&trace.0, before);
        }
        explain.notes = update.notes.iter().map(note).collect();
        explain.jokers = joker_changes(before, after);
        let rerolled = command == Command::Reroll;
        if after.phase == Phase::Shop && (before.phase != Phase::Shop || rerolled) {
            explain.shop = shop(after);
        }
        if after.phase == Phase::Pack && before.phase != Phase::Pack {
            explain.pack = after
                .pack_choices
                .iter()
                .map(|item| item.name.clone())
                .collect();
        }
        (explain != Self::default()).then_some(explain)
    }
}

fn blind(view: &RunView) -> String {
    let mut text = format!("{} {}", view.blind, view.target);
    if !view.blind_rule.is_empty() {
        text.push_str(": ");
        text.push_str(&view.blind_rule);
    }
    text
}

fn hand(steps: &[ScoreStep], score: Option<usize>) -> Option<String> {
    let kind = steps.iter().find_map(|step| match step.source {
        ScoreSource::HandLevel(kind) => Some(kind),
        _ => None,
    });
    let last = steps.last()?;
    let mut text = format!(
        "{} {}x{}",
        kind.map_or("?", hand_name),
        last.chips_after,
        number(last.mult_after)
    );
    if let Some(score) = score {
        text.push_str(&format!(" = {score}"));
    }
    Some(text)
}

/// One entry per source and effect, in order of first appearance, `×n` when repeated.
pub(super) fn effects(steps: &[ScoreStep], before: &RunView) -> Vec<String> {
    let mut entries: Vec<(String, usize)> = Vec::new();
    for step in steps {
        if matches!(step.source, ScoreSource::HandLevel(_)) {
            continue;
        }
        let Some(effect) = effect(step) else {
            continue;
        };
        let entry = format!("{} {effect}", source(&step.source, before));
        match entries.iter_mut().find(|(text, _)| *text == entry) {
            Some((_, count)) => *count += 1,
            None => entries.push((entry, 1)),
        }
    }
    entries
        .into_iter()
        .map(|(text, count)| {
            if count > 1 {
                format!("{text} ×{count}")
            } else {
                text
            }
        })
        .collect()
}

fn source(source: &ScoreSource, before: &RunView) -> String {
    match source {
        ScoreSource::HandLevel(kind) => hand_name(*kind).to_owned(),
        ScoreSource::Card {
            trigger,
            rank,
            suit,
            ..
        } => {
            let card = format!("{rank}{}", initial(suit));
            match trigger {
                CardTrigger::Played => card,
                CardTrigger::Held => format!("{card} held"),
                CardTrigger::StoneKicker => format!("{card} stone"),
            }
        }
        ScoreSource::Joker { name, .. } | ScoreSource::JokerMessage { name, .. } => {
            (*name).to_owned()
        }
        ScoreSource::JokerOnCard { name, card, .. } => {
            match before.cards.iter().find(|held| held.id == *card) {
                Some(held) => format!("{name} on {}{}", held.rank, initial(held.suit)),
                None => format!("{name} on a card"),
            }
        }
        ScoreSource::Voucher(_, name) => (*name).to_owned(),
        ScoreSource::NotAllowed => "Boss".to_owned(),
        ScoreSource::Balanced => "Plasma Deck".to_owned(),
    }
}

/// What a step did, or `None` when it changed nothing worth reading.
fn effect(step: &ScoreStep) -> Option<String> {
    match step.source {
        ScoreSource::NotAllowed => return Some("not allowed".to_owned()),
        ScoreSource::Balanced => return Some("balanced".to_owned()),
        ScoreSource::JokerMessage { message, .. } => return Some(message.to_owned()),
        _ => {}
    }
    let mut parts = Vec::new();
    if step.chips_after != step.chips_before {
        let gained = step.chips_after as i64 - step.chips_before as i64;
        parts.push(format!("{gained:+} chips"));
    }
    match step.xmult {
        Some(factor) => parts.push(format!("x{}", number(factor as f64 / 100.0))),
        None if step.mult_after != step.mult_before => {
            let gained = step.mult_after - step.mult_before;
            let sign = if gained >= 0.0 { "+" } else { "" };
            parts.push(format!("{sign}{} mult", number(gained)));
        }
        None => {}
    }
    if step.money != 0 {
        parts.push(format!("{:+}$", step.money));
    }
    if step.retrigger && !parts.is_empty() {
        parts.push("again".to_owned());
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn note(note: &Note) -> String {
    match note {
        Note::CardPaid { label, amount, .. } => format!("{label} paid ${amount}"),
        Note::SealCreated { label, created, .. } => format!("{label} made {created}"),
        Note::TagGained(tag) => format!("gained {tag}"),
        Note::JokerMessage { name, note, .. } => format!("{name} {note}"),
    }
}

fn joker_changes(before: &RunView, after: &RunView) -> Vec<String> {
    after
        .held_jokers
        .iter()
        .filter_map(|joker| {
            let previous = before.held_jokers.iter().find(|old| old.id == joker.id)?;
            (previous.current != joker.current).then(|| {
                format!(
                    "{} {} -> {}",
                    joker.name,
                    current(previous.current),
                    current(joker.current)
                )
            })
        })
        .collect()
}

fn current(value: Option<CurrentValue>) -> String {
    match value {
        None => "-".to_owned(),
        Some(CurrentValue::Mult(value)) => format!("+{value} mult"),
        Some(CurrentValue::Chips(value)) => format!("+{value} chips"),
        Some(CurrentValue::XMult(value)) => format!("x{}", number(value as f64 / 100.0)),
        Some(CurrentValue::Money(value)) => format!("${value}"),
        Some(CurrentValue::Count(value)) => value.to_string(),
    }
}

fn shop(view: &RunView) -> Vec<String> {
    let jokers = view
        .offers
        .iter()
        .map(|offer| format!("{} ${}", offer.name, offer.price));
    let consumables = view
        .consumable_offers
        .iter()
        .map(|offer| format!("{} ${}", offer.name, offer.price));
    let cards = view
        .shop_cards
        .iter()
        .map(|(_, item)| format!("{} ${}", item.name, item.price));
    let packs = view
        .packs
        .iter()
        .map(|pack| format!("{} ${}", pack.name, pack.price));
    let vouchers = view
        .voucher_offers
        .iter()
        .map(|(voucher, _)| format!("{} (voucher)", voucher.name));
    jokers
        .chain(consumables)
        .chain(cards)
        .chain(packs)
        .chain(vouchers)
        .collect()
}

fn initial(suit: &str) -> char {
    suit.chars().next().unwrap_or('?')
}

/// A number without trailing zeros: 1.5, 12, 0.25.
fn number(value: f64) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn hand_name(kind: HandKind) -> &'static str {
    match kind {
        HandKind::HighCard => "High Card",
        HandKind::OnePair => "Pair",
        HandKind::TwoPair => "Two Pair",
        HandKind::ThreeOfAKind => "Three of a Kind",
        HandKind::Straight => "Straight",
        HandKind::Flush => "Flush",
        HandKind::FullHouse => "Full House",
        HandKind::FourOfAKind => "Four of a Kind",
        HandKind::StraightFlush => "Straight Flush",
        HandKind::RoyalFlush => "Royal Flush",
        HandKind::FiveOfAKind => "Five of a Kind",
        HandKind::FlushHouse => "Flush House",
        HandKind::FlushFive => "Flush Five",
    }
}
