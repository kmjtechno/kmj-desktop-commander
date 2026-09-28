mod policy;

use policy::{PolicyDecision, classify_operation};
use serde::Serialize;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![system_probe, evaluate_operation])
        .run(tauri::generate_context!())
        .expect("failed to run KMJ Desktop Commander");
}
