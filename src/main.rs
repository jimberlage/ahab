use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ahab::cli;
use ahab::error::AhabError;

#[derive(Parser)]
#[command(name = "ahab")]
#[command(about = "Aha Butler - A CLI helper for generating Aha tickets from plain text")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure ahab with API credentials and settings
    Configure {
        /// Profile name (defaults to "default")
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// Convert Aha pages to markdown in a session
    Convert {
        /// Page URLs or slugs (e.g., https://apexlabs.aha.io/pages/VAFM-N-91 or VAFM-N-91)
        pages: Vec<String>,

        /// Profile to use
        #[arg(short = 'P', long)]
        profile: Option<String>,

        /// Session ID to use (creates new if not provided)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Push a session and create epics in Aha
    Push {
        /// Session ID to push
        #[arg(short, long)]
        session: String,

        /// Profile to use
        #[arg(short = 'P', long)]
        profile: Option<String>,
    },

    /// Open a session in OpenCode for editing and review
    Critique {
        /// Session ID to critique
        #[arg(short, long)]
        session: String,
    },

    /// Launch MCP server for programmatic access to ahab commands
    Mcp,

    /// List all sessions
    #[command(name = "list-sessions")]
    ListSessions,

    /// Delete a session
    #[command(name = "delete-session")]
    DeleteSession {
        /// Session ID to delete
        session: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.verbose {
        "ahab=debug,info"
    } else {
        "ahab=info,warn"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let result = match cli.command {
        Commands::Configure { profile } => cli::configure(profile).await,
        Commands::Convert {
            pages,
            profile,
            session,
        } => cli::convert(pages, profile, session).await,
        Commands::Push { session, profile } => cli::push(session, profile).await,
        Commands::Critique { session } => cli::critique(session).await,
        Commands::Mcp => cli::mcp().await,
        Commands::ListSessions => cli::list_sessions().await,
        Commands::DeleteSession { session } => cli::delete_session(session).await,
    };

    match result {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);

            // Use non-zero exit code, with special handling for partial failures
            let exit_code = match e {
                AhabError::PartialFailure { .. } => 2,
                _ => 1,
            };

            std::process::exit(exit_code);
        }
    }
}
