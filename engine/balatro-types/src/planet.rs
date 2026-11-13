use crate::rank::{HandRank, Level};
use std::fmt;
use strum::{EnumIter, EnumString};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, EnumIter, EnumString, Eq, PartialEq, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Planets {
    Pluto,
    Mercury,
    Uranus,
    Venus,
    Saturn,
    Jupiter,
    Earth,
    Mars,
    Neptune,
    PlanetX,
    Ceres,
    Eris,
}

impl Planets {
    pub fn hand_rank(&self) -> HandRank {
        match self {
            Self::Pluto => HandRank::HighCard,
            Self::Mercury => HandRank::OnePair,
            Self::Uranus => HandRank::TwoPair,
            Self::Venus => HandRank::ThreeOfAKind,
            Self::Saturn => HandRank::Straight,
            Self::Jupiter => HandRank::Flush,
            Self::Earth => HandRank::FullHouse,
            Self::Mars => HandRank::FourOfAKind,
            Self::Neptune => HandRank::StraightFlush,
            Self::PlanetX => HandRank::FiveOfAKind,
            Self::Ceres => HandRank::FlushHouse,
            Self::Eris => HandRank::FlushFive,
        }
    }

    pub fn cost(&self) -> usize {
        3
    }

    pub fn sell_value(&self) -> usize {
        1
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pluto => "Pluto",
            Self::Mercury => "Mercury",
            Self::Uranus => "Uranus",
            Self::Venus => "Venus",
            Self::Saturn => "Saturn",
            Self::Jupiter => "Jupiter",
            Self::Earth => "Earth",
            Self::Mars => "Mars",
            Self::Neptune => "Neptune",
            Self::PlanetX => "Planet X",
            Self::Ceres => "Ceres",
            Self::Eris => "Eris",
        }
    }

    /// Save-file id for this planet card.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Pluto => "c_pluto",
            Self::Mercury => "c_mercury",
            Self::Uranus => "c_uranus",
            Self::Venus => "c_venus",
            Self::Saturn => "c_saturn",
            Self::Jupiter => "c_jupiter",
            Self::Earth => "c_earth",
            Self::Mars => "c_mars",
            Self::Neptune => "c_neptune",
            Self::PlanetX => "c_planet_x",
            Self::Ceres => "c_ceres",
            Self::Eris => "c_eris",
        }
    }

    /// Parses a save-file id back into a `Planets`.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "c_pluto" => Some(Self::Pluto),
            "c_mercury" => Some(Self::Mercury),
            "c_uranus" => Some(Self::Uranus),
            "c_venus" => Some(Self::Venus),
            "c_saturn" => Some(Self::Saturn),
            "c_jupiter" => Some(Self::Jupiter),
            "c_earth" => Some(Self::Earth),
            "c_mars" => Some(Self::Mars),
            "c_neptune" => Some(Self::Neptune),
            "c_planet_x" => Some(Self::PlanetX),
            "c_ceres" => Some(Self::Ceres),
            "c_eris" => Some(Self::Eris),
            _ => None,
        }
    }

    pub fn desc(&self) -> String {
        format!("Levels up {}", self.hand_rank_name())
    }

    fn hand_rank_name(&self) -> &str {
        match self {
            Self::Pluto => "High Card",
            Self::Mercury => "One Pair",
            Self::Uranus => "Two Pair",
            Self::Venus => "Three of a Kind",
            Self::Saturn => "Straight",
            Self::Jupiter => "Flush",
            Self::Earth => "Full House",
            Self::Mars => "Four of a Kind",
            Self::Neptune => "Straight Flush",
            Self::PlanetX => "Five of a Kind",
            Self::Ceres => "Flush House",
            Self::Eris => "Flush Five",
        }
    }

    /// PlanetX/Ceres/Eris are the "secret" planets (Five of a Kind, Flush
    /// House, Flush Five), only obtainable via those hand types, not sold
    /// in the shop by default.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::PlanetX | Self::Ceres | Self::Eris)
    }
}

/// Tracks the current level, chip/mult values, and play count for each
/// scored hand rank. Only 12 slots are stored, `RoyalFlush` shares
/// `StraightFlush`'s slot via `HandRank::scoring_rank`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Planetarium {
    highcard: Level,
    onepair: Level,
    twopair: Level,
    threeofkind: Level,
    straight: Level,
    flush: Level,
    fullhouse: Level,
    fourofkind: Level,
    straightflush: Level,
    fiveofkind: Level,
    flushhouse: Level,
    flushfive: Level,
}

impl Planetarium {
    pub fn new() -> Self {
        Planetarium {
            highcard: Level {
                level: 1,
                chips: 5,
                mult: 1,
                plays: 0,
            },
            onepair: Level {
                level: 1,
                chips: 10,
                mult: 2,
                plays: 0,
            },
            twopair: Level {
                level: 1,
                chips: 20,
                mult: 2,
                plays: 0,
            },
            threeofkind: Level {
                level: 1,
                chips: 30,
                mult: 3,
                plays: 0,
            },
            straight: Level {
                level: 1,
                chips: 30,
                mult: 4,
                plays: 0,
            },
            flush: Level {
                level: 1,
                chips: 35,
                mult: 4,
                plays: 0,
            },
            fullhouse: Level {
                level: 1,
                chips: 40,
                mult: 4,
                plays: 0,
            },
            fourofkind: Level {
                level: 1,
                chips: 60,
                mult: 7,
                plays: 0,
            },
            straightflush: Level {
                level: 1,
                chips: 100,
                mult: 8,
                plays: 0,
            },
            fiveofkind: Level {
                level: 1,
                chips: 120,
                mult: 12,
                plays: 0,
            },
            flushhouse: Level {
                level: 1,
                chips: 140,
                mult: 14,
                plays: 0,
            },
            flushfive: Level {
                level: 1,
                chips: 160,
                mult: 16,
                plays: 0,
            },
        }
    }
}

