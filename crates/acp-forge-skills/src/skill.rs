//! Skill definition.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reusable agent skill template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique name (e.g. `code-review`, `api-docs`).
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// One-line description.
    pub description: String,
    /// Which agent roles can use this skill.
    #[serde(default)]
    pub roles: Vec<String>,
    /// The prompt template with `{{param}}` placeholders.
    pub template: String,
    /// Parameter definitions with optional defaults.
    #[serde(default)]
    pub params: HashMap<String, ParamDef>,
    /// Names of skills this one depends on (for composition).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Arbitrary tags for search/filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A parameter that the skill template expects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    /// Human-readable description.
    pub description: String,
    /// Default value (if any).
    #[serde(default)]
    pub default: Option<String>,
    /// Whether this parameter is required.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

impl Skill {
    /// The content hash of this skill (for registry dedup).
    #[must_use]
    pub fn content_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        hasher.update(self.version.as_bytes());
        hasher.update(self.template.as_bytes());
        hex::encode(hasher.finalize())
    }
}
