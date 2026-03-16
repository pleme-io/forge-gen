use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod commands;
mod manifest;
mod registry;

#[derive(Parser)]
#[command(
    name = "forge-gen",
    version,
    about = "Unified code generator — SDKs, IaC providers, schemas, and docs from OpenAPI specs"
)]
pub struct Cli {
    /// Enable JSON log output
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate code from an OpenAPI spec (SDKs, servers, IaC, schemas, docs)
    Generate(commands::generate::Args),

    /// List all available generators by category
    List(commands::list::Args),

    /// Validate an OpenAPI spec and print a summary
    Validate(commands::validate::Args),

    /// Create a starter forge-gen.toml manifest in the current directory
    Init(commands::init::Args),
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.json);

    match run(cli.command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "fatal");
            ExitCode::FAILURE
        }
    }
}

async fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Generate(args) => commands::generate::run(args).await,
        Command::List(args) => commands::list::run(args),
        Command::Validate(args) => commands::validate::run(args),
        Command::Init(args) => commands::init::run(args),
    }
}

fn init_tracing(json: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if json {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}
