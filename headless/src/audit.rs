use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Serialize)]
pub struct AuditRecord {
    pub event_id: String,
    pub request_id: String,
    pub timestamp: u64,
    pub principal: String,
    pub server: String,
    pub device: String,
    pub operation: String,
    pub outcome: String,
    pub exit_code: Option<i32>,
    pub output_sha256: String,
    pub previous_hash: String,
    pub record_hash: String,
}

pub fn append(path: &Path, mut record: AuditRecord) -> Result<String, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    record.previous_hash = last_hash(path).unwrap_or_else(|| "GENESIS".into());
    record.record_hash.clear();
    let canonical = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    record.record_hash = format!("{:x}", Sha256::digest(&canonical));
    let hash = record.record_hash.clone();
    let line = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())?;
    Ok(hash)
}

pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<_> = content
        .lines()
        .rev()
        .take(limit.min(200))
        .map(str::to_owned)
        .collect();
    lines.reverse();
    Ok(lines)
}

fn last_hash(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let line = content.lines().last()?;
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    value.get("record_hash")?.as_str().map(str::to_owned)
}
