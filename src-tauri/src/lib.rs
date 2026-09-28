mod entitlement;
mod jobs;
mod policy;
mod profiles;
mod runner;

use entitlement::{SignedEntitlement, VerifiedEntitlement};
use jobs::{JobRecord, JobStore};
use policy::{PolicyDecision, classify_operation};
use profiles::{ProfileStore, SavedProfile};
use runner::{RemoteOperation, RemoteProfile, RemoteResult};
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
fn performance_ready() {
    if std::env::var("KMJ_PERF_HARNESS").as_deref() == Ok("1") {
        println!("KMJ_PERF_UI_READY");
    }
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
fn verify_entitlement(
    artifact: SignedEntitlement,
    public_key: String,
    device_id: String,
    device_public_key_fingerprint: String,
) -> Result<VerifiedEntitlement, String> {
    entitlement::verify(
        &artifact,
        &public_key,
        &device_id,
        &device_public_key_fingerprint,
        entitlement::now_secs(),
    )
}

#[tauri::command]
fn list_jobs(jobs: State<'_, Mutex<JobStore>>) -> Result<Vec<JobRecord>, String> {
    let store = jobs
        .lock()
        .map_err(|_| "Job store lock poisoned".to_string())?;
    Ok(store.list())
}

#[tauri::command]
fn list_profiles(profiles: State<'_, Mutex<ProfileStore>>) -> Result<Vec<SavedProfile>, String> {
    let store = profiles
        .lock()
        .map_err(|_| "Profile store lock poisoned".to_string())?;
    Ok(store.list())
}

#[tauri::command]
fn save_profile(
    profile: SavedProfile,
    profiles: State<'_, Mutex<ProfileStore>>,
) -> Result<Vec<SavedProfile>, String> {
    let mut store = profiles
        .lock()
        .map_err(|_| "Profile store lock poisoned".to_string())?;
    store.save(profile)
}

#[tauri::command]
fn delete_profile(
    id: String,
    profiles: State<'_, Mutex<ProfileStore>>,
) -> Result<Vec<SavedProfile>, String> {
    let mut store = profiles
        .lock()
        .map_err(|_| "Profile store lock poisoned".to_string())?;
    store.delete(&id)
}

#[tauri::command(async)]
fn remote_execute(
    profile: RemoteProfile,
    operation: RemoteOperation,
    jobs: State<'_, Mutex<JobStore>>,
) -> Result<RemoteResult, String> {
    let policy_id = operation.policy_id();
    if !classify_operation(policy_id).is_allowed() {
        return Err(format!("Operation {policy_id} denied by policy"));
    }

    let target = format!("{}@{}:{}", profile.username, profile.host, profile.port);
    let job_id = {
        let mut store = jobs
            .lock()
            .map_err(|_| "Job store lock poisoned".to_string())?;
        store.start(operation.label(), &target)?
    };

    let result = runner::execute(&profile, operation);
    let (success, summary) = match &result {
        Ok(remote) => (remote.success, summarize(&remote.output)),
        Err(error) => (false, error.clone()),
    };

    let mut store = jobs
        .lock()
        .map_err(|_| "Job store lock poisoned".to_string())?;
    store.finish(&job_id, success, summary)?;
    result
}

fn summarize(output: &str) -> String {
    output.lines().take(4).collect::<Vec<_>>().join(" | ")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(Mutex::new(JobStore::load(data_dir.join("jobs.json"))));
            app.manage(Mutex::new(ProfileStore::load(
                data_dir.join("profiles.json"),
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            performance_ready,
            system_probe,
            evaluate_operation,
            verify_entitlement,
            list_jobs,
            list_profiles,
            save_profile,
            delete_profile,
            remote_execute
        ])
        .run(tauri::generate_context!())
        .expect("failed to run KMJ Desktop Commander");
}
