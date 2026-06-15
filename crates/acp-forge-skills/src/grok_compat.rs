//! SKILL.md format loader — compatible with xAI Grok Build and
//! Anthropic's skill specification.
//!
//! Format:
//! ```markdown
//! ---
//! name: my-skill
//! description: What this skill does
//! ---
//! # Instructions
//! Markdown body with instructions for the agent.
//! ```
//!
//! The YAML frontmatter provides `name` and `description`.
//! The markdown body becomes the skill's template.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};
use crate::skill::Skill;

/// Parsed YAML frontmatter from a SKILL.md file.
#[derive(Debug)]
struct Frontmatter {
    name: String,
    description: String,
}

/// Parse a SKILL.md file into a [`Skill`].
///
/// The markdown body is used as the template verbatim — no `{{param}}`
/// expansion is applied unless the author included placeholders.
pub fn load_skill_md(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path).map_err(Error::Io)?;
    parse_skill_md(&content, path)
}

/// Parse SKILL.md content from a string.
pub fn parse_skill_md(content: &str, source: &Path) -> Result<Skill> {
    let (frontmatter, body) = split_frontmatter(content).ok_or_else(|| Error::InvalidFile {
        path: source.to_path_buf(),
        reason: "missing YAML frontmatter (expected --- delimiters)".into(),
    })?;

    let fm = parse_frontmatter(&frontmatter, source)?;

    Ok(Skill {
        name: fm.name,
        version: "0.0.0".into(), // SKILL.md doesn't carry a version
        description: fm.description,
        roles: Vec::new(),
        template: body.trim().to_owned(),
        params: HashMap::new(),
        depends_on: Vec::new(),
        tags: vec!["grok-compat".into()],
    })
}

/// Split a document into YAML frontmatter and markdown body.
///
/// Expects the document to start with `---\n`.
fn split_frontmatter(content: &str) -> Option<(String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing `---`.
    let after_open = &trimmed[3..];
    let close_pos = after_open.find("\n---")?;

    let yaml = after_open[..close_pos].trim().to_owned();
    let body = after_open[close_pos + 4..].to_owned();

    Some((yaml, body))
}

/// Parse the YAML frontmatter into structured fields.
///
/// We do minimal YAML parsing (key: value per line) to avoid pulling
/// in a full YAML crate. This handles the required fields that Grok
/// Build and Anthropic's spec define.
fn parse_frontmatter(yaml: &str, source: &Path) -> Result<Frontmatter> {
    let mut name = None;
    let mut description = None;

    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "name" => name = Some(value.to_owned()),
                "description" => description = Some(value.to_owned()),
                _ => {} // Ignore unknown fields gracefully.
            }
        }
    }

    let name = name.ok_or_else(|| Error::InvalidFile {
        path: source.to_path_buf(),
        reason: "frontmatter missing required field: name".into(),
    })?;

    let description = description.ok_or_else(|| Error::InvalidFile {
        path: source.to_path_buf(),
        reason: "frontmatter missing required field: description".into(),
    })?;

    Ok(Frontmatter { name, description })
}

/// Load all SKILL.md files from a directory.
///
/// Scans for files named `SKILL.md` in immediate subdirectories,
/// matching xAI's convention: `.grok/skills/<skill-name>/SKILL.md`.
pub fn load_skill_md_dir(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !dir.exists() {
        return Ok(skills);
    }

    let entries = std::fs::read_dir(dir).map_err(Error::Io)?;
    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();

        // Convention: each skill is a folder with a SKILL.md inside.
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                match load_skill_md(&skill_file) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => {
                        tracing::warn!(
                            path = %skill_file.display(),
                            error = %e,
                            "skipping invalid SKILL.md"
                        );
                    }
                }
            }
        }

        // Also handle standalone .md files named SKILL.md or *.skill.md.
        if path.is_file() {
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            if fname == "SKILL.md" || fname.ends_with(".skill.md") {
                match load_skill_md(&path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => {
                        tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "skipping invalid skill markdown"
                        );
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_valid_skill_md() {
        let content = r#"---
name: code-review
description: Review code for bugs and security issues
---
# Code Review

Review the provided file for correctness, security, and style.

## Checklist
- Logic errors
- SQL injection
- Hardcoded secrets
"#;

        let skill = parse_skill_md(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(skill.name, "code-review");
        assert_eq!(
            skill.description,
            "Review code for bugs and security issues"
        );
        assert!(skill.template.starts_with("# Code Review"));
        assert!(skill.tags.contains(&"grok-compat".to_owned()));
    }

    #[test]
    fn missing_frontmatter_errors() {
        let content = "# No frontmatter\nJust markdown.";
        let result = parse_skill_md(content, &PathBuf::from("bad.md"));
        assert!(result.is_err());
    }

    #[test]
    fn missing_name_errors() {
        let content = "---\ndescription: test\n---\nBody.";
        let result = parse_skill_md(content, &PathBuf::from("bad.md"));
        assert!(result.is_err());
    }

    #[test]
    fn quoted_values() {
        let content = "---\nname: \"my-skill\"\ndescription: 'A quoted description'\n---\nBody.";
        let skill = parse_skill_md(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "A quoted description");
    }
}
