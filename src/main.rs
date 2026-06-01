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

    /// Breakdown an Aha page or stdin into epics
    Breakdown {
        /// Aha page ID to breakdown
        #[arg(short, long, conflicts_with = "stdin")]
        page_id: Option<String>,

        /// Profile to use
        #[arg(short = 'P', long)]
        profile: Option<String>,

        /// Session ID to use (creates new if not provided)
        #[arg(short, long)]
        session: Option<String>,

        /// Read content from stdin instead of fetching from Aha
        #[arg(long)]
        stdin: bool,
    },

    /// Accept a session and create epics in Aha
    Accept {
        /// Session ID to accept
        #[arg(short, long)]
        session: String,

        /// Profile to use
        #[arg(short = 'P', long)]
        profile: Option<String>,
    },

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
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let result = match cli.command {
        Commands::Configure { profile } => cli::configure(profile).await,
        Commands::Breakdown {
            page_id,
            profile,
            session,
            stdin,
        } => cli::breakdown(page_id, profile, session, stdin).await,
        Commands::Accept { session, profile } => cli::accept(session, profile).await,
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
