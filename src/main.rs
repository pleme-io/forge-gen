use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod commands;
mod manifest;
mod registry;

/// Top-level CLI definition for `forge-gen`.
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

/// Available subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Generate code from an `OpenAPI` spec (SDKs, servers, `IaC`, schemas, docs)
    Generate(Box<commands::generate::Args>),

    /// List all available generators by category
    List(commands::list::Args),

    /// Validate an `OpenAPI` spec and print a summary
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
        Command::Generate(args) => commands::generate::run(*args).await,
        Command::List(args) => commands::list::run(&args),
        Command::Validate(args) => commands::validate::run(&args),
        Command::Init(args) => commands::init::run(&args),
    }
}

fn init_tracing(json: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if json {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parse_list_subcommand() {
        let cli = Cli::try_parse_from(["forge-gen", "list"]).unwrap();
        assert!(matches!(cli.command, Command::List(_)));
        assert!(!cli.json);
    }

    #[test]
    fn cli_parse_list_with_category() {
        let cli = Cli::try_parse_from(["forge-gen", "list", "--category", "sdk"]).unwrap();
        match cli.command {
            Command::List(args) => assert_eq!(args.category.as_deref(), Some("sdk")),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn cli_parse_validate_subcommand() {
        let cli = Cli::try_parse_from(["forge-gen", "validate", "--spec", "api.yaml"]).unwrap();
        match cli.command {
            Command::Validate(args) => assert_eq!(args.spec, "api.yaml"),
            _ => panic!("expected Validate"),
        }
    }

    #[test]
    fn cli_parse_init_subcommand() {
        let cli = Cli::try_parse_from(["forge-gen", "init"]).unwrap();
        assert!(matches!(cli.command, Command::Init(_)));
    }

    #[test]
    fn cli_parse_init_with_dir() {
        let cli = Cli::try_parse_from(["forge-gen", "init", "--dir", "/tmp/foo"]).unwrap();
        match cli.command {
            Command::Init(args) => assert_eq!(args.dir, "/tmp/foo"),
            _ => panic!("expected Init"),
        }
    }

    #[test]
    fn cli_parse_generate_minimal() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--sdks",
            "go",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.spec.as_deref(), Some("api.yaml"));
                assert_eq!(args.sdks.as_deref(), Some("go"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_all_target_flags() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--sdks",
            "go,python",
            "--servers",
            "rust-axum",
            "--iac",
            "terraform",
            "--schemas",
            "graphql-schema",
            "--docs",
            "markdown",
            "--helm",
            "helm",
            "--mcp",
            "mcp-rust",
            "--completions",
            "fish",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.sdks.as_deref(), Some("go,python"));
                assert_eq!(args.servers.as_deref(), Some("rust-axum"));
                assert_eq!(args.iac.as_deref(), Some("terraform"));
                assert_eq!(args.schemas.as_deref(), Some("graphql-schema"));
                assert_eq!(args.docs.as_deref(), Some("markdown"));
                assert_eq!(args.helm.as_deref(), Some("helm"));
                assert_eq!(args.mcp.as_deref(), Some("mcp-rust"));
                assert_eq!(args.completions.as_deref(), Some("fish"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_optional_name_flags() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--mcp",
            "mcp-rust",
            "--mcp-name",
            "my-api",
            "--completions",
            "fish",
            "--completion-name",
            "my-tool",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.mcp_name.as_deref(), Some("my-api"));
                assert_eq!(args.completion_name.as_deref(), Some("my-tool"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_iac_resource_flags() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--iac",
            "terraform",
            "--resources",
            "./res",
            "--provider",
            "./prov.toml",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.resources.as_deref(), Some("./res"));
                assert_eq!(args.provider.as_deref(), Some("./prov.toml"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_helm_resource_flags() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--helm",
            "helm",
            "--helm-resources",
            "./hr",
            "--helm-provider",
            "./hp.toml",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.helm_resources.as_deref(), Some("./hr"));
                assert_eq!(args.helm_provider.as_deref(), Some("./hp.toml"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_parallel_default_true() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--sdks",
            "go",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => assert!(args.parallel),
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_generate_manifest_flag() {
        let cli = Cli::try_parse_from([
            "forge-gen",
            "generate",
            "--spec",
            "api.yaml",
            "--sdks",
            "go",
            "--manifest",
            "custom.toml",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.manifest.as_deref(), Some("custom.toml"));
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn cli_parse_json_flag_global() {
        let cli = Cli::try_parse_from(["forge-gen", "--json", "list"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn cli_parse_no_subcommand_errors() {
        assert!(Cli::try_parse_from(["forge-gen"]).is_err());
    }

    #[test]
    fn cli_parse_unknown_subcommand_errors() {
        assert!(Cli::try_parse_from(["forge-gen", "bogus"]).is_err());
    }

    #[test]
    fn cli_parse_validate_missing_spec_errors() {
        assert!(Cli::try_parse_from(["forge-gen", "validate"]).is_err());
    }
}
