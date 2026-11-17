//! A run as JSON lines, to retrace and replay the game logic: a header with the run
//! configuration, then one line per command with its result and the confirmed state
//! after it. Commands are stored by position in the confirmed view rather than by runtime
//! id, so a log replays in a fresh process; `replay` reruns one and reports the first
//! step whose state differs. File storage belongs to the client.

use serde::{Deserialize, Serialize};

use crate::contract::{CampaignGoal, Command, Deck, RunConfig, RunError, RunUpdate, RunView};

mod checkpoint;
mod explain;
mod portable;
mod replay;

pub use checkpoint::{Checkpoint, CheckpointDelta};
pub use explain::Explain;
pub use portable::LoggedCommand;
pub use replay::{Divergence, ReplayError, ReplayReport, replay};

/// Version of the line format; bump it when a line changes shape or meaning. Format 2
/// groups selections, stores state deltas and explains steps; the replay still reads 1.
pub const FORMAT_VERSION: u32 = 2;

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
    /// From White (1) to Gold (8); absent from logs before stakes, which were White.
    #[serde(default = "white_stake")]
    pub stake: usize,
}

fn white_stake() -> usize {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub index: usize,
    pub command: LoggedCommand,
    /// The refusal message when the command was refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
    /// Steps of the score trace the command produced.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub trace_steps: usize,
    /// The state fields that changed (every field in format 1).
    pub after: CheckpointDelta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explain: Option<Explain>,
}

/// Selection commands sent one after another (cards toggled, sorted or moved), on one
/// line: they change only the hand, and a run sends hundreds of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Select {
    /// Index of the first command; the others follow it.
    pub index: usize,
    /// Written as short codes: `t2` toggles hand card 2, `sr` and `ss` sort by rank and
    /// suit, `m3>` and `m3<` move hand card 3 right or left.
    #[serde(with = "selection_codes")]
    pub commands: Vec<LoggedCommand>,
    /// Refused commands, by position in `commands`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refused: Vec<(usize, String)>,
    /// The state fields changed by the whole group.
    pub after: CheckpointDelta,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "line", rename_all = "snake_case")]
pub enum LogLine {
    Header(Header),
    Step(Step),
    Select(Select),
}

mod selection_codes {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    use super::LoggedCommand;

    pub fn serialize<S: Serializer>(
        commands: &[LoggedCommand],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(commands.iter().map(code))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<LoggedCommand>, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .iter()
            .map(|text| {
                parse(text).ok_or_else(|| D::Error::custom(format!("bad selection {text}")))
            })
            .collect()
    }

    pub(super) fn code(command: &LoggedCommand) -> String {
        match command {
            LoggedCommand::ToggleCard { hand_index } => format!("t{hand_index}"),
            LoggedCommand::SortRank => "sr".to_owned(),
            LoggedCommand::SortSuit => "ss".to_owned(),
            LoggedCommand::MoveCard { hand_index, right } => {
                format!("m{hand_index}{}", if *right { '>' } else { '<' })
            }
            other => unreachable!("{other:?} is not a selection"),
        }
    }

    pub(super) fn parse(text: &str) -> Option<LoggedCommand> {
        match text {
            "sr" => return Some(LoggedCommand::SortRank),
            "ss" => return Some(LoggedCommand::SortSuit),
            _ => {}
        }
        if let Some(index) = text.strip_prefix('t') {
            return Some(LoggedCommand::ToggleCard {
                hand_index: index.parse().ok()?,
            });
        }
        let rest = text.strip_prefix('m')?;
        let (index, right) = match rest.strip_suffix('>') {
            Some(index) => (index, true),
            None => (rest.strip_suffix('<')?, false),
        };
        Some(LoggedCommand::MoveCard {
            hand_index: index.parse().ok()?,
            right,
        })
    }
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

impl Header {
    pub fn new(config: RunConfig) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            ante_version: env!("CARGO_PKG_VERSION").to_owned(),
            extra_jokers: cfg!(feature = "extra-jokers"),
            seed: config.seed,
            deck: config.deck.id().trim_start_matches("b_").to_owned(),
            goal: match config.goal {
                CampaignGoal::Standard => "standard",
                CampaignGoal::SingleAnte => "single_ante",
            }
            .to_owned(),
            starting_money: config.starting_money,
            stake: config.stake,
        }
    }

    pub fn config(&self) -> Result<RunConfig, ReplayError> {
        let deck = Deck::from_id(&format!("b_{}", self.deck))
            .ok_or_else(|| ReplayError::Header(format!("unknown deck {}", self.deck)))?;
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
            stake: self.stake,
        })
    }
}

/// Builds the lines of one run as the client applies commands to it. Selection commands
/// wait in a group, written before the next other command or by [`RunRecorder::finish`].
#[derive(Debug, Default)]
pub struct RunRecorder {
    next: usize,
    /// The state the last written line left.
    state: Checkpoint,
    pending: Option<(Select, Checkpoint)>,
}

impl RunRecorder {
    /// The recorder and the header line of a new run.
    pub fn start(config: RunConfig) -> (Self, String) {
        (Self::default(), line(&LogLine::Header(Header::new(config))))
    }

    /// The lines to write after one command, given the confirmed view it was sent against,
    /// its result and the confirmed view after it: none while a selection group grows.
    pub fn step(
        &mut self,
        before: &RunView,
        command: Command,
        result: Result<&RunUpdate, &RunError>,
        after: &RunView,
    ) -> Vec<String> {
        let logged = LoggedCommand::from_command(command, before);
        let refused = result.err().map(ToString::to_string);
        let checkpoint = Checkpoint::of(after);
        let index = self.next;
        self.next += 1;
        if logged.is_selection() {
            let (group, state) = self.pending.get_or_insert_with(|| {
                (
                    Select {
                        index,
                        commands: Vec::new(),
                        refused: Vec::new(),
                        after: CheckpointDelta::default(),
                    },
                    Checkpoint::default(),
                )
            });
            if let Some(refusal) = refused {
                group.refused.push((group.commands.len(), refusal));
            }
            group.commands.push(logged);
            *state = checkpoint;
            return Vec::new();
        }
        let mut lines: Vec<String> = self.finish().into_iter().collect();
        let step = Step {
            index,
            command: logged,
            trace_steps: result
                .ok()
                .and_then(|update| update.outcome.trace.as_ref())
                .map_or(0, |trace| trace.0.len()),
            refused,
            after: CheckpointDelta::between(&self.state, &checkpoint),
            explain: Explain::of(before, command, result.ok(), after),
        };
        self.state = checkpoint;
        lines.push(line(&LogLine::Step(step)));
        lines
    }

    /// The pending selection group's line, if any.
    pub fn finish(&mut self) -> Option<String> {
        let (mut group, state) = self.pending.take()?;
        group.after = CheckpointDelta::between(&self.state, &state);
        self.state = state;
        Some(line(&LogLine::Select(group)))
    }
}

fn line(line: &LogLine) -> String {
    serde_json::to_string(line).expect("log lines serialize")
}

#[cfg(test)]
mod tests;
