use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedProfile {
    pub id: String,
    pub label: String,
    pub host: String,
    pub username: String,
    pub port: u16,
    pub project_root: String,
}

pub struct ProfileStore {
    path: PathBuf,
    profiles: Vec<SavedProfile>,
}

impl ProfileStore {
    pub fn load(path: PathBuf) -> Self {
        let profiles = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        Self { path, profiles }
    }

    pub fn list(&self) -> Vec<SavedProfile> {
        self.profiles.clone()
    }

    pub fn save(&mut self, profile: SavedProfile) -> Result<Vec<SavedProfile>, String> {
        validate(&profile)?;
        if let Some(existing) = self.profiles.iter_mut().find(|item| item.id == profile.id) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
        self.persist()?;
        Ok(self.list())
    }

    pub fn delete(&mut self, id: &str) -> Result<Vec<SavedProfile>, String> {
        self.profiles.retain(|profile| profile.id != id);
        self.persist()?;
        Ok(self.list())
    }

    fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let data = serde_json::to_vec_pretty(&self.profiles).map_err(|error| error.to_string())?;
        fs::write(&self.path, data).map_err(|error| error.to_string())
    }
}

fn validate(profile: &SavedProfile) -> Result<(), String> {
    if profile.id.is_empty() || profile.id.len() > 80 {
        return Err("Profile id is invalid".into());
    }
    if profile.label.trim().is_empty() || profile.label.len() > 80 {
        return Err("Profile label is invalid".into());
    }
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
    if profile.project_root.is_empty()
        || !profile.project_root.starts_with('/')
        || profile.project_root.len() > 512
        || !profile
            .project_root
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

    #[test]
    fn project_path_rejects_shell_input() {
        let profile = SavedProfile {
            id: "server-1".into(),
            label: "Server".into(),
            host: "example.com".into(),
            username: "deploy".into(),
            port: 22,
            project_root: "/srv/app;rm".into(),
        };
        assert!(validate(&profile).is_err());
    }
}
