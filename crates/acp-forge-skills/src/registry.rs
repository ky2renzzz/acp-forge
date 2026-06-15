//! In-memory skill registry with versioned lookup.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::skill::Skill;

/// Stores skills keyed by `(name, version)`.
#[derive(Debug, Default)]
pub struct Registry {
    /// `name -> (version -> Skill)`.
    skills: HashMap<String, HashMap<String, Skill>>,
}

impl Registry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a skill. Replaces if same name+version already exists.
    pub fn insert(&mut self, skill: Skill) {
        self.skills
            .entry(skill.name.clone())
            .or_default()
            .insert(skill.version.clone(), skill);
    }

    /// Get a specific version of a skill.
    pub fn get(&self, name: &str, version: &str) -> Result<&Skill> {
        self.skills
            .get(name)
            .and_then(|versions| versions.get(version))
            .ok_or_else(|| Error::NotFound {
                name: format!("{name}@{version}"),
            })
    }

    /// Get the latest version of a skill (lexicographic — semver-aware sort
    /// would require an extra dependency; keep it simple).
    pub fn latest(&self, name: &str) -> Result<&Skill> {
        let versions = self
            .skills
            .get(name)
            .ok_or_else(|| Error::NotFound {
                name: name.to_owned(),
            })?;

        versions
            .values()
            .max_by(|a, b| a.version.cmp(&b.version))
            .ok_or_else(|| Error::NotFound {
                name: name.to_owned(),
            })
    }

    /// List all skill names.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.skills.keys().map(String::as_str).collect();
        names.sort();
        names
    }

    /// Total number of skills (all versions).
    #[must_use]
    pub fn len(&self) -> usize {
        self.skills.values().map(|v| v.len()).sum()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::Skill;
    use std::collections::HashMap as StdMap;

    fn skill(name: &str, version: &str) -> Skill {
        Skill {
            name: name.into(),
            version: version.into(),
            description: "test".into(),
            roles: vec![],
            template: String::new(),
            params: StdMap::new(),
            depends_on: vec![],
            tags: vec![],
        }
    }

    #[test]
    fn insert_and_retrieve() {
        let mut reg = Registry::new();
        reg.insert(skill("review", "0.1.0"));
        reg.insert(skill("review", "0.2.0"));

        assert_eq!(reg.get("review", "0.1.0").unwrap().version, "0.1.0");
        assert_eq!(reg.latest("review").unwrap().version, "0.2.0");
    }

    #[test]
    fn not_found() {
        let reg = Registry::new();
        assert!(reg.latest("nope").is_err());
    }
}
