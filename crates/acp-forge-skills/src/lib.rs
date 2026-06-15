//! # acp-forge-skills
//!
//! Reusable skill templates for agent orchestration. A skill is a
//! structured prompt template with metadata, parameters, and optional
//! composition rules.
//!
//! Supports two formats:
//! - **TOML** (`.toml`) — native ACP Forge format with typed parameters
//! - **SKILL.md** — xAI Grok Build / Anthropic compatible format
//!   (YAML frontmatter + markdown body)

pub mod compose;
pub mod error;
pub mod grok_compat;
pub mod loader;
pub mod registry;
pub mod skill;
pub mod template;

pub use error::{Error, Result};
pub use registry::Registry;
pub use skill::Skill;
