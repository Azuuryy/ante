//! One JSON-lines file per run in local application data (`runs/`), written as commands
//! are confirmed, so a run can be retraced and replayed with `ante_core::run_log::replay`.
//! The most recent `KEEP` runs are kept. Logging never interrupts the game: a file that
//! cannot be written is dropped.

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use ante_core::{
    contract::{Command, RunConfig, RunError, RunUpdate, RunView},
    run_log::RunRecorder,
};

/// Runs kept in the log directory, the current one included.
pub(crate) const KEEP: usize = 20;

pub(crate) struct RunLogFile {
    recorder: RunRecorder,
    file: Option<BufWriter<File>>,
    /// Named in Dev diagnostics when the file cannot be written.
    #[cfg_attr(not(any(test, feature = "dev-tools")), allow(dead_code))]
    path: PathBuf,
}

impl RunLogFile {
    pub(crate) fn start(config: RunConfig) -> Self {
        Self::start_in(&crate::services::storage::local_dir().join("runs"), config)
    }

    fn start_in(dir: &Path, config: RunConfig) -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        let path = dir.join(format!("{millis:013}-{}.jsonl", config.seed));
        let (recorder, header) = RunRecorder::start(config);
        let file = fs::create_dir_all(dir)
            .and_then(|()| prune(dir, KEEP - 1))
            .and_then(|()| File::create(&path))
            .map(BufWriter::new)
            .ok();
        let mut log = Self {
            recorder,
            file,
            path,
        };
        log.write(&header);
        log
    }

    pub(crate) fn record(
        &mut self,
        before: &RunView,
        command: Command,
        result: Result<&RunUpdate, &RunError>,
        after: &RunView,
    ) {
        for line in self.recorder.step(before, command, result, after) {
            self.write(&line);
        }
    }

    #[cfg(test)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn write(&mut self, line: &str) {
        let written = self.file.as_mut().map(|file| {
            file.write_all(line.as_bytes())
                .and_then(|()| file.write_all(b"\n"))
                .and_then(|()| file.flush())
        });
        if let Some(Err(_error)) = written {
            crate::journal_event!(
                "run_log_failed",
                serde_json::json!({"path": self.path.display().to_string(), "error": _error.to_string()}),
            );
            self.file = None;
        }
    }
}

/// A run's last selections wait for the next command; write them when the log closes,
/// with a new run or when the game quits.
impl Drop for RunLogFile {
    fn drop(&mut self) {
        if let Some(line) = self.recorder.finish() {
            self.write(&line);
        }
    }
}

/// Removes the oldest run logs so that at most `keep` remain; names start with a
/// zero-padded timestamp, so they sort by age.
fn prune(dir: &Path, keep: usize) -> std::io::Result<()> {
    let mut logs: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect();
    logs.sort();
    let excess = logs.len().saturating_sub(keep);
    for old in &logs[..excess] {
        fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
