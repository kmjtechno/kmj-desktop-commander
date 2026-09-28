use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Succeeded,
    Failed,
    Interrupted,
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
        let mut jobs: Vec<JobRecord> = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();

        // A process cannot truthfully leave an operation "running" across a restart.
        // Fence stale work as interrupted so it can never be mistaken for a live lease.
        let now = now_ms();
        let mut recovered = false;
        for job in &mut jobs {
            if job.status == JobStatus::Running {
                job.status = JobStatus::Interrupted;
                job.finished_ms = Some(now);
                job.summary = Some("Interrupted by Commander restart; safe to inspect/retry.".into());
                recovered = true;
            }
        }

        let store = Self { path, jobs };
        if recovered {
            let _ = store.persist();
        }
        store
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
        job.status = if success {
            JobStatus::Succeeded
        } else {
            JobStatus::Failed
        };
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
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|error| error.to_string())?;
        fs::rename(&tmp, &self.path).map_err(|error| error.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("kmj-commander-{name}-{}.json", now_ms()))
    }

    #[test]
    fn persists_completed_jobs_atomically() {
        let path = temp_path("persist");
        let mut store = JobStore::load(path.clone());
        let id = store.start("git.status", "deploy@example:22").unwrap();
        store.finish(&id, true, "clean".into()).unwrap();

        let restored = JobStore::load(path.clone()).list();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].status, JobStatus::Succeeded);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn restart_fences_stale_running_jobs() {
        let path = temp_path("recovery");
        let mut store = JobStore::load(path.clone());
        store.start("test.run", "deploy@example:22").unwrap();
        drop(store);

        let restored = JobStore::load(path.clone()).list();
        assert_eq!(restored[0].status, JobStatus::Interrupted);
        assert!(restored[0].finished_ms.is_some());
        let _ = fs::remove_file(path);
    }
}
