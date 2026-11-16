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
