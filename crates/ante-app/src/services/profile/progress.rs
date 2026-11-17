//! A profile's progress, like the original `G.PROFILES[…]`: what has been unlocked,
//! the career statistics the unlocks count (`career_stats`), and the highest stake won
//! with each deck (`deck_usage[…].wins`).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Progress {
    pub(crate) unlocked: BTreeSet<String>,
    pub(crate) career: BTreeMap<String, u64>,
    /// Highest stake (1 = White) won with each deck, by deck id.
    pub(crate) deck_wins: BTreeMap<String, usize>,
}

impl Progress {
    /// Reads the fields of a version 5 profile; older profiles start empty.
    pub(crate) fn read(data: &mut Value) -> Self {
        let mut take = |key: &str| data.get_mut(key).map(Value::take);
        Self {
            unlocked: take("unlocked")
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default(),
            career: take("career")
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default(),
            deck_wins: take("deck_wins")
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default(),
        }
    }

    /// The fields to save, merged into the profile's JSON object.
    pub(crate) fn write(&self, data: &mut Value) {
        data["unlocked"] = json!(self.unlocked);
        data["career"] = json!(self.career);
        data["deck_wins"] = json!(self.deck_wins);
    }

    /// `inc_career_stat`.
    pub(crate) fn add(&mut self, stat: &str, amount: u64) {
        if amount > 0 {
            *self.career.entry(stat.to_owned()).or_default() += amount;
        }
    }

    pub(crate) fn set(&mut self, stat: &str, value: u64) {
        self.career.insert(stat.to_owned(), value);
    }

    pub(crate) fn stat(&self, stat: &str) -> u64 {
        self.career.get(stat).copied().unwrap_or(0)
    }

    /// A win with `deck` at `stake` (1 = White).
    pub(crate) fn record_win(&mut self, deck: &str, stake: usize) {
        let best = self.deck_wins.entry(deck.to_owned()).or_default();
        *best = (*best).max(stake);
    }

    /// `get_deck_win_stake(deck)`: the highest stake won with `deck`, 0 if none.
    pub(crate) fn deck_win_stake(&self, deck: &str) -> usize {
        self.deck_wins.get(deck).copied().unwrap_or(0)
    }

    /// `get_deck_win_stake()`: the highest stake won with any deck.
    pub(crate) fn best_win_stake(&self) -> usize {
        self.deck_wins.values().copied().max().unwrap_or(0)
    }
}
