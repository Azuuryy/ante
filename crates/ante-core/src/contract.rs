use std::{fmt, time::Duration};

mod command;
mod content;
mod note;
mod trace;
mod view;

pub use command::*;
pub use content::*;
pub use note::*;
pub use trace::*;
pub use view::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CardKey(pub usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JokerKey(pub usize);

pub trait RunDriver: Send + Sync {
    fn view(&self) -> RunView;
    fn apply(&mut self, command: Command, trace: TraceMode) -> Result<RunUpdate, RunError>;
    /// Joker and voucher ids the profile has not unlocked: they stay out of the shop
    /// and packs for this run.
    fn set_locked(&mut self, _ids: Vec<String>) {}
}

pub trait GameBackend: Send + Sync + 'static {
    fn start(&self, config: RunConfig) -> Box<dyn RunDriver>;
    fn catalog(&self, group: ContentGroup) -> &'static [Content];
    fn parse_seed(&self, seed: &str) -> u64;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Deck {
    #[default]
    Red,
    Blue,
    Yellow,
    Green,
    Black,
    Magic,
    Nebula,
    Ghost,
    Abandoned,
    Checkered,
    Zodiac,
    Painted,
    Anaglyph,
    Plasma,
    Erratic,
}

impl Deck {
    pub const ALL: [Deck; 15] = [
        Deck::Red,
        Deck::Blue,
        Deck::Yellow,
        Deck::Green,
        Deck::Black,
        Deck::Magic,
        Deck::Nebula,
        Deck::Ghost,
        Deck::Abandoned,
        Deck::Checkered,
        Deck::Zodiac,
        Deck::Painted,
        Deck::Anaglyph,
        Deck::Plasma,
        Deck::Erratic,
    ];

    /// The original's key (`b_red`, …).
    pub fn id(self) -> &'static str {
        match self {
            Deck::Red => "b_red",
            Deck::Blue => "b_blue",
            Deck::Yellow => "b_yellow",
            Deck::Green => "b_green",
            Deck::Black => "b_black",
            Deck::Magic => "b_magic",
            Deck::Nebula => "b_nebula",
            Deck::Ghost => "b_ghost",
            Deck::Abandoned => "b_abandoned",
            Deck::Checkered => "b_checkered",
            Deck::Zodiac => "b_zodiac",
            Deck::Painted => "b_painted",
            Deck::Anaglyph => "b_anaglyph",
            Deck::Plasma => "b_plasma",
            Deck::Erratic => "b_erratic",
        }
    }

    pub fn from_id(id: &str) -> Option<Deck> {
        Deck::ALL.into_iter().find(|deck| deck.id() == id)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CampaignGoal {
    SingleAnte,
    #[default]
    Standard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    pub seed: u64,
    pub deck: Deck,
    pub goal: CampaignGoal,
    /// Overrides the deck's starting money; the Dev build uses it for testing.
    pub starting_money: Option<usize>,
    /// The stake, from White (1) to Gold (8).
    pub stake: usize,
}

#[derive(Debug)]
pub enum RunError {
    Unavailable,
    Simulation(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("This action is unavailable."),
            Self::Simulation(error) => f.write_str(error),
        }
    }
}

#[derive(Debug)]
pub struct Outcome {
    pub phase: Phase,
    pub round_score: usize,
    pub campaign_won: bool,
    pub hand_score: Option<usize>,
    pub trace: Option<ScoreTrace>,
    pub trace_time: Duration,
    pub discovered_item: Option<&'static str>,
}
