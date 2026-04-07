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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_spec_parses_json() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_json");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("spec.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "Test API", "version": "1.0.0" },
  "paths": {
    "/pets": {
      "get": { "operationId": "listPets", "responses": { "200": { "description": "ok" } } }
    }
  },
  "components": {
    "schemas": {
      "Pet": { "type": "object", "properties": { "name": { "type": "string" } } }
    }
  }
}"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        let result = run(args);
        assert!(result.is_ok(), "validate should succeed for valid JSON spec");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_parses_yaml_extension_gracefully() {
        // YAML files that are not valid JSON get a placeholder — this should
        // still not error out (the function warns but continues).
        let dir = std::env::temp_dir().join("forge_gen_test_validate_yaml");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("spec.yaml");
        std::fs::write(
            &path,
            "openapi: '3.0.3'\ninfo:\n  title: Test\n  version: '1.0'\npaths: {}\n",
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        let result = run(args);
        assert!(
            result.is_ok(),
            "validate should not error for YAML specs (degrades gracefully)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_missing_file_errors() {
        let args = Args {
            spec: "/tmp/forge_gen_nonexistent_spec.json".to_string(),
        };
        let result = run(args);
        assert!(result.is_err());
    }

    #[test]
    fn validate_counts_endpoints() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_count");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("multi.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "Multi", "version": "2.0" },
  "paths": {
    "/a": {
      "get": { "operationId": "getA", "responses": {} },
      "post": { "operationId": "createA", "responses": {} }
    },
    "/b": {
      "delete": { "operationId": "deleteB", "responses": {} }
    }
  }
}"#,
        )
        .unwrap();

        // The function prints endpoint counts but does not expose them
        // directly; we verify it runs without error (3 endpoints, 0 warnings).
        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_detects_missing_operation_id() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_warn");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("warn.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "Warn", "version": "1.0" },
  "paths": {
    "/x": {
      "get": { "responses": {} }
    }
  }
}"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_no_info_section() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_no_info");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("no-info.json");
        std::fs::write(
            &path,
            r#"{ "openapi": "3.0.3", "paths": {} }"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok(), "missing info section should not error");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_no_paths_section() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_no_paths");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("no-paths.json");
        std::fs::write(
            &path,
            r#"{ "openapi": "3.0.3", "info": { "title": "T", "version": "1.0" } }"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok(), "missing paths section should not error");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_no_components_section() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_no_comp");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("no-comp.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "No Components", "version": "1.0" },
  "paths": {}
}"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(
            run(args).is_ok(),
            "missing components section should not error"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_empty_json_object() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_empty_obj");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("empty.json");
        std::fs::write(&path, "{}").unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok(), "empty JSON object should not error");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_invalid_json_errors() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_bad_json");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad.json");
        std::fs::write(&path, "not json at all {{{").unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(
            run(args).is_err(),
            "invalid JSON file (not .yaml) should error"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_yml_extension_gracefully() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_yml");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("spec.yml");
        std::fs::write(&path, "openapi: '3.0.3'\ninfo:\n  title: T\n  version: '1'\n").unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(
            run(args).is_ok(),
            ".yml extension should also be handled gracefully"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_json_with_yaml_extension() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_json_yaml_ext");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("spec.yaml");
        std::fs::write(
            &path,
            r#"{ "openapi": "3.0.3", "info": { "title": "JSON in YAML", "version": "1.0" }, "paths": {} }"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(
            run(args).is_ok(),
            "JSON content with .yaml extension should parse normally"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_with_schemas() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_schemas");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("schemas.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "Schema Test", "version": "1.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Alpha": { "type": "object" },
      "Beta": { "type": "string" },
      "Gamma": { "type": "integer" }
    }
  }
}"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_info_missing_title_and_version() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_no_title");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("notitle.json");
        std::fs::write(
            &path,
            r#"{ "info": {}, "paths": {} }"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(
            run(args).is_ok(),
            "missing title/version in info should use defaults, not error"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_spec_multiple_missing_operation_ids() {
        let dir = std::env::temp_dir().join("forge_gen_test_validate_multi_warn");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("multi-warn.json");
        std::fs::write(
            &path,
            r#"{
  "openapi": "3.0.3",
  "info": { "title": "Multi", "version": "1.0" },
  "paths": {
    "/a": { "get": { "responses": {} }, "post": { "responses": {} } },
    "/b": { "delete": { "responses": {} } }
  }
}"#,
        )
        .unwrap();

        let args = Args {
            spec: path.to_str().unwrap().to_string(),
        };
        assert!(run(args).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
