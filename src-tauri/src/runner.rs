use serde::{Deserialize, Serialize};
use std::process::Command;

const PROBE_COMMAND: &str = "printf 'KMJ_COMMANDER_PROBE\\n'; uname -srm; printf 'HOST='; hostname";
const OUTPUT_LIMIT: usize = 64 * 1024;

#[derive(Clone, Deserialize)]
pub struct RemoteProfile {
    pub host: String,
    pub username: String,
    pub port: u16,
}

#[derive(Serialize)]
pub struct RemoteProbeResult {
    pub target: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
}

pub fn probe(profile: &RemoteProfile) -> Result<RemoteProbeResult, String> {
    validate(profile)?;
    let target = format!("{}@{}", profile.username, profile.host);
    let port = profile.port.to_string();
    let output = Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            "-o", "StrictHostKeyChecking=yes",
            "-o", "ConnectTimeout=8",
            "-p", &port,
            &target,
            PROBE_COMMAND,
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

    Ok(RemoteProbeResult {
        target,
        success: output.status.success(),
        exit_code: output.status.code(),
        output: text,
    })
}

fn validate(profile: &RemoteProfile) -> Result<(), String> {
    if profile.port == 0 {
        return Err("SSH port must be greater than zero".into());
    }
    if profile.host.is_empty()
        || profile.host.len() > 253
        || !profile.host.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | ':')
        })
    {
        return Err("Invalid SSH host".into());
    }
    if profile.username.is_empty()
        || profile.username.starts_with('-')
        || profile.username.len() > 64
        || !profile.username.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
    {
        return Err("Invalid SSH username".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_shell_metacharacters() {
        let profile = RemoteProfile {
            host: "example.com;touch /tmp/pwn".into(),
            username: "root".into(),
            port: 22,
        };
        assert!(validate(&profile).is_err());
    }

    #[test]
    fn accepts_ipv4_dns_and_ipv6() {
        for host in ["example.com", "192.0.2.10", "2001:db8::1"] {
            assert!(validate(&RemoteProfile {
                host: host.into(),
                username: "deploy-user".into(),
                port: 22,
            }).is_ok());
        }
    }

    #[test]
    fn rejects_username_injection() {
        assert!(validate(&RemoteProfile {
            host: "example.com".into(),
            username: "root -o ProxyCommand=x".into(),
            port: 22,
        }).is_err());
    }
}
