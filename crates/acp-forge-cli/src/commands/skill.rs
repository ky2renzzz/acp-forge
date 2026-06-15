//! `acp-forge skill` subcommands.

use std::collections::HashMap;
use std::path::Path;

use acp_forge_skills::{grok_compat, loader, template, Registry};

/// Load all skills from a directory (both TOML and SKILL.md formats).
fn load_all(path: &Path) -> anyhow::Result<Vec<acp_forge_skills::Skill>> {
    let mut skills = Vec::new();

    // Native TOML skills.
    if let Ok(toml_skills) = loader::load_dir(path) {
        skills.extend(toml_skills);
    }

    // xAI Grok Build / Anthropic SKILL.md skills.
    if let Ok(md_skills) = grok_compat::load_skill_md_dir(path) {
        skills.extend(md_skills);
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// List skills in a directory.
pub async fn list(dir: &str) -> anyhow::Result<()> {
    let path = Path::new(dir);
    if !path.exists() {
        println!("No skills directory at {dir}.");
        return Ok(());
    }

    let skills = load_all(path)?;
    if skills.is_empty() {
        println!("No skills found in {dir}.");
        return Ok(());
    }

    println!("Skills:");
    for s in &skills {
        let format = if s.tags.contains(&"grok-compat".to_owned()) {
            "SKILL.md"
        } else {
            "TOML"
        };
        let params: Vec<_> = s.params.keys().map(String::as_str).collect();
        println!(
            "  {} v{} [{format}] — {} [params: {}]",
            s.name,
            s.version,
            s.description,
            if params.is_empty() {
                "none".to_owned()
            } else {
                params.join(", ")
            }
        );
    }

    Ok(())
}

/// Render a skill template with parameters.
pub async fn render(dir: &str, name: &str, params: &[String]) -> anyhow::Result<()> {
    let path = Path::new(dir);
    let skills = load_all(path)?;

    let mut registry = Registry::new();
    for s in skills {
        registry.insert(s);
    }

    let skill = registry.latest(name)?;

    let values: HashMap<String, String> = params
        .iter()
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.to_owned(), v.to_owned()))
        })
        .collect();

    let rendered = template::render(skill, &values)?;
    println!("{rendered}");

    Ok(())
}
