//! ACP Forge CLI — multi-agent orchestration over ACP.

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "acp-forge",
    about = "Multi-agent orchestration framework for ACP",
    version,
    propagate_version = true
)]
struct Cli {
    /// Enable verbose logging.
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new forge session in the current repository.
    Init {
        /// Base branch to fork agent worktrees from.
        #[arg(short, long, default_value = "main")]
        base: String,
    },
    /// Spawn an agent with a given role.
    Spawn {
        /// Agent name.
        name: String,
        /// Role: explorer, writer, reviewer, test-runner, refactorer, shell, docs, planner.
        #[arg(short, long)]
        role: String,
    },
    /// List active agents and their status.
    Status,
    /// Check for file-level overlaps between agent worktrees.
    Overlaps,
    /// Record / replay execution sessions.
    #[command(subcommand)]
    Session(SessionCommand),
    /// Manage skills.
    #[command(subcommand)]
    Skill(SkillCommand),
}

#[derive(Subcommand)]
enum SessionCommand {
    /// List recorded sessions.
    List,
    /// Verify integrity of a recorded session.
    Verify {
        /// Path to the session JSON file.
        path: String,
    },
    /// Show a diff between two sessions.
    Diff {
        /// Path to the first session.
        a: String,
        /// Path to the second session.
        b: String,
    },
}

#[derive(Subcommand)]
enum SkillCommand {
    /// List registered skills.
    List {
        /// Directory to scan for skill files.
        #[arg(short, long, default_value = ".forge/skills")]
        dir: String,
    },
    /// Render a skill template with parameters.
    Render {
        /// Skill name.
        name: String,
        /// Parameters as key=value pairs.
        #[arg(short, long)]
        param: Vec<String>,
        /// Directory to scan for skill files.
        #[arg(short, long, default_value = ".forge/skills")]
        dir: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose {
            "debug"
        } else {
            "info"
        })
        .with_target(false)
        .init();

    match cli.command {
        Command::Init { base } => commands::init::run(&base).await,
        Command::Spawn { name, role } => commands::spawn::run(&name, &role).await,
        Command::Status => commands::status::run().await,
        Command::Overlaps => commands::overlaps::run().await,
        Command::Session(sub) => match sub {
            SessionCommand::List => commands::session::list().await,
            SessionCommand::Verify { path } => commands::session::verify(&path).await,
            SessionCommand::Diff { a, b } => commands::session::diff(&a, &b).await,
        },
        Command::Skill(sub) => match sub {
            SkillCommand::List { dir } => commands::skill::list(&dir).await,
            SkillCommand::Render { name, param, dir } => {
                commands::skill::render(&dir, &name, &param).await
            }
        },
    }
}
