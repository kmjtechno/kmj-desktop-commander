use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Device {
    pub id: String,
    pub status: String,
    pub paired_at: u64,
    pub revoked_at: Option<u64>,
}

#[derive(Default, Deserialize, Serialize)]
struct Registry {
    devices: Vec<Device>,
}

fn valid_id(id: &str) -> bool {
    (8..=64).contains(&id.len())
        && id
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'-' || value == b'_')
}

fn load(path: &Path) -> Result<Registry, String> {
    if !path.exists() {
        return Ok(Registry::default());
    }
    let content = fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&content).map_err(|_| "invalid device registry".into())
}

fn save(path: &Path, registry: &Registry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temp = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&temp)
        .map_err(|error| error.to_string())?;
    let content = serde_json::to_vec_pretty(registry).map_err(|error| error.to_string())?;
    file.write_all(&content)
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(temp, path).map_err(|error| error.to_string())
}

pub fn pair(path: &Path, id: &str, now: u64) -> Result<Device, String> {
    if !valid_id(id) {
        return Err("invalid device id".into());
    }
    let mut registry = load(path)?;
    let device = Device {
        id: id.into(),
        status: "active".into(),
        paired_at: now,
        revoked_at: None,
    };
    if let Some(existing) = registry.devices.iter_mut().find(|item| item.id == id) {
        *existing = device.clone();
    } else {
        registry.devices.push(device.clone());
    }
    save(path, &registry)?;
    Ok(device)
}

pub fn revoke(path: &Path, id: &str, now: u64) -> Result<Device, String> {
    let mut registry = load(path)?;
    let device = registry
        .devices
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or("unknown device")?;
    device.status = "revoked".into();
    device.revoked_at = Some(now);
    let result = device.clone();
    save(path, &registry)?;
    Ok(result)
}

pub fn is_active(path: &Path, id: &str) -> Result<bool, String> {
    if !valid_id(id) {
        return Ok(false);
    }
    Ok(load(path)?
        .devices
        .iter()
        .any(|item| item.id == id && item.status == "active"))
}

pub fn list(path: &Path) -> Result<Vec<Device>, String> {
    Ok(load(path)?.devices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn pair_then_revoke() {
        let path = std::env::temp_dir().join(format!("kmj-devices-{}.json", Uuid::new_v4()));
        pair(&path, "chatgpt-test-device", 100).unwrap();
        assert!(is_active(&path, "chatgpt-test-device").unwrap());
        revoke(&path, "chatgpt-test-device", 200).unwrap();
        assert!(!is_active(&path, "chatgpt-test-device").unwrap());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_device_id_denied() {
        let path = std::env::temp_dir().join(format!("kmj-devices-{}.json", Uuid::new_v4()));
        assert!(pair(&path, "../escape", 100).is_err());
    }
}
