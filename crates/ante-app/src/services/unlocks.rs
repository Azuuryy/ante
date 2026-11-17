//! Unlocks, like the original `check_for_unlock`: each confirmed command updates the
//! career statistics (`inc_career_stat`), then every locked item whose condition is now
//! met is unlocked. A seeded run counts statistics but unlocks nothing and records no
//! win, like the original.

mod conditions;

use ante_core::contract::{
    CardView, Command, ContentGroup, GameBackend, Phase, RewardKind, RunUpdate, RunView,
};

use crate::{app::navigation::DeckChoice, services::profile::Profile};
pub(crate) use conditions::{Discoveries, Moment, RoundWon};

/// What a run remembers for its unlocks.
#[derive(Clone, Debug, Default)]
pub(crate) struct RunTracker {
    pub(crate) seeded: bool,
    pub(crate) deck: &'static str,
    pub(crate) stake: usize,
    most_jokers: usize,
    hands_this_round: usize,
    discards_this_round: usize,
}

impl RunTracker {
    pub(crate) fn new(seeded: bool, deck: &'static str, stake: usize) -> Self {
        Self {
            seeded,
            deck,
            stake,
            ..Self::default()
        }
    }
}

/// The ids of the Jokers and vouchers `profile` has not unlocked, for the run's pools.
pub(crate) fn locked_ids(profile: &Profile, backend: &dyn GameBackend) -> Vec<String> {
    [ContentGroup::Jokers, ContentGroup::Vouchers]
        .into_iter()
        .flat_map(|group| backend.catalog(group))
        .filter(|item| !profile.is_unlocked(item.id.0, item.unlock))
        .map(|item| item.id.0.to_owned())
        .collect()
}

/// Career statistics a command adds to (`inc_career_stat`), as (name, amount).
fn career_counts(before: &RunView, command: Command, after: &RunView) -> Vec<(&'static str, u64)> {
    let selected: Vec<&CardView> = before.cards.iter().filter(|c| c.selected).collect();
    let spent = before.money.saturating_sub(after.money) as u64;
    let consumable_set = |cards: &[ante_core::contract::ConsumableView], index: usize| {
        cards
            .get(index)
            .map_or("", |card| card.category.as_str())
            .to_owned()
    };
    let mut counts = Vec::new();
    match command {
        Command::Play => {
            counts.push(("c_hands_played", 1));
            counts.push(("c_cards_played", selected.len() as u64));
            let faces = selected
                .iter()
                .filter(|c| matches!(c.rank, 'J' | 'Q' | 'K'))
                .count();
            counts.push(("c_face_cards_played", faces as u64));
        }
        Command::Discard => counts.push(("c_cards_discarded", selected.len() as u64)),
        Command::SellJoker { .. } => {
            counts.push(("c_jokers_sold", 1));
            counts.push(("c_cards_sold", 1));
        }
        Command::SellConsumable { .. } => counts.push(("c_cards_sold", 1)),
        Command::Reroll => {
            counts.push(("c_shop_rerolls", 1));
            counts.push(("c_shop_dollars_spent", spent));
        }
        Command::BuyJoker(_) | Command::BuyPack { .. } => {
            counts.push(("c_shop_dollars_spent", spent))
        }
        Command::RedeemVoucher { voucher, .. } => {
            counts.push(("c_shop_dollars_spent", spent));
            if voucher.0 == "v_blank" {
                counts.push(("v_blank_redeemed", 1));
            }
        }
        Command::BuyPlayingCard { .. } => {
            counts.push(("c_shop_dollars_spent", spent));
            counts.push(("c_playing_cards_bought", 1));
        }
        Command::BuyConsumable { index, .. } | Command::BuyAndUseConsumable { index, .. } => {
            counts.push(("c_shop_dollars_spent", spent));
            let set = consumable_set(&before.consumable_offers, index);
            match set.as_str() {
                "Tarot" => counts.push(("c_tarots_bought", 1)),
                "Planet" => counts.push(("c_planets_bought", 1)),
                _ => {}
            }
            if matches!(command, Command::BuyAndUseConsumable { .. }) {
                counts.extend(used(&set));
            }
        }
        Command::UseConsumable { index, .. } => {
            counts.extend(used(&consumable_set(&before.consumables, index)));
        }
        _ => {}
    }
    if before.phase != Phase::Lost && after.phase == Phase::Lost {
        counts.push(("c_losses", 1));
    }
    counts
}

fn used(set: &str) -> Option<(&'static str, u64)> {
    match set {
        "Tarot" => Some(("c_tarot_reading_used", 1)),
        "Planet" => Some(("c_planetarium_used", 1)),
        _ => None,
    }
}

