use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub operation: String,
    pub target: String,
    pub status: JobStatus,
    pub started_ms: u64,
    pub finished_ms: Option<u64>,
    pub summary: Option<String>,
}

pub struct JobStore {
    path: PathBuf,
    jobs: Vec<JobRecord>,
}

impl JobStore {
    pub fn load(path: PathBuf) -> Self {
        let jobs = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        Self { path, jobs }
    }

    pub fn start(&mut self, operation: &str, target: &str) -> Result<String, String> {
        let now = now_ms();
        let id = format!("job-{now}-{}", self.jobs.len() + 1);
        self.jobs.push(JobRecord {
            id: id.clone(),
            operation: operation.into(),
            target: target.into(),
            status: JobStatus::Running,
            started_ms: now,
            finished_ms: None,
            summary: None,
        });
        self.persist()?;
        Ok(id)
    }

    pub fn finish(&mut self, id: &str, success: bool, summary: String) -> Result<(), String> {
        let job = self
            .jobs
            .iter_mut()
            .find(|job| job.id == id)
            .ok_or_else(|| "Job not found".to_string())?;
        job.status = if success { JobStatus::Succeeded } else { JobStatus::Failed };
        job.finished_ms = Some(now_ms());
        job.summary = Some(summary);
        self.persist()
    }

    pub fn list(&self) -> Vec<JobRecord> {
        self.jobs.iter().rev().take(50).cloned().collect()
    }

    fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let json = serde_json::to_vec_pretty(&self.jobs).map_err(|error| error.to_string())?;
        fs::write(&self.path, json).map_err(|error| error.to_string())
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
