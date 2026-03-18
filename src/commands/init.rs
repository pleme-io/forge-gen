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

# ── Helm charts (via helm-forge / iac-forge) ────────────────────────────────
# Targets: helm
# [helm]
# targets = ["helm"]
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

# ── MCP servers (via mcp-forge) ─────────────────────────────────────────────
# Targets: mcp-rust
# [mcp]
# targets = ["mcp-rust"]
# name = "my-api"

# ── Shell completions (via completion-forge) ────────────────────────────────
# Targets: skim-tab, fish
# [completions]
# targets = ["skim-tab", "fish"]
# name = "my-tool"           # CLI command name (defaults to spec title)
# icon = "☁"                 # prompt icon
# grouping = "auto"          # auto, tag, path, or operation-id
# aliases = ["mt"]           # command aliases
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_manifest_is_valid_toml() {
        // The starter manifest is heavily commented; ensure it parses as
        // valid TOML (toml::Value accepts any valid document).
        let parsed: Result<toml::Value, _> = toml::from_str(STARTER_MANIFEST);
        assert!(
            parsed.is_ok(),
            "STARTER_MANIFEST is not valid TOML: {}",
            parsed.unwrap_err()
        );
    }

    #[test]
    fn init_creates_file() {
        let dir = std::env::temp_dir().join("forge_gen_test_init_creates");
        // Clean up any leftovers from a previous run.
        let _ = std::fs::remove_dir_all(&dir);

        let args = Args {
            dir: dir.to_str().unwrap().to_string(),
        };
        let result = run(args);
        assert!(result.is_ok(), "init should succeed in an empty directory");

        let manifest_path = dir.join("forge-gen.toml");
        assert!(manifest_path.exists(), "forge-gen.toml should be created");

        let content = std::fs::read_to_string(&manifest_path).unwrap();
        assert_eq!(content, STARTER_MANIFEST);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_refuses_overwrite() {
        let dir = std::env::temp_dir().join("forge_gen_test_init_overwrite");
        let _ = std::fs::create_dir_all(&dir);

        // Create an existing forge-gen.toml so that init should refuse.
        let manifest_path = dir.join("forge-gen.toml");
        std::fs::write(&manifest_path, "# existing").unwrap();

        let args = Args {
            dir: dir.to_str().unwrap().to_string(),
        };
        let result = run(args);
        assert!(result.is_err(), "init should refuse to overwrite existing file");

        // Verify the original content was not changed.
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        assert_eq!(content, "# existing");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
