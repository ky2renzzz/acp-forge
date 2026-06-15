//! # acp-forge-core
//!
//! Core orchestration primitives for multi-agent coordination over ACP.
//!
//! This crate provides the foundational types: agents, tasks, lifecycle
//! management, and the orchestrator that ties them together.

pub mod agent;
pub mod error;
pub mod id;
pub mod orchestrator;
pub mod task;

pub use agent::{Agent, AgentRole, AgentStatus};
pub use error::{Error, Result};
pub use id::Id;
pub use orchestrator::Orchestrator;
pub use task::{Task, TaskStatus};
