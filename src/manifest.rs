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
    pub helm: Option<HelmConfig>,
    pub mcp: Option<McpConfig>,
    pub completions: Option<CompletionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct HelmConfig {
    pub targets: Vec<String>,
    pub resources: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    pub targets: Vec<String>,
    /// Project name for generated MCP server (defaults to spec title)
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionConfig {
    pub targets: Vec<String>,
    /// CLI command name for generated completions (defaults to spec title)
    pub name: Option<String>,
    /// Prompt icon (Unicode glyph)
    pub icon: Option<String>,
    /// Grouping strategy: auto, tag, path, or operation-id
    pub grouping: Option<String>,
    /// Command aliases (e.g., ["ps", "pet"])
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpecConfig {
    pub path: String,
    #[serde(rename = "version")]
    pub _version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OutputConfig {
    pub dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TargetList {
    pub targets: Vec<String>,
    #[serde(default, rename = "overrides")]
    pub _overrides: Option<HashMap<String, HashMap<String, String>>>,
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
    pub helm_targets: Vec<String>,
    pub helm_resources: Option<String>,
    pub helm_provider: Option<String>,
    pub mcp_targets: Vec<String>,
    pub mcp_name: Option<String>,
    pub completion_targets: Vec<String>,
    pub completion_name: Option<String>,
    pub completion_icon: Option<String>,
    pub completion_grouping: Option<String>,
    pub completion_aliases: Vec<String>,
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

    let sdks = parse_csv_or(
        cli.sdks.as_deref(),
        manifest.and_then(|m| m.sdks.as_ref()),
    );
    let servers = parse_csv_or(
        cli.servers.as_deref(),
        manifest.and_then(|m| m.servers.as_ref()),
    );
    let schemas = parse_csv_or(
        cli.schemas.as_deref(),
        manifest.and_then(|m| m.schemas.as_ref()),
    );
    let docs = parse_csv_or(
        cli.docs.as_deref(),
        manifest.and_then(|m| m.docs.as_ref()),
    );

    let mcp_targets = parse_csv_or(
        cli.mcp.as_deref(),
        manifest.and_then(|m| m.mcp.as_ref()),
    );
    let mcp_name = cli
        .mcp_name
        .clone()
        .or_else(|| manifest.and_then(|m| m.mcp.as_ref().and_then(|mc| mc.name.clone())));

    let helm_targets = parse_csv_or(
        cli.helm.as_deref(),
        manifest.and_then(|m| m.helm.as_ref()),
    );

    let helm_resources = cli
        .helm_resources
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.helm
                    .as_ref()
                    .and_then(|h| h.resources.clone())
            })
        });

    let helm_provider = cli
        .helm_provider
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.helm
                    .as_ref()
                    .and_then(|h| h.provider.clone())
            })
        });

    let iac_backends = parse_csv_or(
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

    let completion_targets = parse_csv_or(
        cli.completions.as_deref(),
        manifest.and_then(|m| m.completions.as_ref()),
    );
    let completion_name = cli
        .completion_name
        .clone()
        .or_else(|| {
            manifest.and_then(|m| {
                m.completions
                    .as_ref()
                    .and_then(|c| c.name.clone())
            })
        });
    let completion_icon = manifest.and_then(|m| {
        m.completions
            .as_ref()
            .and_then(|c| c.icon.clone())
    });
    let completion_grouping = manifest.and_then(|m| {
        m.completions
            .as_ref()
            .and_then(|c| c.grouping.clone())
    });
    let completion_aliases = manifest
        .and_then(|m| m.completions.as_ref().map(|c| c.aliases.clone()))
        .unwrap_or_default();

    GenerateConfig {
        spec,
        output_dir,
        sdks,
        servers,
        iac_backends,
        iac_resources,
        iac_provider,
        helm_targets,
        helm_resources,
        helm_provider,
        schemas,
        docs,
        mcp_targets,
        mcp_name,
        completion_targets,
        completion_name,
        completion_icon,
        completion_grouping,
        completion_aliases,
        parallel: cli.parallel,
    }
}

trait HasTargets {
    fn targets(&self) -> &[String];
}

impl HasTargets for TargetList {
    fn targets(&self) -> &[String] {
        &self.targets
    }
}

impl HasTargets for McpConfig {
    fn targets(&self) -> &[String] {
        &self.targets
    }
}