impl Default for Planetarium {
    fn default() -> Self {
        Self::new()
    }
}

impl Planetarium {
    /// Increment play count for the rank and return current level data.
    pub fn play(&mut self, rank: HandRank) -> Level {
        match rank.scoring_rank() {
            HandRank::HighCard => self.highcard.plays += 1,
            HandRank::OnePair => self.onepair.plays += 1,
            HandRank::TwoPair => self.twopair.plays += 1,
            HandRank::ThreeOfAKind => self.threeofkind.plays += 1,
            HandRank::Straight => self.straight.plays += 1,
            HandRank::Flush => self.flush.plays += 1,
            HandRank::FullHouse => self.fullhouse.plays += 1,
            HandRank::FourOfAKind => self.fourofkind.plays += 1,
            HandRank::StraightFlush => self.straightflush.plays += 1,
            HandRank::FiveOfAKind => self.fiveofkind.plays += 1,
            HandRank::FlushHouse => self.flushhouse.plays += 1,
            HandRank::FlushFive => self.flushfive.plays += 1,
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
        self.level(rank)
    }

    pub fn level(&self, rank: HandRank) -> Level {
        match rank.scoring_rank() {
            HandRank::HighCard => self.highcard,
            HandRank::OnePair => self.onepair,
            HandRank::TwoPair => self.twopair,
            HandRank::ThreeOfAKind => self.threeofkind,
            HandRank::Straight => self.straight,
            HandRank::Flush => self.flush,
            HandRank::FullHouse => self.fullhouse,
            HandRank::FourOfAKind => self.fourofkind,
            HandRank::StraightFlush => self.straightflush,
            HandRank::FiveOfAKind => self.fiveofkind,
            HandRank::FlushHouse => self.flushhouse,
            HandRank::FlushFive => self.flushfive,
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
    }

    /// Apply one level-up for the given rank, using the per-planet chip/mult
    /// increments. Since `RoyalFlush` shares `StraightFlush`'s slot,
    /// levelling up either one levels up the same stored data.
    pub fn level_up(&mut self, rank: HandRank) {
        let (chips_step, mult_step) = Self::level_step(rank);
        let level = self.level_mut(rank);
        level.level += 1;
        level.chips += chips_step;
        level.mult += mult_step;
    }

    /// Reverses one level-up for the given rank, stopping at level 1
    pub fn level_down(&mut self, rank: HandRank) {
        if self.level(rank).level <= 1 {
            return;
        }
        let (chips_step, mult_step) = Self::level_step(rank);
        let level = self.level_mut(rank);
        level.level -= 1;
        level.chips = level.chips.saturating_sub(chips_step);
        level.mult = level.mult.saturating_sub(mult_step);
    }

    /// Per-rank (chips, mult) step
    fn level_step(rank: HandRank) -> (usize, usize) {
        match rank.scoring_rank() {
            HandRank::HighCard => (10, 1),
            HandRank::OnePair => (15, 1),
            HandRank::TwoPair => (20, 1),
            HandRank::ThreeOfAKind => (20, 2),
            HandRank::Straight => (30, 3),
            HandRank::Flush => (15, 2),
            HandRank::FullHouse => (25, 2),
            HandRank::FourOfAKind => (30, 3),
            HandRank::StraightFlush => (40, 4),
            HandRank::FiveOfAKind => (35, 3),
            HandRank::FlushHouse => (40, 4),
            HandRank::FlushFive => (50, 3),
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
    }

    /// Mutable access to the stored `Level` for a rank (`RoyalFlush` shares
    /// `StraightFlush`'s slot).
    fn level_mut(&mut self, rank: HandRank) -> &mut Level {
        match rank.scoring_rank() {
            HandRank::HighCard => &mut self.highcard,
            HandRank::OnePair => &mut self.onepair,
            HandRank::TwoPair => &mut self.twopair,
            HandRank::ThreeOfAKind => &mut self.threeofkind,
            HandRank::Straight => &mut self.straight,
            HandRank::Flush => &mut self.flush,
            HandRank::FullHouse => &mut self.fullhouse,
            HandRank::FourOfAKind => &mut self.fourofkind,
            HandRank::StraightFlush => &mut self.straightflush,
            HandRank::FiveOfAKind => &mut self.fiveofkind,
            HandRank::FlushHouse => &mut self.flushhouse,
            HandRank::FlushFive => &mut self.flushfive,
            HandRank::RoyalFlush => unreachable!("scoring_rank() never returns RoyalFlush"),
        }
    }
}

impl fmt::Display for Planetarium {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ranks = [
            ("HC", self.highcard),
            ("1P", self.onepair),
            ("2P", self.twopair),
            ("3K", self.threeofkind),
            ("ST", self.straight),
            ("FL", self.flush),
            ("FH", self.fullhouse),
            ("4K", self.fourofkind),
            ("SF", self.straightflush),
            ("5K", self.fiveofkind),
            ("FLH", self.flushhouse),
            ("FF", self.flushfive),
        ];
        let parts: Vec<String> = ranks
            .iter()
            .map(|(abbr, lvl)| format!("{}:L{}", abbr, lvl.level))
            .collect();
        write!(f, "{}", parts.join(" | "))
    }
}

#[cfg(test)]
mod tests;
