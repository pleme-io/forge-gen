use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use colored::Colorize;
use tokio::process::Command;
use tokio::task::JoinSet;

use crate::manifest::{self, GenerateConfig};
use crate::registry::{self, Category};

// ── TaskRunner trait ─────────────────────────────────────────────────────────

/// Common interface for all generator task types, allowing a single generic
/// `run_task` function to replace the five per-category runner functions.
trait TaskRunner: Send + 'static {
    /// Human-readable target name (e.g. "go", "terraform", "skim-tab").
    fn name(&self) -> &str;
    /// Category label for display (e.g. "sdk", "iac", "helm", "mcp", "completion").
    fn category(&self) -> &'static str;
    /// Directory where output is written.
    fn output_dir(&self) -> &str;
    /// Name of the binary to invoke (e.g. "openapi-generator-cli", "iac-forge").
    fn binary_name(&self) -> &'static str;
    /// Build the full argument list for the subprocess.
    fn build_args(&self) -> Vec<String>;
}

/// Execute any task that implements [`TaskRunner`] as an async subprocess.
/// Core task execution: resolve binary, build args, spawn subprocess.
///
/// Accepts owned strings so this future is `Send` (no references held across await).
async fn execute_task(
    name: String,
    category: String,
    output_dir: String,
    binary_name: String,
    args: Vec<String>,
) -> Result<TaskResult> {
    println!(
        "  {} [{category}/{name}] {binary_name} {name}",
        "->".green(),
    );

    std::fs::create_dir_all(&output_dir)?;

    let bin = which::which(&binary_name).ok();

    let success = if let Some(bin) = bin {
        let mut cmd = Command::new(bin);
        for arg in &args {
            cmd.arg(arg);
        }
        let status = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .with_context(|| format!("spawning {binary_name} for {name}"))?;
        status.success()
    } else {
        tracing::warn!(target = name, "{binary_name} not found in PATH — skipping");
        false
    };

    Ok(TaskResult { name, category, output_dir, success })
}

/// Execute a task from a boxed trait object (used by both parallel and sequential paths).
async fn run_task(task: Box<dyn TaskRunner>) -> Result<TaskResult> {
    execute_task(
        task.name().to_string(),
        task.category().to_string(),
        task.output_dir().to_string(),
        task.binary_name().to_string(),
        task.build_args(),
    )
    .await
}

/// CLI arguments for the `generate` subcommand.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to an `OpenAPI` spec (YAML or JSON)
    #[arg(long)]
    pub spec: Option<String>,

    /// Output directory (default: ./generated)
    #[arg(long)]
    pub output: Option<String>,

    /// Comma-separated SDK languages or "all"
    #[arg(long)]
    pub sdks: Option<String>,

    /// Comma-separated server stubs or "all"
    #[arg(long)]
    pub servers: Option<String>,

    /// Comma-separated `IaC` backends or "all"
    #[arg(long)]
    pub iac: Option<String>,

    /// Comma-separated schema types or "all"
    #[arg(long)]
    pub schemas: Option<String>,

    /// Comma-separated doc formats or "all"
    #[arg(long)]
    pub docs: Option<String>,

    /// Comma-separated Helm chart targets or "all" (via iac-forge --backend helm)
    #[arg(long)]
    pub helm: Option<String>,

    /// TOML resource specs directory (for Helm generation)
    #[arg(long)]
    pub helm_resources: Option<String>,

    /// Path to provider.toml (for Helm generation)
    #[arg(long)]
    pub helm_provider: Option<String>,

    /// Comma-separated MCP server targets or "all" (via mcp-forge)
    #[arg(long)]
    pub mcp: Option<String>,

    /// Project name for MCP server generation
    #[arg(long)]
    pub mcp_name: Option<String>,

    /// Comma-separated completion formats or "all" (via completion-forge)
    #[arg(long)]
    pub completions: Option<String>,

    /// CLI command name for completion generation
    #[arg(long)]
    pub completion_name: Option<String>,

    /// TOML resource specs directory (for `IaC` generation)
    #[arg(long)]
    pub resources: Option<String>,

    /// Path to `provider.toml` (for `IaC` generation)
    #[arg(long)]
    pub provider: Option<String>,

    /// Path to forge-gen.toml manifest (default: ./forge-gen.toml)
    #[arg(long)]
    pub manifest: Option<String>,

    /// Run generators in parallel
    #[arg(long, default_value = "true")]
    pub parallel: bool,
}

