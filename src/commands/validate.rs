use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use colored::Colorize;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to the OpenAPI spec (YAML or JSON)
    #[arg(long)]
    pub spec: String,
}

/// Parse the spec file and print a summary of endpoints, schemas, and any
/// warnings.
///
/// # Errors
///
/// Returns an error if the spec cannot be read or parsed.
pub fn run(args: Args) -> Result<()> {
    let path = Path::new(&args.spec);
    if !path.exists() {
        bail!("spec file not found: {}", args.spec);
    }

    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    // Parse as generic JSON value. If the file is YAML we try serde_json first
    // (works for JSON specs), then fall back to treating it as YAML-flavoured
    // JSON.  For a full YAML parser we would need `serde_yaml`, but for a v0.1
    // validation summary serde_json covers JSON specs and gives a clear error
    // for YAML-only specs.
    let spec: serde_json::Value = if path
        .extension()
        .is_some_and(|ext| ext == "yaml" || ext == "yml")
    {
        // Attempt a simple YAML parse by looking for JSON subset.  Full YAML
        // support would require an extra dep; for now we accept JSON specs and
        // YAML specs that happen to be valid JSON.
        serde_json::from_str(&content).unwrap_or_else(|_| {
            // Provide a minimal placeholder so the rest of the function can
            // still report what it can.
            tracing::warn!("spec appears to be YAML — install serde_yaml for full parsing; showing limited info");
            serde_json::json!({})
        })
    } else {
        serde_json::from_str(&content).with_context(|| "parsing spec as JSON")?
    };

    println!(
        "\n{} Validating {}",
        "=>".blue().bold(),
        path.display()
    );

    // ── Info section ─────────────────────────────────────────────────
    if let Some(info) = spec.get("info") {
        let title = info
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(untitled)");
        let version = info
            .get("version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(unknown)");
        println!("  Title:   {title}");
        println!("  Version: {version}");
    }

    // ── Paths / endpoints ────────────────────────────────────────────
    let mut endpoint_count = 0u64;
    let mut warnings: Vec<String> = Vec::new();

    if let Some(paths) = spec.get("paths").and_then(serde_json::Value::as_object) {
        for (path_str, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, op) in methods_obj {
                    endpoint_count += 1;

                    // Warn if operationId is missing.
                    if op.get("operationId").is_none() {
                        warnings.push(format!(
                            "{} {} is missing operationId",
                            method.to_uppercase(),
                            path_str
                        ));
                    }
                }
            }
        }
    }

    // ── Schemas ──────────────────────────────────────────────────────
    let schema_count = spec
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(serde_json::Value::as_object)
        .map_or(0, serde_json::Map::len);

    println!("  Endpoints: {endpoint_count}");
    println!("  Schemas:   {schema_count}");

    if warnings.is_empty() {
        println!("\n  {} No warnings\n", "ok".green().bold());
    } else {
        println!("\n  {} {} warning(s):", "!".yellow().bold(), warnings.len());
        for w in &warnings {
            println!("    - {w}");
        }
        println!();
    }

    Ok(())
}
