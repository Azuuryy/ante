use std::sync::LazyLock;

use super::*;

/// The rarity badge text of a Joker, as `JokerView::rarity` writes it.
fn rarity_name(rarity: balatro_types::Rarity) -> &'static str {
    use balatro_types::Rarity::*;
    match rarity {
        Common => "Common",
        Uncommon => "Uncommon",
        Rare => "Rare",
        Legendary => "Legendary",
    }
}

pub struct ExternalBackend;
impl GameBackend for ExternalBackend {
    fn start(&self, config: RunConfig) -> Box<dyn RunDriver> {
        Box::new(EngineRun::with_config(config))
    }

    fn catalog(&self, group: ContentGroup) -> &'static [Content] {
        use balatro_rs::joker::{JokerEffects, joker_catalog};
        static JOKERS: LazyLock<Vec<Content>> = LazyLock::new(|| {
            joker_catalog()
                .map(|joker| Content {
                    id: ContentId(joker.id()),
                    name: joker.name().to_owned(),
                    description: joker_description(joker.id()),
                    cost: joker.cost(),
                    supported: joker.is_implemented(),
                    unlock: unlock_condition(joker.id()),
                    rarity: Some(rarity_name(joker.rarity())),
                })
                .collect()
        });
        match group {
            ContentGroup::Jokers => &JOKERS,
            ContentGroup::Vouchers => &VOUCHERS,
            ContentGroup::Tags => &TAGS,
            ContentGroup::Tarots => &TAROTS,
            ContentGroup::Planets => &PLANETS,
            ContentGroup::Spectrals => &SPECTRALS,
            ContentGroup::Boosters => &BOOSTER_CONTENT,
            ContentGroup::Blinds => &BLINDS,
            ContentGroup::Enhancements => &ENHANCEMENTS,
            ContentGroup::Editions => &EDITIONS,
            ContentGroup::Seals => &SEALS,
        }
    }

    fn parse_seed(&self, seed: &str) -> u64 {
        balatro_rs::seed_from_str(seed)
    }
}

impl RunDriver for EngineRun {
    fn view(&self) -> RunView {
        EngineRun::view(self)
    }

    fn set_locked(&mut self, ids: Vec<String>) {
        self.game.set_locked(ids);
    }

    fn apply(&mut self, command: Command, trace: TraceMode) -> Result<RunUpdate, RunError> {
        let before = self.phase();
        let won = self.game.campaign_won;
        let outcome = self.resolve(
            command,
            match trace {
                TraceMode::Off => None,
                TraceMode::Capture => Some(false),
                TraceMode::Profile => Some(true),
            },
        );
        // A refused command leaves no notes behind for the next one.
        let notes = self.take_notes();
        let mut update = RunUpdate::confirmed(before, won, command, outcome?, self.view());
        update.notes = notes;
        Ok(update)
    }
}
