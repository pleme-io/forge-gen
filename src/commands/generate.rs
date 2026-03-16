use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use colored::Colorize;
use tokio::process::Command;
use tokio::task::JoinSet;

use crate::manifest::{self, GenerateConfig};
use crate::registry::{self, Category};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to an OpenAPI spec (YAML or JSON)
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

    /// Comma-separated IaC backends or "all"
    #[arg(long)]
    pub iac: Option<String>,

    /// Comma-separated schema types or "all"
    #[arg(long)]
    pub schemas: Option<String>,

    /// Comma-separated doc formats or "all"
    #[arg(long)]
    pub docs: Option<String>,

    /// TOML resource specs directory (for IaC generation)
    #[arg(long)]
    pub resources: Option<String>,

    /// Path to provider.toml (for IaC generation)
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

    let total =
        sdks.len() + servers.len() + schemas.len() + docs.len() + iac_backends.len();

    if total == 0 {
        bail!(
            "nothing to generate — specify at least one of --sdks, --servers, --iac, --schemas, --docs"
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

    if config.parallel {
        let mut set = JoinSet::new();

        for name in &sdks {
            let task = build_openapi_task(name, "sdk", &config);
            set.spawn(run_openapi_generator(task));
        }
        for name in &servers {
            let task = build_openapi_task(name, "server", &config);
            set.spawn(run_openapi_generator(task));
        }
        for name in &schemas {
            let task = build_openapi_task(name, "schema", &config);
            set.spawn(run_openapi_generator(task));
        }
        for name in &docs {
            let task = build_openapi_task(name, "doc", &config);
            set.spawn(run_openapi_generator(task));
        }
        for name in &iac_backends {
            let task = build_iac_task(name, &config);
            set.spawn(run_iac_generator(task));
        }

        while let Some(res) = set.join_next().await {
            results.push(res.context("task panicked")??);
        }
    } else {
        for name in &sdks {
            let task = build_openapi_task(name, "sdk", &config);
            results.push(run_openapi_generator(task).await?);
        }
        for name in &servers {
            let task = build_openapi_task(name, "server", &config);
            results.push(run_openapi_generator(task).await?);
        }
        for name in &schemas {
            let task = build_openapi_task(name, "schema", &config);
            results.push(run_openapi_generator(task).await?);
        }
        for name in &docs {
            let task = build_openapi_task(name, "doc", &config);
            results.push(run_openapi_generator(task).await?);
        }
        for name in &iac_backends {
            let task = build_iac_task(name, &config);
            results.push(run_iac_generator(task).await?);
        }
    }

    let elapsed = started.elapsed();

    // ── Summary table ────────────────────────────────────────────────
    println!("\n{}", "  Generator Results".bold());
    println!(
        "  {:<24} {:<10} {:<8} {}",
        "Target", "Category", "Status", "Output"
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
    category: String,
    spec: String,
    output_dir: String,
}

/// Describes an iac-forge invocation.
struct IacTask {
    name: String,
    spec: String,
    output_dir: String,
    resources: Option<String>,
    provider: Option<String>,
}

fn build_openapi_task(name: &str, category: &str, config: &GenerateConfig) -> OpenApiTask {
    let info = registry::find(name);
    let generator = info.map_or_else(|| name.to_string(), |i| i.generator.to_string());
    let out = format!("{}/{category}/{name}", config.output_dir);

    OpenApiTask {
        name: name.to_string(),
        generator,
        category: category.to_string(),
        spec: config.spec.clone(),
        output_dir: out,
    }
}

fn build_iac_task(name: &str, config: &GenerateConfig) -> IacTask {
    let out = format!("{}/iac/{name}", config.output_dir);

    IacTask {
        name: name.to_string(),
        spec: config.spec.clone(),
        output_dir: out,
        resources: config.iac_resources.clone(),
        provider: config.iac_provider.clone(),
    }
}

async fn run_openapi_generator(task: OpenApiTask) -> Result<TaskResult> {
    println!(
        "  {} [{}/{}] openapi-generator-cli -g {}",
        "->".green(),
        task.category,
        task.name,
        task.generator,
    );

    std::fs::create_dir_all(&task.output_dir)?;

    let bin = which::which("openapi-generator-cli").ok();

    let success = if let Some(bin) = bin {
        let status = Command::new(bin)
            .args([
                "generate",
                "-i",
                &task.spec,
                "-g",
                &task.generator,
                "-o",
                &task.output_dir,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .with_context(|| format!("spawning openapi-generator-cli for {}", task.name))?;

        status.success()
    } else {
        tracing::warn!(
            target = task.name,
            "openapi-generator-cli not found in PATH — skipping"
        );
        false
    };

    Ok(TaskResult {
        name: task.name,
        category: task.category,
        output_dir: task.output_dir,
        success,
    })
}

async fn run_iac_generator(task: IacTask) -> Result<TaskResult> {
    println!(
        "  {} [iac/{}] iac-forge generate --backend {}",
        "->".green(),
        task.name,
        task.name,
    );

    std::fs::create_dir_all(&task.output_dir)?;

    let bin = which::which("iac-forge").ok();

    let success = if let Some(bin) = bin {
        let mut cmd = Command::new(bin);
        cmd.args([
            "generate",
            "--backend",
            &task.name,
            "--spec",
            &task.spec,
            "--output",
            &task.output_dir,
        ]);

        if let Some(ref resources) = task.resources {
            cmd.args(["--resources", resources]);
        }
        if let Some(ref provider) = task.provider {
            cmd.args(["--provider", provider]);
        }

        let status = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .with_context(|| format!("spawning iac-forge for {}", task.name))?;

        status.success()
    } else {
        tracing::warn!(
            target = task.name,
            "iac-forge not found in PATH — skipping"
        );
        false
    };

    Ok(TaskResult {
        name: task.name,
        category: String::from("iac"),
        output_dir: task.output_dir,
        success,
    })
}
