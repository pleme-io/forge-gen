use std::path::Path;

use anyhow::{Result, bail};
use clap::Args as ClapArgs;
use colored::Colorize;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Output directory (default: current directory)
    #[arg(long, default_value = ".")]
    pub dir: String,
}

/// Starter manifest content with commented examples.
const STARTER_MANIFEST: &str = r#"# forge-gen.toml — unified code generation manifest
#
# Run: forge-gen generate
# Or override on the CLI: forge-gen generate --spec api.yaml --sdks go,python

[spec]
path = "openapi.yaml"
# version = "3.0"   # optional; informational only

[output]
dir = "./generated"

# ── SDK clients ───────────────────────────────────────────────────────────────
# Targets: go, python, javascript, typescript, typescript-axios, typescript-node,
#   typescript-angular, java, ruby, csharp, rust, kotlin, swift, dart, php, perl,
#   elixir, scala, haskell, c, cpp, lua, r, ocaml, clojure, elm, powershell, bash
# Use "all" to generate every SDK.
[sdks]
targets = ["go", "python", "typescript"]
# Per-target overrides (additional-properties for openapi-generator-cli):
# [sdks.overrides.go]
# packageName = "myapi"
# isGoSubmodule = "true"

# ── Server stubs ──────────────────────────────────────────────────────────────
# Targets: go-server, python-fastapi, rust-axum, spring, kotlin-spring
# [servers]
# targets = ["rust-axum"]

# ── IaC providers (via iac-forge) ─────────────────────────────────────────────
# Backends: terraform, pulumi, crossplane, ansible, pangea, steampipe
# [iac]
# backends = ["terraform", "pulumi"]
# resources = "./resources"   # directory of TOML resource specs
# provider = "./provider.toml"

# ── Schema generators ────────────────────────────────────────────────────────
# Targets: graphql-schema, protobuf-schema, mysql-schema, postgresql-schema
# [schemas]
# targets = ["graphql-schema", "protobuf-schema"]

# ── Documentation ────────────────────────────────────────────────────────────
# Targets: markdown, html, asciidoc, plantuml
# [docs]
# targets = ["markdown"]
"#;

/// Write a starter `forge-gen.toml` into the target directory.
///
/// # Errors
///
/// Returns an error if the file already exists or cannot be written.
pub fn run(args: Args) -> Result<()> {
    let dest = Path::new(&args.dir).join("forge-gen.toml");

    if dest.exists() {
        bail!(
            "forge-gen.toml already exists at {} — refusing to overwrite",
            dest.display()
        );
    }

    std::fs::create_dir_all(&args.dir)?;
    std::fs::write(&dest, STARTER_MANIFEST)?;

    println!(
        "{} Created {}",
        "done".green().bold(),
        dest.display()
    );

    Ok(())
}