/// Run the generate command.
///
/// # Errors
///
/// Returns an error if spec loading, tool resolution, or generation fails.
pub async fn run(args: Args) -> Result<()> {
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| String::from("./forge-gen.toml"));

    let loaded = if Path::new(&manifest_path).exists() {
        tracing::info!(path = %manifest_path, "loading manifest");
        Some(manifest::load(Path::new(&manifest_path))?)
    } else {
        None
    };

    let config = manifest::merge_with_cli(loaded.as_ref(), &args);

    if config.spec.is_empty() {
        bail!("no OpenAPI spec provided — use --spec or set spec.path in forge-gen.toml");
    }

    if !Path::new(&config.spec).exists() {
        bail!("spec file not found: {}", config.spec);
    }

    // Resolve "all" targets into the full set of names per category.
    let sdks = resolve_targets(&config.sdks, Category::Sdk);
    let servers = resolve_targets(&config.servers, Category::Server);
    let schemas = resolve_targets(&config.schemas, Category::Schema);
    let docs = resolve_targets(&config.docs, Category::Doc);
    let iac_backends = resolve_targets(&config.iac_backends, Category::Iac);
    let helm_targets = resolve_targets(&config.helm_targets, Category::Helm);
    let mcp_targets = resolve_targets(&config.mcp_targets, Category::Mcp);
    let completion_targets = resolve_targets(&config.completion_targets, Category::Completion);

    let total =
        sdks.len() + servers.len() + schemas.len() + docs.len() + iac_backends.len() + helm_targets.len() + mcp_targets.len() + completion_targets.len();

    if total == 0 {
        bail!(
            "nothing to generate — specify at least one of --sdks, --servers, --iac, --helm, --schemas, --docs, --mcp, --completions"
        );
    }

    println!(
        "\n{} forge-gen: generating {} target(s) from {}",
        "=>".blue().bold(),
        total,
        config.spec,
    );

    std::fs::create_dir_all(&config.output_dir)
        .with_context(|| format!("creating output directory {}", config.output_dir))?;

    let started = Instant::now();
    let mut results: Vec<TaskResult> = Vec::new();

    // Build all tasks once — shared between parallel and sequential paths.
    let mut tasks: Vec<Box<dyn TaskRunner>> = Vec::with_capacity(total);
    for name in &sdks { tasks.push(Box::new(build_openapi_task(name, "sdk", &config))); }
    for name in &servers { tasks.push(Box::new(build_openapi_task(name, "server", &config))); }
    for name in &schemas { tasks.push(Box::new(build_openapi_task(name, "schema", &config))); }
    for name in &docs { tasks.push(Box::new(build_openapi_task(name, "doc", &config))); }
    for name in &iac_backends { tasks.push(Box::new(build_iac_task(name, &config))); }
    for name in &helm_targets { tasks.push(Box::new(build_helm_task(name, &config))); }
    for name in &mcp_targets { tasks.push(Box::new(build_mcp_task(name, &config))); }
    for name in &completion_targets { tasks.push(Box::new(build_completion_task(name, &config))); }

    if config.parallel {
        let mut set = JoinSet::new();
        for task in tasks {
            set.spawn(run_task(task));
        }
        while let Some(res) = set.join_next().await {
            results.push(res.context("task panicked")??);
        }
    } else {
        for task in tasks {
            results.push(run_task(task).await?);
        }
    }

    let elapsed = started.elapsed();

    // ── Summary table ────────────────────────────────────────────────
    println!("\n{}", "  Generator Results".bold());
    println!(
        "  {:<24} {:<10} {:<8} Output",
        "Target", "Category", "Status"
    );
    println!("  {}", "-".repeat(70));

    let mut ok_count = 0usize;
    let mut fail_count = 0usize;

    for r in &results {
        let status = if r.success {
            ok_count += 1;
            "ok".green().to_string()
        } else {
            fail_count += 1;
            "FAIL".red().bold().to_string()
        };
        println!(
            "  {:<24} {:<10} {:<8} {}",
            r.name, r.category, status, r.output_dir
        );
    }

    println!("  {}", "-".repeat(70));
    println!(
        "  {} ok, {} failed — {:.1}s\n",
        ok_count,
        fail_count,
        elapsed.as_secs_f64()
    );

    if fail_count > 0 {
        bail!("{fail_count} generator(s) failed");
    }

    Ok(())
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Resolve a list of target names: if the list contains "all", expand to every
/// name in the given category.
fn resolve_targets(targets: &[String], category: Category) -> Vec<String> {
    if targets.is_empty() {
        return Vec::new();
    }
    if targets.iter().any(|t| t.eq_ignore_ascii_case("all")) {
        return registry::names_for_category(category)
            .into_iter()
            .map(String::from)
            .collect();
    }
    targets.to_vec()
}

/// Append `--flag value` to `args` if `value` is `Some`.
fn push_optional(args: &mut Vec<String>, flag: &str, value: Option<&String>) {
    if let Some(v) = value {
        args.push(String::from(flag));
        args.push(v.clone());
    }
}

/// Result of a single generator invocation.
struct TaskResult {
    name: String,
    category: String,
    output_dir: String,
    success: bool,
}

