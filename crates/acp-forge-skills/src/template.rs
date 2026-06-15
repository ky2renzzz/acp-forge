//! Minimal template rendering — replaces `{{param}}` placeholders.
//!
//! Intentionally simple. No logic, no loops, no filters.
//! If you need more, generate the prompt in code and pass it as a param.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::skill::Skill;

/// Render a skill's template with the given parameter values.
///
/// Missing required parameters cause an error. Optional parameters
/// with defaults are filled in automatically.
pub fn render(skill: &Skill, values: &HashMap<String, String>) -> Result<String> {
    let mut merged = HashMap::new();

    // Apply defaults first, then user values.
    for (name, def) in &skill.params {
        if let Some(default) = &def.default {
            merged.insert(name.as_str(), default.as_str());
        }
    }
    for (k, v) in values {
        merged.insert(k.as_str(), v.as_str());
    }

    // Check required params are present.
    for (name, def) in &skill.params {
        if def.required && !merged.contains_key(name.as_str()) {
            return Err(Error::MissingParam {
                param: name.clone(),
            });
        }
    }

    // Replace placeholders.
    let mut output = skill.template.clone();
    for (key, val) in &merged {
        let placeholder = format!("{{{{{key}}}}}");
        output = output.replace(&placeholder, val);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{ParamDef, Skill};

    fn test_skill() -> Skill {
        Skill {
            name: "test".into(),
            version: "0.1.0".into(),
            description: "a test skill".into(),
            roles: vec![],
            template: "Review {{file}} for {{focus}}".into(),
            params: HashMap::from([
                (
                    "file".into(),
                    ParamDef {
                        description: "file to review".into(),
                        default: None,
                        required: true,
                    },
                ),
                (
                    "focus".into(),
                    ParamDef {
                        description: "review focus".into(),
                        default: Some("correctness".into()),
                        required: false,
                    },
                ),
            ]),
            depends_on: vec![],
            tags: vec![],
        }
    }

    #[test]
    fn renders_with_defaults() {
        let skill = test_skill();
        let vals = HashMap::from([("file".to_owned(), "main.rs".to_owned())]);
        let out = render(&skill, &vals).unwrap();
        assert_eq!(out, "Review main.rs for correctness");
    }

    #[test]
    fn override_default() {
        let skill = test_skill();
        let vals = HashMap::from([
            ("file".to_owned(), "lib.rs".to_owned()),
            ("focus".to_owned(), "security".to_owned()),
        ]);
        let out = render(&skill, &vals).unwrap();
        assert_eq!(out, "Review lib.rs for security");
    }

    #[test]
    fn missing_required_param_errors() {
        let skill = test_skill();
        let vals = HashMap::new();
        assert!(render(&skill, &vals).is_err());
    }
}
