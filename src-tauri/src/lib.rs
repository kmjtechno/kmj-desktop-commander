mod jobs;
mod policy;
mod runner;

use jobs::{JobRecord, JobStore};
use policy::{PolicyDecision, classify_operation};
use runner::{RemoteProbeResult, RemoteProfile};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Serialize)]
struct SystemProbe {
    app_version: &'static str,
    platform: &'static str,
    architecture: &'static str,
    policy_mode: &'static str,
}

#[tauri::command]
fn system_probe() -> SystemProbe {
    SystemProbe {
        app_version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        policy_mode: "deny-by-default",
    }
}

#[tauri::command]
fn evaluate_operation(operation: String) -> PolicyDecision {
    classify_operation(&operation)
}

#[tauri::command]
fn list_jobs(jobs: State<'_, Mutex<JobStore>>) -> Result<Vec<JobRecord>, String> {
    let store = jobs
        .lock()
        .map_err(|_| "Job store lock poisoned".to_string())?;
    Ok(store.list())
}

#[tauri::command(async)]
fn remote_probe(
    profile: RemoteProfile,
    jobs: State<'_, Mutex<JobStore>>,
) -> Result<RemoteProbeResult, String> {
    if !classify_operation("remote.probe").is_allowed() {
        return Err("Remote probe denied by policy".into());
    }

    let target = format!("{}@{}:{}", profile.username, profile.host, profile.port);
    let job_id = {
        let mut store = jobs
            .lock()
            .map_err(|_| "Job store lock poisoned".to_string())?;
        store.start("remote.probe", &target)?
    };

    let result = runner::probe(&profile);
    let (success, summary) = match &result {
        Ok(probe) => (probe.success, summarize(&probe.output)),
        Err(error) => (false, error.clone()),
    };

    let mut store = jobs
        .lock()
        .map_err(|_| "Job store lock poisoned".to_string())?;
    store.finish(&job_id, success, summary)?;
    result
}

fn summarize(output: &str) -> String {
    output.lines().take(3).collect::<Vec<_>>().join(" | ")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let jobs_path = app.path().app_data_dir()?.join("jobs.json");
            app.manage(Mutex::new(JobStore::load(jobs_path)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            system_probe,
            evaluate_operation,
            list_jobs,
            remote_probe
        ])
        .run(tauri::generate_context!())
        .expect("failed to run KMJ Desktop Commander");
}
