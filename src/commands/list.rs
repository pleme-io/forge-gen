use clap::Args as ClapArgs;
use colored::Colorize;

use crate::registry::{self, Category};

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
pub fn run(args: Args) -> anyhow::Result<()> {
    let categories: Vec<Category> = if let Some(ref filter) = args.category {
        vec![parse_category(filter)?]
    } else {
        vec![
            Category::Sdk,
            Category::Server,
            Category::Schema,
            Category::Doc,
            Category::Iac,
        ]
    };

    for cat in &categories {
        let generators = registry::by_category(*cat);
        if generators.is_empty() {
            continue;
        }

        println!("\n  {} ({})", format!("{cat}").bold(), generators.len());
        println!("  {:<24} {:<26} {}", "Name", "Generator", "Description");
        println!("  {}", "-".repeat(74));

        for g in &generators {
            println!("  {:<24} {:<26} {}", g.name.green(), g.generator, g.description);
        }
    }
    println!();

    let total = registry::REGISTRY.len();
    println!("  {} generators available\n", total.to_string().bold());

    Ok(())
}

fn parse_category(s: &str) -> anyhow::Result<Category> {
    match s.to_lowercase().as_str() {
        "sdk" | "sdks" => Ok(Category::Sdk),
        "server" | "servers" => Ok(Category::Server),
        "schema" | "schemas" => Ok(Category::Schema),
        "doc" | "docs" => Ok(Category::Doc),
        "iac" => Ok(Category::Iac),
        other => anyhow::bail!(
            "unknown category: {other} (expected sdk, server, schema, doc, or iac)"
        ),
    }
}
