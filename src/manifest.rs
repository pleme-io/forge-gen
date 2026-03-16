use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::commands::generate::Args as CliArgs;

/// Top-level `forge-gen.toml` schema.
#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    pub spec: Option<SpecConfig>,
    pub output: Option<OutputConfig>,
    pub sdks: Option<TargetList>,
    pub servers: Option<TargetList>,
    pub iac: Option<IacConfig>,
    pub schemas: Option<TargetList>,
    pub docs: Option<TargetList>,
}

#[derive(Debug, Deserialize)]
pub struct SpecConfig {
    pub path: String,
    pub version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OutputConfig {
    pub dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TargetList {
    pub targets: Vec<String>,
    #[serde(default)]
    pub overrides: Option<HashMap<String, HashMap<String, String>>>,
}

#[derive(Debug, Deserialize)]
pub struct IacConfig {
    pub backends: Vec<String>,
    pub resources: Option<String>,
    pub provider: Option<String>,
}

/// Resolved configuration used by the generate orchestrator.
#[derive(Debug)]
pub struct GenerateConfig {
    pub spec: String,
    pub output_dir: String,
    pub sdks: Vec<String>,
    pub servers: Vec<String>,
    pub iac_backends: Vec<String>,
    pub iac_resources: Option<String>,
    pub iac_provider: Option<String>,
    pub schemas: Vec<String>,
    pub docs: Vec<String>,
    pub parallel: bool,
}

/// Load a `forge-gen.toml` manifest from the given path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not valid TOML.
pub fn load(path: &Path) -> Result<Manifest> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
}

/// Merge a (possibly absent) manifest with CLI arguments.
///
/// CLI arguments override manifest values. "all" in any target list is resolved
/// later by the generator (which knows the full registry).
#[must_use]
pub fn merge_with_cli(manifest: Option<&Manifest>, cli: &CliArgs) -> GenerateConfig {
    let spec = cli
        .spec
        .clone()
        .or_else(|| manifest.and_then(|m| m.spec.as_ref().map(|s| s.path.clone())))
        .unwrap_or_default();

    let output_dir = cli
        .output
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.output
                    .as_ref()
                    .and_then(|o| o.dir.clone())
            })
        })
        .unwrap_or_else(|| String::from("./generated"));

    let sdks = parse_csv_or_manifest(
        cli.sdks.as_deref(),
        manifest.and_then(|m| m.sdks.as_ref()),
    );
    let servers = parse_csv_or_manifest(
        cli.servers.as_deref(),
        manifest.and_then(|m| m.servers.as_ref()),
    );
    let schemas = parse_csv_or_manifest(
        cli.schemas.as_deref(),
        manifest.and_then(|m| m.schemas.as_ref()),
    );
    let docs = parse_csv_or_manifest(
        cli.docs.as_deref(),
        manifest.and_then(|m| m.docs.as_ref()),
    );

    let iac_backends = parse_csv_or_iac(
        cli.iac.as_deref(),
        manifest.and_then(|m| m.iac.as_ref()),
    );

    let iac_resources = cli
        .resources
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.iac
                    .as_ref()
                    .and_then(|i| i.resources.clone())
            })
        });

    let iac_provider = cli
        .provider
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.iac
                    .as_ref()
                    .and_then(|i| i.provider.clone())
            })
        });

    GenerateConfig {
        spec,
        output_dir,
        sdks,
        servers,
        iac_backends,
        iac_resources,
        iac_provider,
        schemas,
        docs,
        parallel: cli.parallel,
    }
}

fn parse_csv_or_manifest(cli_value: Option<&str>, manifest: Option<&TargetList>) -> Vec<String> {
    if let Some(csv) = cli_value {
        csv.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(tl) = manifest {
        tl.targets.clone()
    } else {
        Vec::new()
    }
}

fn parse_csv_or_iac(cli_value: Option<&str>, manifest: Option<&IacConfig>) -> Vec<String> {
    if let Some(csv) = cli_value {
        csv.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(ic) = manifest {
        ic.backends.clone()
    } else {
        Vec::new()
    }
}
