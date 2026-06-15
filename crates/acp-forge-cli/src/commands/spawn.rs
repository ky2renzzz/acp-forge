//! `acp-forge spawn` — create an agent with a role.

use acp_forge_core::agent::AgentRole;
use acp_forge_core::Orchestrator;

pub async fn run(name: &str, role_str: &str) -> anyhow::Result<()> {
    let role = parse_role(role_str)?;

    let mut orch = Orchestrator::new();
    let id = orch.add_agent(name, role)?;

    println!("✓ Spawned agent {name} ({role_str}) — id: {id}");
    Ok(())
}

fn parse_role(s: &str) -> anyhow::Result<AgentRole> {
    match s {
        "explorer" => Ok(AgentRole::Explorer),
        "writer" => Ok(AgentRole::Writer),
        "test-runner" | "test_runner" => Ok(AgentRole::TestRunner),
        "reviewer" => Ok(AgentRole::Reviewer),
        "refactorer" => Ok(AgentRole::Refactorer),
        "shell" | "shell-operator" | "shell_operator" => Ok(AgentRole::ShellOperator),
        "docs" | "doc-writer" | "doc_writer" => Ok(AgentRole::DocWriter),
        "planner" => Ok(AgentRole::Planner),
        _ => anyhow::bail!(
            "unknown role: {s}. Valid: explorer, writer, test-runner, \
             reviewer, refactorer, shell, docs, planner"
        ),
    }
}
