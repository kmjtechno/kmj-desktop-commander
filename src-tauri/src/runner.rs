use serde::{Deserialize, Serialize};
use std::process::Command;

const OUTPUT_LIMIT: usize = 256 * 1024;

#[derive(Clone, Deserialize)]
pub struct RemoteProfile {
    pub host: String,
    pub username: String,
    pub port: u16,
    pub project_root: Option<String>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteOperation {
    Probe,
    ProjectInspect,
    GitStatus,
    GitDiffCheck,
    PhpTest,
    FrontendTypecheck,
    FrontendBuild,
    RustTest,
}

impl RemoteOperation {
    pub fn policy_id(self) -> &'static str {
        match self {
            Self::Probe => "remote.probe",
            Self::ProjectInspect => "project.inspect",
            Self::GitStatus | Self::GitDiffCheck => "git.status",
            Self::PhpTest | Self::FrontendTypecheck | Self::FrontendBuild | Self::RustTest => {
                "test.run"
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Probe => "remote.probe",
            Self::ProjectInspect => "project.inspect",
            Self::GitStatus => "git.status",
            Self::GitDiffCheck => "git.diff_check",
            Self::PhpTest => "php.test",
            Self::FrontendTypecheck => "frontend.typecheck",
            Self::FrontendBuild => "frontend.build",
            Self::RustTest => "rust.test",
        }
    }
}

#[derive(Serialize)]
pub struct RemoteResult {
    pub target: String,
    pub operation: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
}

pub fn execute(\n    profile: &RemoteProfile,\n    operation: RemoteOperation,\n) -> Result<RemoteResult, String> {
    validate(profile)?;
    let command = remote_command(profile, operation)?;
    let port = profile.port.to_string();

    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=yes",
            "-o",
            "ConnectTimeout=10",
            "-p",
            &port,
            "-l",
            &profile.username,
            &profile.host,
            &command,
        ])
        .output()
        .map_err(|error| format!("Unable to start OpenSSH client: {error}"))?;

    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.stderr.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    text.truncate(OUTPUT_LIMIT);

    Ok(RemoteResult {
        target: format!("{}@{}:{}", profile.username, profile.host, profile.port),
        operation: operation.label().into(),
        success: output.status.success(),
        exit_code: output.status.code(),
        output: text,
    })
}

fn remote_command(profile: &RemoteProfile, operation: RemoteOperation) -> Result<String, String> {
    if matches!(operation, RemoteOperation::Probe) {
        return Ok("printf 'KMJ_COMMANDER_PROBE\\n'; uname -srm; printf 'HOST='; hostname".into());
    }

    let root = profile
        .project_root
        .as_deref()
        .ok_or_else(|| "Project root is required for this operation".to_string())?;
    validate_project_root(root)?;

    let action = match operation {
        RemoteOperation::Probe => unreachable!(),
        RemoteOperation::ProjectInspect => {
            "printf 'ROOT='; pwd; printf '\\nBRANCH='; git branch --show-current 2>/dev/null || true; printf '\\nSTATUS\\n'; git status --short --branch 2>/dev/null || true; printf '\\nSTACK\\n'; test -f composer.json && echo PHP; test -f package.json && echo NODE; test -f Cargo.toml && echo RUST"
        }
        RemoteOperation::GitStatus => "git status --short --branch",
        RemoteOperation::GitDiffCheck => "git diff --check && git diff --stat",
        RemoteOperation::PhpTest => "php artisan test",
        RemoteOperation::FrontendTypecheck => "pnpm typecheck",
        RemoteOperation::FrontendBuild => "pnpm build",
        RemoteOperation::RustTest => "cargo test",
    };

    Ok(format!("cd -- {root} && {action}"))
}

fn validate(profile: &RemoteProfile) -> Result<(), String> {
    if profile.port == 0 {
        return Err("SSH port must be greater than zero".into());
    }
    if profile.host.is_empty()
        || profile.host.len() > 253
        || profile.host.starts_with('-')
        || !profile
            .host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
    {
        return Err("Invalid SSH host".into());
    }
    if profile.username.is_empty()
        || profile.username.starts_with('-')
        || profile.username.len() > 64
        || !profile
            .username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err("Invalid SSH username".into());
    }
    if let Some(root) = &profile.project_root {
        validate_project_root(root)?;
    }
    Ok(())
}

fn validate_project_root(root: &str) -> Result<(), String> {
    if root.is_empty()
        || !root.starts_with('/')
        || root.len() > 512
        || !root
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
    {
        return Err("Project root must be a safe absolute Unix path".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> RemoteProfile {
        RemoteProfile {
            host: "example.com".into(),
            username: "deploy-user".into(),
            port: 22,
            project_root: Some("/srv/app".into()),
        }
    }

    #[test]
    fn rejects_host_injection() {
        let mut item = profile();
        item.host = "example.com;touch".into();
        assert!(validate(&item).is_err());
    }

    #[test]
    fn rejects_username_injection() {
        let mut item = profile();
        item.username = "root -o ProxyCommand=x".into();
        assert!(validate(&item).is_err());
    }

    #[test]
    fn rejects_project_path_injection() {
        let mut item = profile();
        item.project_root = Some("/srv/app;rm".into());
        assert!(validate(&item).is_err());
    }

    #[test]
    fn commands_are_fixed_presets() {
        let item = profile();
        assert_eq!(
            remote_command(&item, RemoteOperation::GitStatus).unwrap(),
            "cd -- /srv/app && git status --short --branch"
        );
    }
}
