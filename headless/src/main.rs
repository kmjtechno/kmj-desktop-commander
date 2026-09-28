#[path = "../../src-tauri/src/policy.rs"]
mod policy;

use policy::classify_operation;
use serde::Serialize;
use std::{env, path::{Path, PathBuf}, process::{Command, ExitStatus}};

const CLOUDOS_ROOT: &str = "/home/info/kmj-cloudos";

#[derive(Serialize)]
struct Probe<'a> {
    app: &'a str,
    mode: &'a str,
    policy_mode: &'a str,
    platform: &'a str,
    architecture: &'a str,
}

#[derive(Clone, Copy)]
enum CloudOsOperation {
    Inspect,
    GitStatus,
    DiffCheck,
    PyCompile,
    ProviderTests,
}

impl CloudOsOperation {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "inspect" => Some(Self::Inspect),
            "git-status" => Some(Self::GitStatus),
            "diff-check" => Some(Self::DiffCheck),
            "py-compile" => Some(Self::PyCompile),
            "provider-tests" => Some(Self::ProviderTests),
            _ => None,
        }
    }

    fn policy_id(self) -> &'static str {
        match self {
            Self::Inspect => "project.inspect",
            Self::GitStatus | Self::DiffCheck => "git.status",
            Self::PyCompile | Self::ProviderTests => "test.run",
        }
    }
}

fn validate_cloudos_root(root: &str) -> Result<PathBuf, String> {
    if root != CLOUDOS_ROOT {
        return Err(format!("CloudOS root must be exactly {CLOUDOS_ROOT}"));
    }
    let path = Path::new(root);
    if !path.is_absolute() || !path.join(".git").exists() {
        return Err("CloudOS root is not an existing Git workspace".into());
    }
    Ok(path.to_path_buf())
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<ExitStatus, String> {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| format!("unable to execute {program}: {e}"))
}

fn cloudos_execute(root: &str, operation: CloudOsOperation) -> Result<ExitStatus, String> {
    let root = validate_cloudos_root(root)?;
    let decision = classify_operation(operation.policy_id());
    if !decision.is_allowed() {
        return Err(format!("{} denied by policy", operation.policy_id()));
    }

    match operation {
        CloudOsOperation::Inspect => run(&root, "git", &["status", "--short", "--branch"]),
        CloudOsOperation::GitStatus => run(&root, "git", &["status", "--short", "--branch"]),
        CloudOsOperation::DiffCheck => run(&root, "git", &["diff", "--check"]),
        CloudOsOperation::PyCompile => run(
            &root,
            "python3",
            &["-m", "py_compile", "src/hypervisor/runtime/libvirt_provider.py"],
        ),
        CloudOsOperation::ProviderTests => run(
            &root,
            "python3",
            &["-m", "pytest", "-q", "tests/test_libvirt_provider.py"],
        ),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("probe");

    let result = match command {
        "probe" => {
            let probe = Probe {
                app: "KMJ Desktop Commander",
                mode: "headless",
                policy_mode: "deny-by-default",
                platform: env::consts::OS,
                architecture: env::consts::ARCH,
            };
            println!("{}", serde_json::to_string_pretty(&probe).expect("serialize probe"));
            return;
        }
        "policy" => {
            let Some(operation) = args.get(2) else {
                eprintln!("usage: kmj-commander-headless policy <operation>");
                std::process::exit(2);
            };
            let decision = classify_operation(operation);
            println!("{}", serde_json::to_string_pretty(&decision).expect("serialize policy"));
            return;
        }
        "cloudos" => {
            let Some(name) = args.get(2) else {
                eprintln!("usage: kmj-commander-headless cloudos <inspect|git-status|diff-check|py-compile|provider-tests> [root]");
                std::process::exit(2);
            };
            let Some(operation) = CloudOsOperation::parse(name) else {
                eprintln!("unknown CloudOS operation; denied by default");
                std::process::exit(3);
            };
            let root = args.get(3).map(String::as_str).unwrap_or(CLOUDOS_ROOT);
            cloudos_execute(root, operation)
        }
        _ => {
            eprintln!("unknown command; denied by default");
            std::process::exit(3);
        }
    };

    match result {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(3);
        }
    }
}

#[cfg(test)]
mod headless_tests {
    use super::*;

    #[test]
    fn unknown_cloudos_operation_is_not_allowlisted() {
        assert!(CloudOsOperation::parse("shell").is_none());
        assert!(CloudOsOperation::parse("system-reboot").is_none());
    }

    #[test]
    fn cloudos_root_is_pinned() {
        assert!(validate_cloudos_root("/tmp/kmj-cloudos").is_err());
        assert!(validate_cloudos_root("/home/info/kmj-cloudos;id").is_err());
    }

    #[test]
    fn operations_map_only_to_allowed_policy_classes() {
        for op in [
            CloudOsOperation::Inspect,
            CloudOsOperation::GitStatus,
            CloudOsOperation::DiffCheck,
            CloudOsOperation::PyCompile,
            CloudOsOperation::ProviderTests,
        ] {
            assert!(classify_operation(op.policy_id()).is_allowed());
        }
    }
}
