mod commands;
mod config;
mod error;
mod git;
mod url;

use clap::{Parser, Subcommand};

const APP_VERSION: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " version ",
    env!("CARGO_PKG_VERSION"),
    " (rev:",
    env!("GIT_HASH"),
    ")"
);

#[derive(Parser)]
#[command(
    name = "bw",
    about = "A worktree management tool based on bare clone",
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, short = 'V', help = "Print version")]
    version: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone a repository as bare with worktree-friendly structure
    Get {
        /// Repository URL or path (e.g., github.com/user/repo, git@github.com:user/repo.git)
        repo: String,

        /// SSH clone (default)
        #[arg(long)]
        ssh: bool,

        /// HTTPS clone
        #[arg(long)]
        https: bool,

        /// Suffix for directory name (e.g., repo.suffix)
        #[arg(long, short = 's')]
        suffix: Option<String>,
    },
    /// Open config file in editor
    Config,
    /// Add a new worktree with a new branch
    Add {
        /// Branch name to create (e.g., feature/000). If omitted, auto-generates wip/MMDD-HHmmss
        branch: Option<String>,

        /// Base branch to create from (overrides bw.toml)
        #[arg(long, short = 'b')]
        base: Option<String>,
    },
    /// List worktrees and select with fzf
    List,
    /// Remove a worktree
    Remove {
        /// Worktree name (directory name)
        name: String,

        /// Force removal
        #[arg(long, short = 'f')]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("{APP_VERSION}");
        std::process::exit(0);
    }

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let Some(command) = cli.command else {
        eprintln!("No command specified. Use --help for usage.");
        std::process::exit(1);
    };

    match command {
        Commands::Get { repo, ssh, https, suffix } => {
            commands::get::execute(&repo, ssh, https, suffix)?;
        }
        Commands::Config => {
            commands::config::execute()?;
        }
        Commands::Add { branch, base } => {
            commands::bw::execute_add(branch.as_deref(), base)?;
        }
        Commands::List => {
            commands::bw::execute_list()?;
        }
        Commands::Remove { name, force } => {
            commands::bw::execute_remove(&name, force)?;
        }
    }

    Ok(())
}
