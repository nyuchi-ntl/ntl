//! NTL CLI — Command-line interface for the Neural Transfer Layer.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ntl")]
#[command(about = "Neural Transfer Layer — The Neural Transfer Layer for Modern Compute")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new NTL node
    Init,
    /// Start the NTL node
    Start {
        /// Run in development mode (connects to test network)
        #[arg(long)]
        dev: bool,
    },
    /// Emit a signal into the network
    Emit {
        /// Signal type (data, query, event, command, discovery)
        #[arg(long, short = 't')]
        r#type: String,
        /// JSON payload
        #[arg(long, short = 'p')]
        payload: Option<String>,
        /// Signal weight (0.0 - 1.0)
        #[arg(long, short = 'w', default_value = "0.5")]
        weight: f32,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Wait for correlated response (e.g., "5s", "30s")
        #[arg(long)]
        wait_correlation: Option<String>,
    },
    /// Listen for incoming signals
    Listen {
        /// Filter by signal type
        #[arg(long, short = 't')]
        r#type: Option<String>,
    },
    /// Show active synapses
    Synapses,
    /// Show node status
    Status,
    /// Show local topology
    Topology,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("✓ Initializing NTL node at ~/.ntl/");
            // TODO: Generate keypair, create config, create directories
        }
        Commands::Start { dev } => {
            let mode = if dev { "development" } else { "production" };
            println!("Starting NTL node in {mode} mode...");
            // TODO: Build and run node
        }
        Commands::Emit { r#type, weight, .. } => {
            println!("Emitting {type} signal (weight: {weight})...");
            // TODO: Build signal, sign, emit
        }
        Commands::Listen { r#type } => {
            let filter = r#type.as_deref().unwrap_or("all");
            println!("Listening for {filter} signals...");
            // TODO: Subscribe to signal stream
        }
        Commands::Synapses => println!("Active synapses:"),
        Commands::Status => println!("Node status:"),
        Commands::Topology => println!("Local topology:"),
    }
}
