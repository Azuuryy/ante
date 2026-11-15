use std::fmt;

use super::{Checkpoint, FORMAT_VERSION, Header, LogLine, LoggedCommand, Step};
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
    let mut run: Option<EngineRun> = None;
    let mut steps = 0;
    for (number, text) in lines.into_iter().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        let line: LogLine = serde_json::from_str(text).map_err(|error| ReplayError::Line {
            number: number + 1,
            message: error.to_string(),
        })?;
        match line {
            LogLine::Header(header) => run = Some(start(&header)?),
            LogLine::Step(step) => {
                let run = run
                    .as_mut()
                    .ok_or_else(|| ReplayError::Header("missing before the first step".into()))?;
                if let Some(divergence) = apply(run, &step)? {
                    return Ok(ReplayReport {
                        steps,
                        divergence: Some(divergence),
                    });
                }
                steps += 1;
            }
        }
    }
    Ok(ReplayReport {
        steps,
        divergence: None,
    })
}

fn start(header: &Header) -> Result<EngineRun, ReplayError> {
    if header.format_version != FORMAT_VERSION {
        return Err(ReplayError::Header(format!(
            "format {} (this build reads {FORMAT_VERSION})",
            header.format_version
        )));
    }
    if header.extra_jokers != cfg!(feature = "extra-jokers") {
        return Err(ReplayError::Header(
            "logged with a different optional Joker set".into(),
        ));
    }
    Ok(EngineRun::with_config(header.config()?))
}

fn apply(run: &mut EngineRun, step: &Step) -> Result<Option<Divergence>, ReplayError> {
    let before = RunDriver::view(run);
    let command = step
        .command
        .to_command(&before)
        .ok_or_else(|| ReplayError::Unresolvable {
            index: step.index,
            command: step.command.clone(),
        })?;
    let refusal = RunDriver::apply(run, command, TraceMode::Off)
        .err()
        .map(|error| error.to_string());
    let replayed = Checkpoint::of(&RunDriver::view(run));
    if replayed == step.after && refusal == step.refused {
        return Ok(None);
    }
    Ok(Some(Divergence {
        index: step.index,
        command: step.command.clone(),
        logged_refusal: step.refused.clone(),
        replayed_refusal: refusal,
        logged: step.after.clone(),
        replayed,
    }))
}