/// Describes an openapi-generator-cli invocation.
struct OpenApiTask {
    name: String,
    generator: String,
    category: &'static str,
    spec: String,
    output_dir: String,
}

impl TaskRunner for OpenApiTask {
    fn name(&self) -> &str { &self.name }
    fn category(&self) -> &'static str { self.category }
    fn output_dir(&self) -> &str { &self.output_dir }
    fn binary_name(&self) -> &'static str { "openapi-generator-cli" }

    fn build_args(&self) -> Vec<String> {
        vec![
            String::from("generate"),
            String::from("-i"),
            self.spec.clone(),
            String::from("-g"),
            self.generator.clone(),
            String::from("-o"),
            self.output_dir.clone(),
        ]
    }
}

fn build_openapi_task(name: &str, category: &'static str, config: &GenerateConfig) -> OpenApiTask {
    let info = registry::find(name);
    let generator = info.map_or_else(|| name.to_string(), |i| i.generator.to_string());
    let out = format!("{}/{category}/{name}", config.output_dir);

    OpenApiTask {
        name: name.to_string(),
        generator,
        category,
        spec: config.spec.clone(),
        output_dir: out,
    }
}

/// Describes an `iac-forge` invocation (used for both `IaC` and Helm categories).
struct IacForgeTask {
    name: String,
    backend: String,
    category: &'static str,
    spec: String,
    output_dir: String,
    resources: Option<String>,
    provider: Option<String>,
}

impl TaskRunner for IacForgeTask {
    fn name(&self) -> &str { &self.name }
    fn category(&self) -> &'static str { self.category }
    fn output_dir(&self) -> &str { &self.output_dir }
    fn binary_name(&self) -> &'static str { "iac-forge" }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from("--backend"),
            self.backend.clone(),
            String::from("--spec"),
            self.spec.clone(),
            String::from("--output"),
            self.output_dir.clone(),
        ];
        push_optional(&mut args, "--resources", self.resources.as_ref());
        push_optional(&mut args, "--provider", self.provider.as_ref());
        args
    }
}

fn build_iac_task(name: &str, config: &GenerateConfig) -> IacForgeTask {
    let out = format!("{}/iac/{name}", config.output_dir);

    IacForgeTask {
        name: name.to_string(),
        backend: name.to_string(),
        category: "iac",
        spec: config.spec.clone(),
        output_dir: out,
        resources: config.iac_resources.clone(),
        provider: config.iac_provider.clone(),
    }
}

fn build_helm_task(name: &str, config: &GenerateConfig) -> IacForgeTask {
    let out = format!("{}/helm/{name}", config.output_dir);

    IacForgeTask {
        name: name.to_string(),
        backend: String::from("helm"),
        category: "helm",
        spec: config.spec.clone(),
        output_dir: out,
        resources: config.helm_resources.clone().or_else(|| config.iac_resources.clone()),
        provider: config.helm_provider.clone().or_else(|| config.iac_provider.clone()),
    }
}

/// Describes an mcp-forge invocation.
struct McpTask {
    name: String,
    spec: String,
    output_dir: String,
    project_name: Option<String>,
}

impl TaskRunner for McpTask {
    fn name(&self) -> &str { &self.name }
    fn category(&self) -> &'static str { "mcp" }
    fn output_dir(&self) -> &str { &self.output_dir }
    fn binary_name(&self) -> &'static str { "mcp-forge" }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from("--spec"),
            self.spec.clone(),
            String::from("--output"),
            self.output_dir.clone(),
        ];
        push_optional(&mut args, "--name", self.project_name.as_ref());
        args
    }
}

fn build_mcp_task(name: &str, config: &GenerateConfig) -> McpTask {
    let out = format!("{}/mcp/{name}", config.output_dir);

    McpTask {
        name: name.to_string(),
        spec: config.spec.clone(),
        output_dir: out,
        project_name: config.mcp_name.clone(),
    }
}

/// Describes a completion-forge invocation.
struct CompletionTask {
    name: String,
    format: String,
    spec: String,
    output_dir: String,
    project_name: Option<String>,
    icon: Option<String>,
    grouping: Option<String>,
    aliases: Vec<String>,
}

impl TaskRunner for CompletionTask {
    fn name(&self) -> &str { &self.name }
    fn category(&self) -> &'static str { "completion" }
    fn output_dir(&self) -> &str { &self.output_dir }
    fn binary_name(&self) -> &'static str { "completion-forge" }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            String::from("generate"),
            String::from("--spec"),
            self.spec.clone(),
            String::from("--output"),
            self.output_dir.clone(),
            String::from("--format"),
            self.format.clone(),
        ];
        push_optional(&mut args, "--name", self.project_name.as_ref());
        push_optional(&mut args, "--icon", self.icon.as_ref());
        push_optional(&mut args, "--grouping", self.grouping.as_ref());
        if !self.aliases.is_empty() {
            args.push(String::from("--aliases"));
            args.push(self.aliases.join(","));
        }
        args
    }
}

