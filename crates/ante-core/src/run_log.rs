//! A run as JSON lines, to retrace and replay the game logic: a header with the run
//! configuration, then one line per command with its result and the confirmed state
//! after it. Commands are stored by position in the confirmed view rather than by runtime
//! id, so a log replays in a fresh process; `replay` reruns one and reports the first
//! step whose state differs. File storage belongs to the client.

use serde::{Deserialize, Serialize};

use crate::contract::{CampaignGoal, Command, Deck, RunConfig, RunError, RunUpdate, RunView};

mod checkpoint;
mod portable;
mod replay;

pub use checkpoint::Checkpoint;
pub use portable::LoggedCommand;
pub use replay::{Divergence, ReplayError, ReplayReport, replay};

/// Version of the line format; bump it when a line changes shape or meaning.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub format_version: u32,
    pub ante_version: String,
    /// Whether the run used the optional Jokers; a log replays only with the same set.
    pub extra_jokers: bool,
    pub seed: u64,
    pub deck: String,
    pub goal: String,
    pub starting_money: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub index: usize,
    pub command: LoggedCommand,
    /// The refusal message when the command was refused.
    pub refused: Option<String>,
    /// Steps of the score trace the command produced.
    pub trace_steps: usize,
    pub after: Checkpoint,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "line", rename_all = "snake_case")]
pub enum LogLine {
    Header(Header),
    Step(Step),
}

impl Header {
    pub fn new(config: RunConfig) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            ante_version: env!("CARGO_PKG_VERSION").to_owned(),
            extra_jokers: cfg!(feature = "extra-jokers"),
            seed: config.seed,
            deck: match config.deck {
                Deck::Red => "red",
                Deck::Blue => "blue",
            }
            .to_owned(),
            goal: match config.goal {
                CampaignGoal::Standard => "standard",
                CampaignGoal::SingleAnte => "single_ante",
            }
            .to_owned(),
            starting_money: config.starting_money,
        }
    }

    pub fn config(&self) -> Result<RunConfig, ReplayError> {
        let deck = match self.deck.as_str() {
            "red" => Deck::Red,
            "blue" => Deck::Blue,
            other => return Err(ReplayError::Header(format!("unknown deck {other}"))),
        };
        let goal = match self.goal.as_str() {
            "standard" => CampaignGoal::Standard,
            "single_ante" => CampaignGoal::SingleAnte,
            other => return Err(ReplayError::Header(format!("unknown goal {other}"))),
        };
        Ok(RunConfig {
            seed: self.seed,
            deck,
            goal,
            starting_money: self.starting_money,
        })
    }
}

/// Builds the lines of one run as the client applies commands to it.
#[derive(Debug, Default)]
pub struct RunRecorder {
    next: usize,
}

impl RunRecorder {
    /// The recorder and the header line of a new run.
    pub fn start(config: RunConfig) -> (Self, String) {
        (Self::default(), line(&LogLine::Header(Header::new(config))))
    }

    /// The line of one command, given the confirmed view it was sent against, its result
    /// and the confirmed view after it.
    pub fn step(
        &mut self,
        before: &RunView,
        command: Command,
        result: Result<&RunUpdate, &RunError>,
        after: &RunView,
    ) -> String {
        let step = Step {
            index: self.next,
            command: LoggedCommand::from_command(command, before),
            refused: result.err().map(ToString::to_string),
            trace_steps: result
                .ok()
                .and_then(|update| update.outcome.trace.as_ref())
                .map_or(0, |trace| trace.0.len()),
            after: Checkpoint::of(after),
        };
        self.next += 1;
        line(&LogLine::Step(step))
    }
}

fn line(line: &LogLine) -> String {
    serde_json::to_string(line).expect("log lines serialize")
}

#[cfg(test)]
mod tests;
