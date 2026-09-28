mod audit;
mod auth;
mod devices;
mod gateway;
#[path = "../../src-tauri/src/policy.rs"]
mod policy;

use policy::classify_operation;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub(crate) const CLOUDOS_ROOT: &str = "/home/info/kmj-cloudos";
const DEFAULT_AUDIT: &str = "/var/lib/kmj-commander/audit/events.jsonl";
const DEFAULT_DEVICES: &str = "/var/lib/kmj-commander/devices.json";

#[derive(Serialize)]
struct Probe<'a> {
    app: &'a str,
    mode: &'a str,
    policy_mode: &'a str,
    protocol: &'a str,
    platform: &'a str,
    architecture: &'a str,
}

#[derive(Debug)]
pub(crate) struct ExecOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
}

fn root(value: &str) -> Result<PathBuf, String> {
    if value != CLOUDOS_ROOT {
        return Err("CloudOS root denied".into());
    }
    let path = Path::new(value);
    if !path.is_absolute() || !path.join(".git").exists() {
        return Err("invalid CloudOS workspace".into());
    }
    let canonical = fs::canonicalize(path).map_err(|error| error.to_string())?;
    if canonical != Path::new(CLOUDOS_ROOT) {
        return Err("canonical root mismatch".into());
    }
    Ok(canonical)
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<ExecOutcome, String> {
    let result = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    let mut output = String::from_utf8_lossy(&result.stdout).to_string();
    output.push_str(&String::from_utf8_lossy(&result.stderr));
    output.truncate(262_144);
    Ok(ExecOutcome {
        success: result.status.success(),
        exit_code: result.status.code(),
        output,
    })
}

fn allowed(operation: &str) -> Result<(), String> {
    if classify_operation(operation).is_allowed() {
        Ok(())
    } else {
        Err("policy denied".into())
    }
}

pub(crate) fn execute_named(root_value: &str, operation: &str) -> Result<ExecOutcome, String> {
    if operation == "commander.probe" {
        allowed("project.inspect")?;
        return Ok(ExecOutcome {
            success: true,
            exit_code: Some(0),
            output: serde_json::to_string(&Probe {
                app: "KMJ Desktop Commander",
                mode: "headless",
                policy_mode: "deny-by-default",
                protocol: "KMJ-COMMANDER/1",
                platform: env::consts::OS,
                architecture: env::consts::ARCH,
            })
            .unwrap(),
        });
    }

    let root = root(root_value)?;
    match operation {
        "cloudos.inspect" => {
            allowed("project.inspect")?;
            run(&root, "git", &["status", "--short", "--branch"])
        }
        "cloudos.git_status" => {
            allowed("git.status")?;
            run(&root, "git", &["status", "--short", "--branch"])
        }
        "cloudos.diff_check" => {
            allowed("git.status")?;
            run(&root, "git", &["diff", "--check"])
        }
        "cloudos.py_compile" => {
            allowed("test.run")?;
            run(
                &root,
                "python3",
                &[
                    "-m",
                    "py_compile",
                    "src/hypervisor/runtime/libvirt_provider.py",
                ],
            )
        }
        "cloudos.provider_tests" => {
            allowed("test.run")?;
            run(
                &root,
                "python3",
                &["-m", "pytest", "-q", "tests/test_libvirt_provider.py"],
            )
        }
        "cloudos.full_tests" => {
            allowed("test.run")?;
            run(&root, "python3", &["-m", "pytest", "-q"])
        }
        _ => Err("denied by default".into()),
    }
}

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn output_hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn devices_path() -> PathBuf {
    env::var("KMJ_COMMANDER_DEVICES_PATH")
        .unwrap_or_else(|_| DEFAULT_DEVICES.into())
        .into()
}

fn secret() -> Result<String, String> {
    let secret = env::var("KMJ_COMMANDER_SIGNING_SECRET")
        .map_err(|_| "KMJ_COMMANDER_SIGNING_SECRET is required")?;
    if secret.len() < 32 {
        return Err("signing secret must be at least 32 characters".into());
    }
    Ok(secret)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("probe");
    let result: Result<i32, String> = match command {
        "probe" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&Probe {
                    app: "KMJ Desktop Commander",
                    mode: "headless",
                    policy_mode: "deny-by-default",
                    protocol: "KMJ-COMMANDER/1",
                    platform: env::consts::OS,
                    architecture: env::consts::ARCH,
                })
                .unwrap()
            );
            Ok(0)
        }
        "policy" => match args.get(2) {
            Some(operation) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&classify_operation(operation)).unwrap()
                );
                Ok(0)
            }
            None => Err("operation required".into()),
        },
        "cloudos" => match args.get(2) {
            Some(operation) => execute_named(
                args.get(3).map(String::as_str).unwrap_or(CLOUDOS_ROOT),
                &format!("cloudos.{operation}"),
            )
            .map(|outcome| {
                print!("{}", outcome.output);
                if outcome.success {
                    0
                } else {
                    outcome.exit_code.unwrap_or(1)
                }
            }),
            None => Err("operation required".into()),
        },
        "pair-device" => match args.get(2) {
            Some(device) => devices::pair(&devices_path(), device, now_secs()).map(|value| {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
                0
            }),
            None => Err("device id required".into()),
        },
        "revoke-device" => match args.get(2) {
            Some(device) => devices::revoke(&devices_path(), device, now_secs()).map(|value| {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
                0
            }),
            None => Err("device id required".into()),
        },
        "list-devices" => devices::list(&devices_path()).map(|values| {
            println!("{}", serde_json::to_string_pretty(&values).unwrap());
            0
        }),
        "mint-token" => match (args.get(2), args.get(3), args.get(4)) {
            (Some(subject), Some(device), Some(scopes)) => {
                let now = now_secs();
                let server =
                    env::var("KMJ_COMMANDER_SERVER_ID").unwrap_or_else(|_| "kmjtechnonet".into());
                match devices::is_active(&devices_path(), device) {
                    Ok(true) => {
                        let claims = auth::TokenClaims {
                            iss: "kmj-commander".into(),
                            sub: subject.clone(),
                            aud: "kmj-vps".into(),
                            server,
                            device: device.clone(),
                            iat: now,
                            nbf: now,
                            exp: now + 300,
                            jti: Uuid::new_v4().to_string(),
                            scopes: scopes.split(',').map(str::to_owned).collect(),
                        };
                        secret()
                            .and_then(|value| auth::mint(&claims, &value))
                            .map(|token| {
                                println!("{token}");
                                0
                            })
                    }
                    Ok(false) => Err("device is not paired or has been revoked".into()),
                    Err(error) => Err(error),
                }
            }
            _ => Err("subject, device id and comma-separated scopes required".into()),
        },
        "gateway" => {
            let bind = env::var("KMJ_COMMANDER_BIND").unwrap_or_else(|_| "127.0.0.1:8770".into());
            let server =
                env::var("KMJ_COMMANDER_SERVER_ID").unwrap_or_else(|_| "kmjtechnonet".into());
            let audit_path =
                env::var("KMJ_COMMANDER_AUDIT_PATH").unwrap_or_else(|_| DEFAULT_AUDIT.into());
            match secret() {
                Ok(value) => gateway::serve(
                    &bind,
                    gateway::GatewayState {
                        secret: value,
                        server,
                        audit_path: audit_path.into(),
                        devices_path: devices_path(),
                        replay: std::sync::Arc::new(std::sync::Mutex::new(
                            std::collections::HashMap::new(),
                        )),
                    },
                )
                .await
                .map(|_| 0),
                Err(error) => Err(error),
            }
        }
        _ => Err("denied by default".into()),
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(3)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_denied() {
        assert!(execute_named(CLOUDOS_ROOT, "shell").is_err());
        assert!(execute_named(CLOUDOS_ROOT, "system-reboot").is_err());
    }

    #[test]
    fn wrong_root_denied() {
        assert!(root("/tmp/kmj-cloudos").is_err());
        assert!(root("/home/info/kmj-cloudos;id").is_err());
    }

    #[test]
    fn output_hash_stable() {
        assert_eq!(output_hash("x").len(), 64);
    }
}
