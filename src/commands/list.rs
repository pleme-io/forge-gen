use clap::Args as ClapArgs;
use colored::Colorize;

use crate::registry::{self, Category};

/// CLI arguments for the `list` subcommand.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Filter to a specific category (sdk, server, schema, doc, iac)
    #[arg(long)]
    pub category: Option<String>,
}

/// Print a formatted table of all available generators grouped by category.
///
/// # Errors
///
/// Returns an error if the filter category is unrecognised.
pub fn run(args: &Args) -> anyhow::Result<()> {
    let categories: Vec<Category> = if let Some(ref filter) = args.category {
        vec![parse_category(filter)?]
    } else {
        vec![
            Category::Sdk,
            Category::Server,
            Category::Schema,
            Category::Doc,
            Category::Iac,
            Category::Helm,
            Category::Mcp,
            Category::Grpc,
            Category::Completion,
        ]
    };

    for cat in &categories {
        let generators = registry::by_category(*cat);
        if generators.is_empty() {
            continue;
        }

        println!("\n  {} ({})", format!("{cat}").bold(), generators.len());
        println!("  {:<24} {:<26} Description", "Name", "Generator");
        println!("  {}", "-".repeat(74));

        for g in &generators {
            println!(
                "  {:<24} {:<26} {}",
                g.name.green(),
                g.generator,
                g.description
            );
        }
    }
    println!();

    let total = registry::REGISTRY.len();
    println!("  {} generators available\n", total.to_string().bold());

    Ok(())
}

fn parse_category(s: &str) -> anyhow::Result<Category> {
    Ok(s.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_no_filter_lists_all_categories() {
        let args = Args { category: None };
        let result = run(&args);
        assert!(result.is_ok(), "list with no filter should succeed");
    }

    #[test]
    fn run_filter_sdk() {
        let args = Args {
            category: Some("sdk".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_server() {
        let args = Args {
            category: Some("server".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_iac() {
        let args = Args {
            category: Some("iac".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_schema() {
        let args = Args {
            category: Some("schema".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_doc() {
        let args = Args {
            category: Some("doc".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_helm() {
        let args = Args {
            category: Some("helm".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_mcp() {
        let args = Args {
            category: Some("mcp".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_filter_completion() {
        let args = Args {
            category: Some("completion".to_string()),
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn run_invalid_category_returns_error() {
        let args = Args {
            category: Some("bogus".to_string()),
        };
        let result = run(&args);
        assert!(result.is_err(), "invalid category should produce an error");
    }

    #[test]
    fn run_empty_string_category_returns_error() {
        let args = Args {
            category: Some(String::new()),
        };
        assert!(run(&args).is_err());
    }

    #[test]
    fn parse_category_case_insensitive() {
        assert!(parse_category("SDK").is_ok());
        assert!(parse_category("Sdk").is_ok());
        assert!(parse_category("sDk").is_ok());
    }

    #[test]
    fn parse_category_plural_forms() {
        assert_eq!(parse_category("sdks").unwrap(), Category::Sdk);
        assert_eq!(parse_category("servers").unwrap(), Category::Server);
        assert_eq!(parse_category("schemas").unwrap(), Category::Schema);
        assert_eq!(parse_category("docs").unwrap(), Category::Doc);
        assert_eq!(parse_category("completions").unwrap(), Category::Completion);
    }
}
