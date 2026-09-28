#[path = "../../src-tauri/src/policy.rs"]
mod policy;

use policy::classify_operation;
use serde::Serialize;
use std::{env, process::Command};

#[derive(Serialize)]
struct Probe<'a> {
    app: &'a str,
    mode: &'a str,
    policy_mode: &'a str,
    platform: &'a str,
    architecture: &'a str,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("probe");

    match command {
        "probe" => {
            let probe = Probe {
                app: "KMJ Desktop Commander",
                mode: "headless",
                policy_mode: "deny-by-default",
                platform: env::consts::OS,
                architecture: env::consts::ARCH,
            };
            println!("{}", serde_json::to_string_pretty(&probe).expect("serialize probe"));
        }
        "policy" => {
            let Some(operation) = args.get(2) else {
                eprintln!("usage: kmj-commander-headless policy <operation>");
                std::process::exit(2);
            };
            let decision = classify_operation(operation);
            println!("{}", serde_json::to_string_pretty(&decision).expect("serialize policy"));
        }
        "project-inspect" => {
            let decision = classify_operation("project.inspect");
            if !decision.is_allowed() {
                eprintln!("project.inspect denied by policy");
                std::process::exit(3);
            }
            let root = args.get(2).map(String::as_str).unwrap_or(".");
            if root.starts_with('-') || root.contains('\0') {
                eprintln!("invalid project root");
                std::process::exit(2);
            }
            let status = Command::new("git")
                .args(["-C", root, "status", "--short", "--branch"])
                .status()
                .expect("unable to execute git");
            std::process::exit(status.code().unwrap_or(1));
        }
        _ => {
            eprintln!("unknown command; denied by default");
            std::process::exit(3);
        }
    }
}