/// What the profile has discovered, counted by kind.
fn discoveries(profile: &Profile, backend: &dyn GameBackend) -> Discoveries {
    let count = |group: ContentGroup| {
        backend
            .catalog(group)
            .iter()
            .filter(|item| profile.discovered(item.id.0))
            .count()
    };
    let total = [
        ContentGroup::Jokers,
        ContentGroup::Vouchers,
        ContentGroup::Tarots,
        ContentGroup::Planets,
        ContentGroup::Spectrals,
        ContentGroup::Boosters,
        ContentGroup::Enhancements,
        ContentGroup::Editions,
        ContentGroup::Seals,
    ]
    .into_iter()
    .map(count)
    .sum();
    Discoveries {
        total,
        tarots: count(ContentGroup::Tarots),
        planets: count(ContentGroup::Planets),
        blinds: count(ContentGroup::Blinds),
    }
}

/// Follows one confirmed command: updates the career statistics and the run's tracker,
/// then unlocks what is now met. Returns the names of the items unlocked.
pub(crate) fn observe(
    tracker: &mut RunTracker,
    before: &RunView,
    command: Command,
    update: &RunUpdate,
    profile: &mut Profile,
    backend: &dyn GameBackend,
) -> Vec<String> {
    let after = &update.view;
    let selected: Vec<&CardView> = before.cards.iter().filter(|c| c.selected).collect();
    let mut moment = Moment::default();
    match command {
        Command::StartBlind => {
            tracker.hands_this_round = 0;
            tracker.discards_this_round = 0;
        }
        Command::Play => {
            tracker.hands_this_round += 1;
            moment.played = Some(&selected);
            moment.hand_score = update.outcome.hand_score;
        }
        Command::Discard => {
            tracker.discards_this_round += 1;
            moment.discarded = Some(&selected);
        }
        Command::ContinueEndless => moment.continued = true,
        _ => {}
    }
    tracker.most_jokers = tracker.most_jokers.max(after.held_jokers.len());
    let round_won = command == Command::Play
        && before.phase == Phase::Playing
        && matches!(after.phase, Phase::Reward | Phase::Won);
    if round_won {
        moment.round_won = Some(RoundWon {
            boss: !matches!(before.blind.as_str(), "Small Blind" | "Big Blind"),
            one_hand_no_discard: tracker.hands_this_round == 1 && tracker.discards_this_round == 0,
            last_hand: before.selected_hand.map(|hand| hand.rank),
        });
    }
    let won = !before.campaign_won && after.campaign_won;
    if won {
        moment.run_won = Some(tracker.most_jokers);
    }
    if after.ante > before.ante {
        moment.ante_reached = Some(after.ante);
    }
    let interest_capped = (command == Command::CashOut).then(|| {
        before.reward_rows.iter().any(|row| match row.kind {
            RewardKind::Interest { max } => row.dollars >= max,
            _ => false,
        })
    });
    let counts = career_counts(before, command, after);
    let single_hand = round_won.then_some(tracker.hands_this_round == 1);
    let seeded = tracker.seeded;
    let (deck, stake) = (tracker.deck, tracker.stake);
    let _ = profile.update_progress(|progress| {
        for (stat, amount) in counts {
            progress.add(stat, amount);
        }
        // `c_single_hand_round_streak` and `c_round_interest_cap_streak`.
        match single_hand {
            Some(true) => progress.add("c_single_hand_round_streak", 1),
            Some(false) => progress.set("c_single_hand_round_streak", 0),
            None => {}
        }
        match interest_capped {
            Some(true) => progress.add("c_round_interest_cap_streak", 1),
            Some(false) => progress.set("c_round_interest_cap_streak", 0),
            None => {}
        }
        if won && !seeded {
            progress.record_win(deck, stake);
        }
    });
    if seeded {
        return Vec::new();
    }
    let found = discoveries(profile, backend);
    let decks = DeckChoice::ALL
        .into_iter()
        .map(|deck| (deck.id(), deck.name().to_owned(), deck.unlock()));
    let locked: Vec<(&'static str, String)> = [ContentGroup::Jokers, ContentGroup::Vouchers]
        .into_iter()
        .flat_map(|group| backend.catalog(group))
        .map(|item| (item.id.0, item.name.clone(), item.unlock))
        .chain(decks)
        .filter(|(id, _, condition)| !profile.is_unlocked(id, *condition))
        .map(|(id, name, _)| (id, name))
        .collect();
    let newly: Vec<(&'static str, String)> = locked
        .into_iter()
        .filter(|(id, _)| conditions::met(id, &moment, after, profile.progress(), found))
        .collect();
    if newly.is_empty() {
        return Vec::new();
    }
    let _ = profile.update_progress(|progress| {
        for (id, _) in &newly {
            progress.unlocked.insert((*id).to_owned());
        }
    });
    newly.into_iter().map(|(_, name)| name).collect()
}

#[cfg(test)]
mod tests;
