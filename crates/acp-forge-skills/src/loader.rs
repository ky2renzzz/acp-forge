//! Load skills from TOML files.

use std::path::Path;

use crate::error::{Error, Result};
use crate::skill::Skill;

/// Load a single skill from a TOML file.
pub fn load_file(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path).map_err(Error::Io)?;
    let skill: Skill = toml::from_str(&content).map_err(|e| Error::InvalidFile {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    Ok(skill)
}

/// Load all `.toml` skills from a directory (non-recursive).
pub fn load_dir(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(Error::Io)?;
    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            match load_file(&path) {
                Ok(skill) => skills.push(skill),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "skipping invalid skill");
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}
