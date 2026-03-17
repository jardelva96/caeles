use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub const STATE_DIR: &str = ".caeles/state";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunRecord {
    pub run_id: String,
    pub capsule_id: String,
    pub capsule_name: String,
    pub manifest_path: String,
    pub status: String,
    pub started_at_unix_ms: u128,
    pub finished_at_unix_ms: u128,
}

pub fn ensure_state_dirs() -> anyhow::Result<PathBuf> {
    let base = PathBuf::from(STATE_DIR);
    fs::create_dir_all(base.join("logs"))?;
    Ok(base)
}

pub fn runs_file_path(base: &Path) -> PathBuf {
    base.join("runs.jsonl")
}

pub fn append_run_record(base: &Path, record: &RunRecord) -> anyhow::Result<()> {
    let line = serde_json::to_string(record)?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(runs_file_path(base))?;
    writeln!(f, "{line}")?;
    Ok(())
}

pub fn load_run_records(base: &Path) -> anyhow::Result<Vec<RunRecord>> {
    let runs_path = runs_file_path(base);
    if !runs_path.exists() {
        return Ok(vec![]);
    }

    let file = fs::File::open(runs_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: RunRecord = serde_json::from_str(&line)?;
        records.push(record);
    }

    Ok(records)
}

pub fn persist_run_records(base: &Path, records: &[RunRecord]) -> anyhow::Result<()> {
    let mut text = String::new();
    for r in records {
        text.push_str(&serde_json::to_string(r)?);
        text.push('\n');
    }
    fs::write(runs_file_path(base), text)?;
    Ok(())
}

pub fn log_file_path(base: &Path, run_id: &str) -> PathBuf {
    base.join("logs").join(format!("{run_id}.log"))
}

pub fn write_log_line(base: &Path, run_id: &str, message: &str) -> anyhow::Result<()> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path(base, run_id))?;
    writeln!(f, "{message}")?;
    Ok(())
}
