use std::fmt;

use super::{Checkpoint, FORMAT_VERSION, Header, LogLine, LoggedCommand, Select, Step};
use crate::contract::{RunDriver, TraceMode};
use crate::engine::EngineRun;

/// The first step whose replayed state differs from the logged one.
#[derive(Clone, Debug, PartialEq)]
pub struct Divergence {
    pub index: usize,
    pub command: LoggedCommand,
    pub logged_refusal: Option<String>,
    pub replayed_refusal: Option<String>,
    pub logged: Checkpoint,
    pub replayed: Checkpoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayReport {
    pub steps: usize,
    pub divergence: Option<Divergence>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReplayError {
    Line {
        number: usize,
        message: String,
    },
    Header(String),
    /// The command's card or offer is not in the replayed view: the runs already differ.
    Unresolvable {
        index: usize,
        command: LoggedCommand,
    },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line { number, message } => write!(f, "line {number}: {message}"),
            Self::Header(message) => write!(f, "header: {message}"),
            Self::Unresolvable { index, command } => {
                write!(
                    f,
                    "step {index}: {command:?} has no target in the replayed view"
                )
            }
        }
    }
}

impl std::error::Error for ReplayError {}

/// Reruns a logged run from its header and reports the first step that diverges.
pub fn replay<'a>(lines: impl IntoIterator<Item = &'a str>) -> Result<ReplayReport, ReplayError> {
    let mut replayer: Option<Replayer> = None;
    let mut steps = 0;
    for (number, text) in lines.into_iter().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        let line: LogLine = serde_json::from_str(text).map_err(|error| ReplayError::Line {
            number: number + 1,
            message: error.to_string(),
        })?;
        let divergence = match line {
            LogLine::Header(header) => {
                replayer = Some(Replayer::start(&header)?);
                None
            }
            LogLine::Step(step) => {
                steps += 1;
                replayer_of(&mut replayer)?.step(&step)?
            }
            LogLine::Select(group) => {
                steps += group.commands.len();
                replayer_of(&mut replayer)?.select(&group)?
            }
        };
        if divergence.is_some() {
            return Ok(ReplayReport { steps, divergence });
        }
    }
    Ok(ReplayReport {
        steps,
        divergence: None,
    })
}

fn replayer_of(replayer: &mut Option<Replayer>) -> Result<&mut Replayer, ReplayError> {
    replayer
        .as_mut()
        .ok_or_else(|| ReplayError::Header("missing before the first step".into()))
}

/// The replayed run, the log's format, and the state the log describes so far.
struct Replayer {
    run: EngineRun,
    format: u32,
    logged: Checkpoint,
}

impl Replayer {
    fn start(header: &Header) -> Result<Self, ReplayError> {
        if !(1..=FORMAT_VERSION).contains(&header.format_version) {
            return Err(ReplayError::Header(format!(
                "format {} (this build reads 1 to {FORMAT_VERSION})",
                header.format_version
            )));
        }
        if header.extra_jokers != cfg!(feature = "extra-jokers") {
            return Err(ReplayError::Header(
                "logged with a different optional Joker set".into(),
            ));
        }
        Ok(Self {
            run: EngineRun::with_config(header.config()?),
            format: header.format_version,
            logged: Checkpoint::default(),
        })
    }

    /// Applies one logged command and returns its refusal, if any.
    fn send(
        &mut self,
        index: usize,
        command: &LoggedCommand,
    ) -> Result<Option<String>, ReplayError> {
        let before = RunDriver::view(&self.run);
        let resolved = command
            .to_command(&before)
            .ok_or_else(|| ReplayError::Unresolvable {
                index,
                command: command.clone(),
            })?;
        Ok(RunDriver::apply(&mut self.run, resolved, TraceMode::Off)
            .err()
            .map(|error| error.to_string()))
    }

    fn replayed(&self) -> Checkpoint {
        Checkpoint::of_format(&RunDriver::view(&self.run), self.format)
    }

    fn step(&mut self, step: &Step) -> Result<Option<Divergence>, ReplayError> {
        let refusal = self.send(step.index, &step.command)?;
        step.after.apply_to(&mut self.logged);
        Ok(self.compare(step.index, &step.command, step.refused.clone(), refusal))
    }

    fn select(&mut self, group: &Select) -> Result<Option<Divergence>, ReplayError> {
        for (position, command) in group.commands.iter().enumerate() {
            let index = group.index + position;
            let refusal = self.send(index, command)?;
            let logged = group
                .refused
                .iter()
                .find(|(at, _)| *at == position)
                .map(|(_, message)| message.clone());
            if refusal != logged {
                return Ok(Some(Divergence {
                    index,
                    command: command.clone(),
                    logged_refusal: logged,
                    replayed_refusal: refusal,
                    logged: self.logged.clone(),
                    replayed: self.replayed(),
                }));
            }
        }
        group.after.apply_to(&mut self.logged);
        let Some(last) = group.commands.last() else {
            return Ok(None);
        };
        let index = group.index + group.commands.len() - 1;
        let refused = group
            .refused
            .iter()
            .find(|(at, _)| *at + 1 == group.commands.len())
            .map(|(_, message)| message.clone());
        Ok(self.compare(index, last, refused.clone(), refused))
    }

    fn compare(
        &self,
        index: usize,
        command: &LoggedCommand,
        logged_refusal: Option<String>,
        replayed_refusal: Option<String>,
    ) -> Option<Divergence> {
        let replayed = self.replayed();
        (replayed != self.logged || logged_refusal != replayed_refusal).then(|| Divergence {
            index,
            command: command.clone(),
            logged_refusal,
            replayed_refusal,
            logged: self.logged.clone(),
            replayed,
        })
    }
}