fn build_completion_task(name: &str, config: &GenerateConfig) -> CompletionTask {
    let out = format!("{}/completion/{name}", config.output_dir);

    CompletionTask {
        name: name.to_string(),
        format: name.to_string(),
        spec: config.spec.clone(),
        output_dir: out,
        project_name: config.completion_name.clone(),
        icon: config.completion_icon.clone(),
        grouping: config.completion_grouping.clone(),
        aliases: config.completion_aliases.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_targets_empty_returns_empty() {
        assert!(resolve_targets(&[], Category::Sdk).is_empty());
    }

    #[test]
    fn resolve_targets_all_expands() {
        let result = resolve_targets(&["all".to_string()], Category::Completion);
        assert!(result.contains(&"skim-tab".to_string()));
        assert!(result.contains(&"fish".to_string()));
    }

    #[test]
    fn resolve_targets_specific_passes_through() {
        let input = vec!["go".to_string(), "python".to_string()];
        let result = resolve_targets(&input, Category::Sdk);
        assert_eq!(result, input);
    }

    #[test]
    fn build_openapi_task_sets_correct_fields() {
        let config = GenerateConfig {
            spec: "api.yaml".to_string(),
            output_dir: "./out".to_string(),
            sdks: vec![],
            servers: vec![],
            iac_backends: vec![],
            iac_resources: None,
            iac_provider: None,
            schemas: vec![],
            docs: vec![],
            helm_targets: vec![],
            helm_resources: None,
            helm_provider: None,
            mcp_targets: vec![],
            mcp_name: None,
            completion_targets: vec![],
            completion_name: None,
            completion_icon: None,
            completion_grouping: None,
            completion_aliases: vec![],
            parallel: true,
        };
        let task = build_openapi_task("go", "sdk", &config);
        assert_eq!(task.name, "go");
        assert_eq!(task.category, "sdk");
        assert_eq!(task.spec, "api.yaml");
        assert!(task.output_dir.ends_with("sdk/go"));
    }

    #[test]
    fn build_completion_task_sets_correct_fields() {
        let config = GenerateConfig {
            spec: "api.yaml".to_string(),
            output_dir: "./out".to_string(),
            sdks: vec![],
            servers: vec![],
            iac_backends: vec![],
            iac_resources: None,
            iac_provider: None,
            schemas: vec![],
            docs: vec![],
            helm_targets: vec![],
            helm_resources: None,
            helm_provider: None,
            mcp_targets: vec![],
            mcp_name: None,
            completion_targets: vec![],
            completion_name: Some("my-tool".to_string()),
            completion_icon: Some("\u{2601}".to_string()),
            completion_grouping: Some("tag".to_string()),
            completion_aliases: vec!["mt".to_string()],
            parallel: true,
        };
        let task = build_completion_task("skim-tab", &config);
        assert_eq!(task.name, "skim-tab");
        assert_eq!(task.format, "skim-tab");
        assert_eq!(task.project_name.as_deref(), Some("my-tool"));
        assert_eq!(task.icon.as_deref(), Some("\u{2601}"));
        assert_eq!(task.aliases, vec!["mt"]);
    }

    // ── TaskRunner trait tests ─────────────────────────────────────────────

    #[test]
    fn openapi_task_runner_fields() {
        let task = OpenApiTask {
            name: "go".to_string(),
            generator: "go".to_string(),
            category: "sdk",
            spec: "api.yaml".to_string(),
            output_dir: "./out/sdk/go".to_string(),
        };
        assert_eq!(task.name(), "go");
        assert_eq!(task.category(), "sdk");
        assert_eq!(task.output_dir(), "./out/sdk/go");
        assert_eq!(task.binary_name(), "openapi-generator-cli");
    }

    #[test]
    fn openapi_task_build_args() {
        let task = OpenApiTask {
            name: "python".to_string(),
            generator: "python".to_string(),
            category: "sdk",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/sdk/python".to_string(),
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec!["generate", "-i", "spec.yaml", "-g", "python", "-o", "./out/sdk/python"]
        );
    }

    #[test]
    fn iac_task_runner_fields() {
        let task = IacForgeTask {
            name: "terraform".to_string(),
            backend: "terraform".to_string(),
            category: "iac",
            spec: "api.yaml".to_string(),
            output_dir: "./out/iac/terraform".to_string(),
            resources: None,
            provider: None,
        };
        assert_eq!(task.name(), "terraform");
        assert_eq!(task.category(), "iac");
        assert_eq!(task.binary_name(), "iac-forge");
    }

    #[test]
    fn iac_task_build_args_minimal() {
        let task = IacForgeTask {
            name: "pulumi".to_string(),
            backend: "pulumi".to_string(),
            category: "iac",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/iac/pulumi".to_string(),
            resources: None,
            provider: None,
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec!["generate", "--backend", "pulumi", "--spec", "spec.yaml", "--output", "./out/iac/pulumi"]
        );
    }

    #[test]
    fn iac_task_build_args_with_resources_and_provider() {
        let task = IacForgeTask {
            name: "terraform".to_string(),
            backend: "terraform".to_string(),
            category: "iac",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/iac/terraform".to_string(),
            resources: Some("./res".to_string()),
            provider: Some("./prov.toml".to_string()),
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec![
                "generate", "--backend", "terraform", "--spec", "spec.yaml",
                "--output", "./out/iac/terraform",
                "--resources", "./res",
                "--provider", "./prov.toml",
            ]
        );
    }

    #[test]
    fn helm_task_runner_fields() {
        let task = IacForgeTask {
            name: "helm".to_string(),
            backend: "helm".to_string(),
            category: "helm",
            spec: "api.yaml".to_string(),
            output_dir: "./out/helm/helm".to_string(),
            resources: None,
            provider: None,
        };
        assert_eq!(task.name(), "helm");
        assert_eq!(task.category(), "helm");
        assert_eq!(task.binary_name(), "iac-forge");
    }

    #[test]
    fn helm_task_build_args_uses_helm_backend() {
        let task = IacForgeTask {
            name: "helm".to_string(),
            backend: "helm".to_string(),
            category: "helm",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/helm/helm".to_string(),
            resources: Some("./r".to_string()),
            provider: None,
        };
        let args = task.build_args();
        assert!(args.contains(&String::from("helm")));
        assert_eq!(args[2], "helm"); // --backend helm
        assert!(args.contains(&String::from("--resources")));
    }

    #[test]
    fn mcp_task_runner_fields() {
        let task = McpTask {
            name: "mcp-rust".to_string(),
            spec: "api.yaml".to_string(),
            output_dir: "./out/mcp/mcp-rust".to_string(),
            project_name: None,
        };
        assert_eq!(task.name(), "mcp-rust");
        assert_eq!(task.category(), "mcp");
        assert_eq!(task.binary_name(), "mcp-forge");
    }

    #[test]
    fn mcp_task_build_args_with_name() {
        let task = McpTask {
            name: "mcp-rust".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/mcp/mcp-rust".to_string(),
            project_name: Some("my-api".to_string()),
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec!["generate", "--spec", "spec.yaml", "--output", "./out/mcp/mcp-rust", "--name", "my-api"]
        );
    }

    #[test]
    fn completion_task_runner_fields() {
        let task = CompletionTask {
            name: "fish".to_string(),
            format: "fish".to_string(),
            spec: "api.yaml".to_string(),
            output_dir: "./out/completion/fish".to_string(),
            project_name: None,
            icon: None,
            grouping: None,
            aliases: vec![],
        };
        assert_eq!(task.name(), "fish");
        assert_eq!(task.category(), "completion");
        assert_eq!(task.binary_name(), "completion-forge");
    }

    #[test]
    fn completion_task_build_args_full() {
        let task = CompletionTask {
            name: "skim-tab".to_string(),
            format: "skim-tab".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/completion/skim-tab".to_string(),
            project_name: Some("my-tool".to_string()),
            icon: Some("*".to_string()),
            grouping: Some("tag".to_string()),
            aliases: vec!["mt".to_string(), "tool".to_string()],
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec![
                "generate", "--spec", "spec.yaml", "--output", "./out/completion/skim-tab",
                "--format", "skim-tab", "--name", "my-tool",
                "--icon", "*", "--grouping", "tag",
                "--aliases", "mt,tool",
            ]
        );
    }

    #[test]
    fn completion_task_build_args_minimal() {
        let task = CompletionTask {
            name: "fish".to_string(),
            format: "fish".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/completion/fish".to_string(),
            project_name: None,
            icon: None,
            grouping: None,
            aliases: vec![],
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec!["generate", "--spec", "spec.yaml", "--output", "./out/completion/fish", "--format", "fish"]
        );
    }

    // ── build_iac_task ──────────────────────────────────────────────────

    #[test]
    fn build_iac_task_sets_correct_fields() {
        let config = make_config();
        let task = build_iac_task("terraform", &config);
        assert_eq!(task.name, "terraform");
        assert_eq!(task.spec, "api.yaml");
        assert!(task.output_dir.ends_with("iac/terraform"));
        assert!(task.resources.is_none());
        assert!(task.provider.is_none());
    }

    #[test]
    fn build_iac_task_propagates_resources_and_provider() {
        let mut config = make_config();
        config.iac_resources = Some("./res".to_string());
        config.iac_provider = Some("./prov.toml".to_string());

        let task = build_iac_task("pulumi", &config);
        assert_eq!(task.resources.as_deref(), Some("./res"));
        assert_eq!(task.provider.as_deref(), Some("./prov.toml"));
    }

    #[test]
    fn iac_task_build_args_with_only_resources() {
        let task = IacForgeTask {
            name: "crossplane".to_string(),
            backend: "crossplane".to_string(),
            category: "iac",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/iac/crossplane".to_string(),
            resources: Some("./r".to_string()),
            provider: None,
        };
        let args = task.build_args();
        assert!(args.contains(&"--resources".to_string()));
        assert!(!args.contains(&"--provider".to_string()));
    }

    #[test]
    fn iac_task_build_args_with_only_provider() {
        let task = IacForgeTask {
            name: "ansible".to_string(),
            backend: "ansible".to_string(),
            category: "iac",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/iac/ansible".to_string(),
            resources: None,
            provider: Some("./p.toml".to_string()),
        };
        let args = task.build_args();
        assert!(!args.contains(&"--resources".to_string()));
        assert!(args.contains(&"--provider".to_string()));
    }

    // ── build_helm_task ─────────────────────────────────────────────────

    #[test]
    fn build_helm_task_sets_correct_fields() {
        let config = make_config();
        let task = build_helm_task("helm", &config);
        assert_eq!(task.name, "helm");
        assert_eq!(task.spec, "api.yaml");
        assert!(task.output_dir.ends_with("helm/helm"));
    }

    #[test]
    fn build_helm_task_uses_helm_resources_when_set() {
        let mut config = make_config();
        config.helm_resources = Some("./helm-res".to_string());
        config.iac_resources = Some("./iac-res".to_string());

        let task = build_helm_task("helm", &config);
        assert_eq!(
            task.resources.as_deref(),
            Some("./helm-res"),
            "helm_resources should take priority over iac_resources"
        );
    }

    #[test]
    fn build_helm_task_falls_back_to_iac_resources() {
        let mut config = make_config();
        config.helm_resources = None;
        config.iac_resources = Some("./iac-res".to_string());

        let task = build_helm_task("helm", &config);
        assert_eq!(
            task.resources.as_deref(),
            Some("./iac-res"),
            "should fall back to iac_resources when helm_resources is None"
        );
    }

    #[test]
    fn build_helm_task_uses_helm_provider_when_set() {
        let mut config = make_config();
        config.helm_provider = Some("./hp.toml".to_string());
        config.iac_provider = Some("./ip.toml".to_string());

        let task = build_helm_task("helm", &config);
        assert_eq!(task.provider.as_deref(), Some("./hp.toml"));
    }

    #[test]
    fn build_helm_task_falls_back_to_iac_provider() {
        let mut config = make_config();
        config.helm_provider = None;
        config.iac_provider = Some("./ip.toml".to_string());

        let task = build_helm_task("helm", &config);
        assert_eq!(task.provider.as_deref(), Some("./ip.toml"));
    }

    #[test]
    fn build_helm_task_no_resources_no_provider() {
        let config = make_config();
        let task = build_helm_task("helm", &config);
        assert!(task.resources.is_none());
        assert!(task.provider.is_none());
    }

    #[test]
    fn helm_task_build_args_minimal() {
        let task = IacForgeTask {
            name: "helm".to_string(),
            backend: "helm".to_string(),
            category: "helm",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/helm/helm".to_string(),
            resources: None,
            provider: None,
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec![
                "generate", "--backend", "helm", "--spec", "spec.yaml",
                "--output", "./out/helm/helm",
            ]
        );
    }

    #[test]
    fn helm_task_build_args_with_provider() {
        let task = IacForgeTask {
            name: "helm".to_string(),
            backend: "helm".to_string(),
            category: "helm",
            spec: "spec.yaml".to_string(),
            output_dir: "./out/helm/helm".to_string(),
            resources: None,
            provider: Some("./p.toml".to_string()),
        };
        let args = task.build_args();
        assert!(args.contains(&"--provider".to_string()));
        assert!(args.contains(&"./p.toml".to_string()));
    }

    // ── build_mcp_task ──────────────────────────────────────────────────

    #[test]
    fn build_mcp_task_sets_correct_fields() {
        let config = make_config();
        let task = build_mcp_task("mcp-rust", &config);
        assert_eq!(task.name, "mcp-rust");
        assert_eq!(task.spec, "api.yaml");
        assert!(task.output_dir.ends_with("mcp/mcp-rust"));
        assert!(task.project_name.is_none());
    }

    #[test]
    fn build_mcp_task_propagates_project_name() {
        let mut config = make_config();
        config.mcp_name = Some("my-server".to_string());

        let task = build_mcp_task("mcp-rust", &config);
        assert_eq!(task.project_name.as_deref(), Some("my-server"));
    }

    #[test]
    fn mcp_task_build_args_without_name() {
        let task = McpTask {
            name: "mcp-rust".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/mcp/mcp-rust".to_string(),
            project_name: None,
        };
        let args = task.build_args();
        assert_eq!(
            args,
            vec!["generate", "--spec", "spec.yaml", "--output", "./out/mcp/mcp-rust"]
        );
        assert!(!args.contains(&"--name".to_string()));
    }

    // ── resolve_targets edge cases ──────────────────────────────────────

    #[test]
    fn resolve_targets_all_case_insensitive() {
        let result = resolve_targets(&["ALL".to_string()], Category::Helm);
        assert!(result.contains(&"helm".to_string()));
    }

    #[test]
    fn resolve_targets_all_mixed_with_others_still_expands() {
        let result = resolve_targets(
            &["go".to_string(), "all".to_string()],
            Category::Completion,
        );
        assert!(
            result.contains(&"skim-tab".to_string()),
            "presence of 'all' anywhere should expand"
        );
        assert!(result.contains(&"fish".to_string()));
    }

    #[test]
    fn resolve_targets_single_specific() {
        let result = resolve_targets(&["terraform".to_string()], Category::Iac);
        assert_eq!(result, vec!["terraform"]);
    }

    #[test]
    fn resolve_targets_all_sdk_returns_28() {
        let result = resolve_targets(&["all".to_string()], Category::Sdk);
        assert_eq!(result.len(), 28);
    }

    #[test]
    fn resolve_targets_all_iac_returns_6() {
        let result = resolve_targets(&["all".to_string()], Category::Iac);
        assert_eq!(result.len(), 6);
    }

    // ── build_openapi_task edge cases ───────────────────────────────────

    #[test]
    fn build_openapi_task_unknown_generator_uses_name_as_generator() {
        let config = make_config();
        let task = build_openapi_task("nonexistent-gen", "sdk", &config);
        assert_eq!(
            task.generator, "nonexistent-gen",
            "unknown generators should use the name as the generator string"
        );
    }

    #[test]
    fn build_openapi_task_known_generator_maps_name_to_generator() {
        let config = make_config();
        let task = build_openapi_task("typescript", "sdk", &config);
        assert_eq!(task.generator, "typescript-fetch");
    }

    #[test]
    fn build_openapi_task_server_category() {
        let config = make_config();
        let task = build_openapi_task("rust-axum", "server", &config);
        assert_eq!(task.category, "server");
        assert_eq!(task.generator, "rust-axum");
        assert!(task.output_dir.ends_with("server/rust-axum"));
    }

    #[test]
    fn build_openapi_task_schema_category() {
        let config = make_config();
        let task = build_openapi_task("graphql-schema", "schema", &config);
        assert_eq!(task.category, "schema");
        assert!(task.output_dir.ends_with("schema/graphql-schema"));
    }

    #[test]
    fn build_openapi_task_doc_category() {
        let config = make_config();
        let task = build_openapi_task("html", "doc", &config);
        assert_eq!(task.generator, "html2");
        assert!(task.output_dir.ends_with("doc/html"));
    }

    // ── completion_task partial options ─────────────────────────────────

    #[test]
    fn completion_task_build_args_with_only_name() {
        let task = CompletionTask {
            name: "skim-tab".to_string(),
            format: "skim-tab".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/completion/skim-tab".to_string(),
            project_name: Some("cli".to_string()),
            icon: None,
            grouping: None,
            aliases: vec![],
        };
        let args = task.build_args();
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"cli".to_string()));
        assert!(!args.contains(&"--icon".to_string()));
        assert!(!args.contains(&"--grouping".to_string()));
        assert!(!args.contains(&"--aliases".to_string()));
    }

    #[test]
    fn completion_task_build_args_with_only_icon() {
        let task = CompletionTask {
            name: "fish".to_string(),
            format: "fish".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out/completion/fish".to_string(),
            project_name: None,
            icon: Some("🐟".to_string()),
            grouping: None,
            aliases: vec![],
        };
        let args = task.build_args();
        assert!(args.contains(&"--icon".to_string()));
        assert!(args.contains(&"🐟".to_string()));
        assert!(!args.contains(&"--name".to_string()));
    }

    #[test]
    fn completion_task_build_args_with_only_grouping() {
        let task = CompletionTask {
            name: "skim-tab".to_string(),
            format: "skim-tab".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out".to_string(),
            project_name: None,
            icon: None,
            grouping: Some("path".to_string()),
            aliases: vec![],
        };
        let args = task.build_args();
        assert!(args.contains(&"--grouping".to_string()));
        assert!(args.contains(&"path".to_string()));
    }

    #[test]
    fn completion_task_single_alias() {
        let task = CompletionTask {
            name: "fish".to_string(),
            format: "fish".to_string(),
            spec: "spec.yaml".to_string(),
            output_dir: "./out".to_string(),
            project_name: None,
            icon: None,
            grouping: None,
            aliases: vec!["f".to_string()],
        };
        let args = task.build_args();
        assert!(args.contains(&"--aliases".to_string()));
        assert!(args.contains(&"f".to_string()));
    }

    // ── build_completion_task edge cases ───────────────────────────────

    #[test]
    fn build_completion_task_output_dir_format() {
        let config = make_config();
        let task = build_completion_task("fish", &config);
        assert_eq!(task.output_dir, "./out/completion/fish");
    }

    #[test]
    fn build_completion_task_no_optional_fields() {
        let config = make_config();
        let task = build_completion_task("skim-tab", &config);
        assert!(task.project_name.is_none());
        assert!(task.icon.is_none());
        assert!(task.grouping.is_none());
        assert!(task.aliases.is_empty());
    }

    // ── openapi_task generator mapping ──────────────────────────────────

    #[test]
    fn build_openapi_task_swift_maps_to_swift6() {
        let config = make_config();
        let task = build_openapi_task("swift", "sdk", &config);
        assert_eq!(task.generator, "swift6");
    }

    #[test]
    fn build_openapi_task_scala_maps_to_sttp() {
        let config = make_config();
        let task = build_openapi_task("scala", "sdk", &config);
        assert_eq!(task.generator, "scala-sttp");
    }

    #[test]
    fn build_openapi_task_haskell_maps_to_http_client() {
        let config = make_config();
        let task = build_openapi_task("haskell", "sdk", &config);
        assert_eq!(task.generator, "haskell-http-client");
    }

    #[test]
    fn build_openapi_task_cpp_maps_to_restsdk() {
        let config = make_config();
        let task = build_openapi_task("cpp", "sdk", &config);
        assert_eq!(task.generator, "cpp-restsdk");
    }

    // ── resolve_targets for every category ──────────────────────────────

    #[test]
    fn resolve_targets_all_server_returns_5() {
        let result = resolve_targets(&["all".to_string()], Category::Server);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn resolve_targets_all_schema_returns_4() {
        let result = resolve_targets(&["all".to_string()], Category::Schema);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn resolve_targets_all_doc_returns_4() {
        let result = resolve_targets(&["all".to_string()], Category::Doc);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn resolve_targets_all_mcp_returns_1() {
        let result = resolve_targets(&["all".to_string()], Category::Mcp);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "mcp-rust");
    }

    #[test]
    fn resolve_targets_all_completion_returns_2() {
        let result = resolve_targets(&["all".to_string()], Category::Completion);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn resolve_targets_all_helm_returns_1() {
        let result = resolve_targets(&["all".to_string()], Category::Helm);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "helm");
    }

    #[test]
    fn resolve_targets_multiple_specific() {
        let input = vec![
            "terraform".to_string(),
            "pulumi".to_string(),
            "crossplane".to_string(),
        ];
        let result = resolve_targets(&input, Category::Iac);
        assert_eq!(result, input);
    }

    // ── TaskRunner trait contract ────────────────────────────────────────

    #[test]
    fn all_task_types_implement_correct_binary_names() {
        let config = make_config();

        let openapi = build_openapi_task("go", "sdk", &config);
        assert_eq!(openapi.binary_name(), "openapi-generator-cli");

        let iac = build_iac_task("terraform", &config);
        assert_eq!(iac.binary_name(), "iac-forge");

        let helm = build_helm_task("helm", &config);
        assert_eq!(helm.binary_name(), "iac-forge");

        let mcp = build_mcp_task("mcp-rust", &config);
        assert_eq!(mcp.binary_name(), "mcp-forge");

        let comp = build_completion_task("fish", &config);
        assert_eq!(comp.binary_name(), "completion-forge");
    }

    #[test]
    fn all_task_types_implement_correct_categories() {
        let config = make_config();

        let openapi = build_openapi_task("go", "sdk", &config);
        assert_eq!(openapi.category(), "sdk");

        let iac = build_iac_task("terraform", &config);
        assert_eq!(iac.category(), "iac");

        let helm = build_helm_task("helm", &config);
        assert_eq!(helm.category(), "helm");

        let mcp = build_mcp_task("mcp-rust", &config);
        assert_eq!(mcp.category(), "mcp");

        let comp = build_completion_task("fish", &config);
        assert_eq!(comp.category(), "completion");
    }

    // ── Helper ──────────────────────────────────────────────────────────

    fn make_config() -> GenerateConfig {
        GenerateConfig {
            spec: "api.yaml".to_string(),
            output_dir: "./out".to_string(),
            ..GenerateConfig::default()
        }
    }
}
