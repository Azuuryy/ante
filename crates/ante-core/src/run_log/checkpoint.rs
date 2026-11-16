use serde::{Deserialize, Serialize};

use crate::contract::{Phase, RunView, card_label};

/// The confirmed state after a command, enough to tell whether a replay diverges.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub phase: String,
    pub ante: usize,
    pub round: usize,
    pub money: usize,
    pub score: usize,
    pub hands: usize,
    pub discards: usize,
    pub reward: usize,
    /// Cards in hand by [`card_label`], `*` when selected.
    pub hand: Vec<String>,
    /// Jokers by name, with their edition when they have one.
    pub jokers: Vec<String>,
    pub consumables: Vec<String>,
    /// Tags waiting in the tag row; absent from format 1.
    #[serde(default)]
    pub tags: Vec<String>,
    pub deck_cards: usize,
}

impl Checkpoint {
    pub fn of(view: &RunView) -> Self {
        Self {
            phase: phase_name(view.phase).to_owned(),
            ante: view.ante,
            round: view.round,
            money: view.money,
            score: view.score,
            hands: view.hands,
            discards: view.discards,
            reward: view.reward,
            hand: view
                .cards
                .iter()
                .map(|card| {
                    let mut label = card_label(
                        card.rank,
                        card.suit,
                        &card.enhancement,
                        &card.edition,
                        &card.seal,
                    );
                    if card.selected {
                        label.push('*');
                    }
                    label
                })
                .collect(),
            jokers: view
                .held_jokers
                .iter()
                .map(|joker| match joker.edition.as_str() {
                    "" | "Base" => joker.name.to_owned(),
                    edition => format!("{} {}", joker.name, edition.to_lowercase()),
                })
                .collect(),
            consumables: view.consumables.iter().map(|c| c.name.clone()).collect(),
            tags: view.tags.iter().map(|tag| tag.name.clone()).collect(),
            deck_cards: view.deck_cards,
        }
    }
}

impl Checkpoint {
    /// The checkpoint as a log of `format` writes it: format 1 named cards in full
    /// (`QClubs Lucky *`), Jokers with `Base`, and had no tags.
    pub fn of_format(view: &RunView, format: u32) -> Self {
        let mut checkpoint = Self::of(view);
        if format == 1 {
            checkpoint.hand = view
                .cards
                .iter()
                .map(|card| {
                    let mut label = format!("{}{}", card.rank, card.suit);
                    for part in [&card.enhancement, &card.edition, &card.seal] {
                        if !part.is_empty() && part != "Base" && part != "None" {
                            label.push(' ');
                            label.push_str(part);
                        }
                    }
                    if card.selected {
                        label.push_str(" *");
                    }
                    label
                })
                .collect();
            checkpoint.jokers = view
                .held_jokers
                .iter()
                .map(|joker| {
                    format!("{} {}", joker.name, joker.edition)
                        .trim()
                        .to_owned()
                })
                .collect();
            checkpoint.tags = Vec::new();
        }
        checkpoint
    }
}

/// The fields of a [`Checkpoint`] that changed since the previous line. A format 1 line,
/// which stores every field, reads as a delta that replaces them all.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ante: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub money: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hands: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discards: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jokers: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumables: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deck_cards: Option<usize>,
}

/// Runs `$apply!(field)` for each checkpoint field.
macro_rules! each_field {
    ($apply:ident) => {
        $apply!(phase);
        $apply!(ante);
        $apply!(round);
        $apply!(money);
        $apply!(score);
        $apply!(hands);
        $apply!(discards);
        $apply!(reward);
        $apply!(hand);
        $apply!(jokers);
        $apply!(consumables);
        $apply!(tags);
        $apply!(deck_cards);
    };
}

impl CheckpointDelta {
    /// The fields of `next` that differ from `previous`.
    pub fn between(previous: &Checkpoint, next: &Checkpoint) -> Self {
        let mut delta = Self::default();
        macro_rules! diff {
            ($field:ident) => {
                if previous.$field != next.$field {
                    delta.$field = Some(ToOwned::to_owned(&next.$field));
                }
            };
        }
        each_field!(diff);
        delta
    }

    /// Writes the fields this delta carries onto `state`.
    pub fn apply_to(&self, state: &mut Checkpoint) {
        macro_rules! set {
            ($field:ident) => {
                if let Some(value) = &self.$field {
                    state.$field = ToOwned::to_owned(value);
                }
            };
        }
        each_field!(set);
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::BeforeBlind => "before_blind",
        Phase::Playing => "playing",
        Phase::Reward => "reward",
        Phase::Shop => "shop",
        Phase::Pack => "pack",
        Phase::Won => "won",
        Phase::Lost => "lost",
    }
}