impl HasTargets for HelmConfig {
    fn targets(&self) -> &[String] {
        &self.targets
    }
}

impl HasTargets for IacConfig {
    fn targets(&self) -> &[String] {
        &self.backends
    }
}

impl HasTargets for CompletionConfig {
    fn targets(&self) -> &[String] {
        &self.targets
    }
}

fn parse_csv_or<T: HasTargets>(cli_value: Option<&str>, manifest: Option<&T>) -> Vec<String> {
    if let Some(csv) = cli_value {
        csv.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(m) = manifest {
        m.targets().to_vec()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper: build a CliArgs with all fields set to None / defaults ────────

    fn empty_cli() -> CliArgs {
        CliArgs {
            spec: None,
            output: None,
            sdks: None,
            servers: None,
            iac: None,
            schemas: None,
            docs: None,
            helm: None,
            helm_resources: None,
            helm_provider: None,
            mcp: None,
            mcp_name: None,
            completions: None,
            completion_name: None,
            resources: None,
            provider: None,
            manifest: None,
            parallel: true,
        }
    }

    // ── TOML parsing (load via toml::from_str) ──────────────────────────────

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
[spec]
path = "api.yaml"
version = "3.0"

[output]
dir = "./out"

[sdks]
targets = ["go", "python"]

[sdks.overrides.go]
packageName = "myapi"

[servers]
targets = ["rust-axum"]

[iac]
backends = ["terraform", "pulumi"]
resources = "./res"
provider = "./provider.toml"

[schemas]
targets = ["graphql-schema"]

[docs]
targets = ["markdown"]

[helm]
targets = ["helm"]
resources = "./helm-res"
provider = "./helm-provider.toml"

[mcp]
targets = ["mcp-rust"]
name = "my-mcp"

[completions]
targets = ["skim-tab", "fish"]
name = "my-cli"
icon = "☁"
grouping = "tag"
aliases = ["mc"]
"#;
        let m: Manifest = toml::from_str(toml).expect("valid TOML");

        let spec = m.spec.as_ref().unwrap();
        assert_eq!(spec.path, "api.yaml");
        assert_eq!(spec._version.as_deref(), Some("3.0"));

        assert_eq!(m.output.as_ref().unwrap().dir.as_deref(), Some("./out"));

        let sdks = m.sdks.as_ref().unwrap();
        assert_eq!(sdks.targets, vec!["go", "python"]);
        let overrides = sdks._overrides.as_ref().unwrap();
        assert_eq!(overrides["go"]["packageName"], "myapi");

        assert_eq!(
            m.servers.as_ref().unwrap().targets,
            vec!["rust-axum"]
        );

        let iac = m.iac.as_ref().unwrap();
        assert_eq!(iac.backends, vec!["terraform", "pulumi"]);
        assert_eq!(iac.resources.as_deref(), Some("./res"));
        assert_eq!(iac.provider.as_deref(), Some("./provider.toml"));

        assert_eq!(
            m.schemas.as_ref().unwrap().targets,
            vec!["graphql-schema"]
        );
        assert_eq!(m.docs.as_ref().unwrap().targets, vec!["markdown"]);

        let helm = m.helm.as_ref().unwrap();
        assert_eq!(helm.targets, vec!["helm"]);
        assert_eq!(helm.resources.as_deref(), Some("./helm-res"));
        assert_eq!(helm.provider.as_deref(), Some("./helm-provider.toml"));

        let mcp = m.mcp.as_ref().unwrap();
        assert_eq!(mcp.targets, vec!["mcp-rust"]);
        assert_eq!(mcp.name.as_deref(), Some("my-mcp"));

        let comp = m.completions.as_ref().unwrap();
        assert_eq!(comp.targets, vec!["skim-tab", "fish"]);
        assert_eq!(comp.name.as_deref(), Some("my-cli"));
        assert_eq!(comp.icon.as_deref(), Some("☁"));
        assert_eq!(comp.grouping.as_deref(), Some("tag"));
        assert_eq!(comp.aliases, vec!["mc"]);
    }

    #[test]
    fn parse_minimal_manifest() {
        let toml = r#"
[spec]
path = "spec.json"
"#;
        let m: Manifest = toml::from_str(toml).expect("valid TOML");
        assert_eq!(m.spec.as_ref().unwrap().path, "spec.json");
        assert!(m.spec.as_ref().unwrap()._version.is_none());
        assert!(m.sdks.is_none());
        assert!(m.servers.is_none());
        assert!(m.iac.is_none());
        assert!(m.schemas.is_none());
        assert!(m.docs.is_none());
        assert!(m.helm.is_none());
        assert!(m.mcp.is_none());
    }

    #[test]
    fn parse_empty_manifest() {
        let m: Manifest = toml::from_str("").expect("empty TOML is valid");
        assert!(m.spec.is_none());
        assert!(m.output.is_none());
        assert!(m.sdks.is_none());
    }

    #[test]
    fn parse_target_list_without_overrides() {
        let toml = r#"
[sdks]
targets = ["go"]
"#;
        let m: Manifest = toml::from_str(toml).expect("valid TOML");
        let sdks = m.sdks.as_ref().unwrap();
        assert_eq!(sdks.targets, vec!["go"]);
        assert!(sdks._overrides.is_none());
    }

    #[test]
    fn parse_target_list_with_empty_overrides() {
        let toml = r#"
[sdks]
targets = ["go"]

[sdks.overrides]
"#;
        let m: Manifest = toml::from_str(toml).expect("valid TOML");
        let sdks = m.sdks.as_ref().unwrap();
        let overrides = sdks._overrides.as_ref().unwrap();
        assert!(overrides.is_empty());
    }

    // ── load() with a real temp file ────────────────────────────────────────

    #[test]
    fn load_valid_file() {
        let dir = std::env::temp_dir().join("forge_gen_test_load");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("forge-gen.toml");
        std::fs::write(
            &path,
            r#"
[spec]
path = "my-api.yaml"

[sdks]
targets = ["rust"]
"#,
        )
        .unwrap();

        let m = load(&path).expect("should load");
        assert_eq!(m.spec.as_ref().unwrap().path, "my-api.yaml");
        assert_eq!(m.sdks.as_ref().unwrap().targets, vec!["rust"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_file_returns_error() {
        let result = load(Path::new("/tmp/forge_gen_test_nonexistent.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let dir = std::env::temp_dir().join("forge_gen_test_invalid");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad.toml");
        std::fs::write(&path, "[spec\npath = broken").unwrap();

        let result = load(&path);
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── merge_with_cli: CLI overrides manifest ──────────────────────────────

    #[test]
    fn merge_cli_overrides_spec() {
        let manifest: Manifest = toml::from_str(
            r#"
[spec]
path = "manifest-spec.yaml"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.spec = Some(String::from("cli-spec.yaml"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.spec, "cli-spec.yaml");
    }

    #[test]
    fn merge_cli_overrides_output_dir() {
        let manifest: Manifest = toml::from_str(
            r#"
[output]
dir = "./manifest-out"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.output = Some(String::from("./cli-out"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.output_dir, "./cli-out");
    }

    #[test]
    fn merge_cli_overrides_sdks() {
        let manifest: Manifest = toml::from_str(
            r#"
[sdks]
targets = ["go", "python"]
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.sdks = Some(String::from("rust,java"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.sdks, vec!["rust", "java"]);
    }

    #[test]
    fn merge_cli_overrides_iac() {
        let manifest: Manifest = toml::from_str(
            r#"
[iac]
backends = ["terraform"]
resources = "./manifest-res"
provider = "./manifest-prov.toml"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.iac = Some(String::from("pulumi"));
        cli.resources = Some(String::from("./cli-res"));
        cli.provider = Some(String::from("./cli-prov.toml"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.iac_backends, vec!["pulumi"]);
        assert_eq!(config.iac_resources.as_deref(), Some("./cli-res"));
        assert_eq!(config.iac_provider.as_deref(), Some("./cli-prov.toml"));
    }

    #[test]
    fn merge_cli_overrides_helm() {
        let manifest: Manifest = toml::from_str(
            r#"
[helm]
targets = ["helm"]
resources = "./manifest-helm-res"
provider = "./manifest-helm-prov.toml"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.helm = Some(String::from("helm"));
        cli.helm_resources = Some(String::from("./cli-helm-res"));
        cli.helm_provider = Some(String::from("./cli-helm-prov.toml"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.helm_targets, vec!["helm"]);
        assert_eq!(config.helm_resources.as_deref(), Some("./cli-helm-res"));
        assert_eq!(
            config.helm_provider.as_deref(),
            Some("./cli-helm-prov.toml")
        );
    }

    #[test]
    fn merge_cli_overrides_mcp() {
        let manifest: Manifest = toml::from_str(
            r#"
[mcp]
targets = ["mcp-rust"]
name = "manifest-name"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.mcp = Some(String::from("mcp-rust"));
        cli.mcp_name = Some(String::from("cli-name"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.mcp_targets, vec!["mcp-rust"]);
        assert_eq!(config.mcp_name.as_deref(), Some("cli-name"));
    }

    // ── merge_with_cli: manifest values used when CLI absent ────────────────

    #[test]
    fn merge_manifest_used_when_cli_absent() {
        let manifest: Manifest = toml::from_str(
            r#"
[spec]
path = "manifest.yaml"

[output]
dir = "./manifest-out"

[sdks]
targets = ["go"]

[servers]
targets = ["rust-axum"]

[schemas]
targets = ["protobuf-schema"]

[docs]
targets = ["html"]

[iac]
backends = ["ansible"]
resources = "./iac-res"
provider = "./iac-prov.toml"

[helm]
targets = ["helm"]
resources = "./helm-res"
provider = "./helm-prov.toml"

[mcp]
targets = ["mcp-rust"]
name = "my-server"

[completions]
targets = ["skim-tab"]
name = "my-tool"
icon = "☁"
grouping = "tag"
"#,
        )
        .unwrap();

        let cli = empty_cli();
        let config = merge_with_cli(Some(&manifest), &cli);

        assert_eq!(config.spec, "manifest.yaml");
        assert_eq!(config.output_dir, "./manifest-out");
        assert_eq!(config.sdks, vec!["go"]);
        assert_eq!(config.servers, vec!["rust-axum"]);
        assert_eq!(config.schemas, vec!["protobuf-schema"]);
        assert_eq!(config.docs, vec!["html"]);
        assert_eq!(config.iac_backends, vec!["ansible"]);
        assert_eq!(config.iac_resources.as_deref(), Some("./iac-res"));
        assert_eq!(config.iac_provider.as_deref(), Some("./iac-prov.toml"));
        assert_eq!(config.helm_targets, vec!["helm"]);
        assert_eq!(config.helm_resources.as_deref(), Some("./helm-res"));
        assert_eq!(config.helm_provider.as_deref(), Some("./helm-prov.toml"));
        assert_eq!(config.mcp_targets, vec!["mcp-rust"]);
        assert_eq!(config.mcp_name.as_deref(), Some("my-server"));
        assert_eq!(config.completion_targets, vec!["skim-tab"]);
        assert_eq!(config.completion_name.as_deref(), Some("my-tool"));
        assert_eq!(config.completion_icon.as_deref(), Some("☁"));
        assert_eq!(config.completion_grouping.as_deref(), Some("tag"));
    }

    // ── merge_with_cli: no manifest ─────────────────────────────────────────

    #[test]
    fn merge_no_manifest_uses_defaults() {
        let cli = empty_cli();
        let config = merge_with_cli(None, &cli);

        assert_eq!(config.spec, "");
        assert_eq!(config.output_dir, "./generated");
        assert!(config.sdks.is_empty());
        assert!(config.servers.is_empty());
        assert!(config.iac_backends.is_empty());
        assert!(config.iac_resources.is_none());
        assert!(config.iac_provider.is_none());
        assert!(config.schemas.is_empty());
        assert!(config.docs.is_empty());
        assert!(config.helm_targets.is_empty());
        assert!(config.helm_resources.is_none());
        assert!(config.helm_provider.is_none());
        assert!(config.mcp_targets.is_empty());
        assert!(config.mcp_name.is_none());
        assert!(config.completion_targets.is_empty());
        assert!(config.completion_name.is_none());
        assert!(config.parallel);
    }

    #[test]
    fn merge_no_manifest_cli_only() {
        let mut cli = empty_cli();
        cli.spec = Some(String::from("cli.yaml"));
        cli.output = Some(String::from("./cli-out"));
        cli.sdks = Some(String::from("go,python"));
        cli.servers = Some(String::from("rust-axum"));
        cli.iac = Some(String::from("terraform"));
        cli.resources = Some(String::from("./r"));
        cli.provider = Some(String::from("./p.toml"));
        cli.schemas = Some(String::from("graphql-schema"));
        cli.docs = Some(String::from("markdown,html"));
        cli.helm = Some(String::from("helm"));
        cli.helm_resources = Some(String::from("./hr"));
        cli.helm_provider = Some(String::from("./hp.toml"));
        cli.mcp = Some(String::from("mcp-rust"));
        cli.mcp_name = Some(String::from("my-name"));
        cli.completions = Some(String::from("skim-tab,fish"));
        cli.completion_name = Some(String::from("my-tool"));
        cli.parallel = false;

        let config = merge_with_cli(None, &cli);

        assert_eq!(config.spec, "cli.yaml");
        assert_eq!(config.output_dir, "./cli-out");
        assert_eq!(config.sdks, vec!["go", "python"]);
        assert_eq!(config.servers, vec!["rust-axum"]);
        assert_eq!(config.iac_backends, vec!["terraform"]);
        assert_eq!(config.iac_resources.as_deref(), Some("./r"));
        assert_eq!(config.iac_provider.as_deref(), Some("./p.toml"));
        assert_eq!(config.schemas, vec!["graphql-schema"]);
        assert_eq!(config.docs, vec!["markdown", "html"]);
        assert_eq!(config.helm_targets, vec!["helm"]);
        assert_eq!(config.helm_resources.as_deref(), Some("./hr"));
        assert_eq!(config.helm_provider.as_deref(), Some("./hp.toml"));
        assert_eq!(config.mcp_targets, vec!["mcp-rust"]);
        assert_eq!(config.mcp_name.as_deref(), Some("my-name"));
        assert_eq!(config.completion_targets, vec!["skim-tab", "fish"]);
        assert_eq!(config.completion_name.as_deref(), Some("my-tool"));
        assert!(!config.parallel);
    }

    // ── merge_with_cli: empty manifest sections ─────────────────────────────

    #[test]
    fn merge_empty_manifest_uses_defaults() {
        let manifest = Manifest::default();
        let cli = empty_cli();
        let config = merge_with_cli(Some(&manifest), &cli);

        assert_eq!(config.spec, "");
        assert_eq!(config.output_dir, "./generated");
        assert!(config.sdks.is_empty());
        assert!(config.servers.is_empty());
        assert!(config.iac_backends.is_empty());
        assert!(config.schemas.is_empty());
        assert!(config.docs.is_empty());
        assert!(config.helm_targets.is_empty());
        assert!(config.mcp_targets.is_empty());
        assert!(config.completion_targets.is_empty());
    }

    // ── parse_csv functions ─────────────────────────────────────────────────

    #[test]
    fn parse_csv_or_target_list_with_csv() {
        let result = parse_csv_or::<TargetList>(Some("go, python , rust"), None);
        assert_eq!(result, vec!["go", "python", "rust"]);
    }

    #[test]
    fn parse_csv_or_target_list_with_single_value() {
        let result = parse_csv_or::<TargetList>(Some("go"), None);
        assert_eq!(result, vec!["go"]);
    }

    #[test]
    fn parse_csv_or_target_list_empty_csv() {
        let result = parse_csv_or::<TargetList>(Some(""), None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_or_target_list_csv_with_trailing_comma() {
        let result = parse_csv_or::<TargetList>(Some("go,python,"), None);
        assert_eq!(result, vec!["go", "python"]);
    }

    #[test]
    fn parse_csv_or_target_list_csv_overrides_manifest() {
        let tl = TargetList {
            targets: vec![String::from("java")],
            _overrides: None,
        };
        let result = parse_csv_or(Some("go"), Some(&tl));
        assert_eq!(result, vec!["go"]);
    }

    #[test]
    fn parse_csv_or_target_list_uses_manifest_fallback() {
        let tl = TargetList {
            targets: vec![String::from("java"), String::from("kotlin")],
            _overrides: None,
        };
        let result = parse_csv_or(None, Some(&tl));
        assert_eq!(result, vec!["java", "kotlin"]);
    }

    #[test]
    fn parse_csv_or_target_list_both_none() {
        let result = parse_csv_or::<TargetList>(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_or_mcp_with_csv() {
        let result = parse_csv_or::<McpConfig>(Some("mcp-rust"), None);
        assert_eq!(result, vec!["mcp-rust"]);
    }

    #[test]
    fn parse_csv_or_mcp_uses_manifest_fallback() {
        let mc = McpConfig {
            targets: vec![String::from("mcp-rust")],
            name: Some(String::from("test")),
        };
        let result = parse_csv_or(None, Some(&mc));
        assert_eq!(result, vec!["mcp-rust"]);
    }

    #[test]
    fn parse_csv_or_mcp_both_none() {
        let result = parse_csv_or::<McpConfig>(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_or_helm_with_csv() {
        let result = parse_csv_or::<HelmConfig>(Some("helm"), None);
        assert_eq!(result, vec!["helm"]);
    }

    #[test]
    fn parse_csv_or_helm_uses_manifest_fallback() {
        let hc = HelmConfig {
            targets: vec![String::from("helm")],
            resources: None,
            provider: None,
        };
        let result = parse_csv_or(None, Some(&hc));
        assert_eq!(result, vec!["helm"]);
    }

    #[test]
    fn parse_csv_or_helm_both_none() {
        let result = parse_csv_or::<HelmConfig>(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_or_iac_with_csv() {
        let result = parse_csv_or::<IacConfig>(Some("terraform,pulumi"), None);
        assert_eq!(result, vec!["terraform", "pulumi"]);
    }

    #[test]
    fn parse_csv_or_iac_uses_manifest_fallback() {
        let ic = IacConfig {
            backends: vec![String::from("crossplane")],
            resources: None,
            provider: None,
        };
        let result = parse_csv_or(None, Some(&ic));
        assert_eq!(result, vec!["crossplane"]);
    }

    #[test]
    fn parse_csv_or_iac_both_none() {
        let result = parse_csv_or::<IacConfig>(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_or_completion_with_csv() {
        let result = parse_csv_or::<CompletionConfig>(Some("skim-tab,fish"), None);
        assert_eq!(result, vec!["skim-tab", "fish"]);
    }

    #[test]
    fn parse_csv_or_completion_uses_manifest_fallback() {
        let cc = CompletionConfig {
            targets: vec![String::from("skim-tab")],
            name: Some(String::from("test")),
            icon: None,
            grouping: None,
            aliases: vec![],
        };
        let result = parse_csv_or(None, Some(&cc));
        assert_eq!(result, vec!["skim-tab"]);
    }

    #[test]
    fn parse_csv_or_completion_both_none() {
        let result = parse_csv_or::<CompletionConfig>(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn merge_cli_overrides_completions() {
        let manifest: Manifest = toml::from_str(
            r#"
[completions]
targets = ["skim-tab"]
name = "manifest-name"
"#,
        )
        .unwrap();

        let mut cli = empty_cli();
        cli.completions = Some(String::from("fish"));
        cli.completion_name = Some(String::from("cli-name"));

        let config = merge_with_cli(Some(&manifest), &cli);
        assert_eq!(config.completion_targets, vec!["fish"]);
        assert_eq!(config.completion_name.as_deref(), Some("cli-name"));
    }

    // ── Edge cases ──────────────────────────────────────────────────────────

    #[test]
    fn merge_parallel_defaults_to_true() {
        let cli = empty_cli();
        let config = merge_with_cli(None, &cli);
        assert!(config.parallel);
    }

    #[test]
    fn merge_parallel_can_be_false() {
        let mut cli = empty_cli();
        cli.parallel = false;
        let config = merge_with_cli(None, &cli);
        assert!(!config.parallel);
    }

    #[test]
    fn parse_csv_whitespace_only() {
        let result = parse_csv_or::<TargetList>(Some("  ,  , "), None);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_csv_preserves_case() {
        let result = parse_csv_or::<TargetList>(Some("Go,PYTHON,Rust"), None);
        assert_eq!(result, vec!["Go", "PYTHON", "Rust"]);
    }

    #[test]
    fn merge_output_dir_default() {
        let cli = empty_cli();
        let config = merge_with_cli(None, &cli);
        assert_eq!(config.output_dir, "./generated");
    }

    #[test]
    fn merge_spec_default_is_empty_string() {
        let cli = empty_cli();
        let config = merge_with_cli(None, &cli);
        assert_eq!(config.spec, "");
    }
}
