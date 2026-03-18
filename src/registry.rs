use std::fmt;

/// Category of a code generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Sdk,
    Server,
    Schema,
    Doc,
    Iac,
    Helm,
    Mcp,
    Completion,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdk => write!(f, "SDK"),
            Self::Server => write!(f, "Server"),
            Self::Schema => write!(f, "Schema"),
            Self::Doc => write!(f, "Doc"),
            Self::Iac => write!(f, "IaC"),
            Self::Helm => write!(f, "Helm"),
            Self::Mcp => write!(f, "MCP"),
            Self::Completion => write!(f, "Completion"),
        }
    }
}

/// Metadata for a single generator target.
#[derive(Debug)]
pub struct GeneratorInfo {
    /// Friendly name used on the CLI (e.g. "go", "typescript-axios").
    pub name: &'static str,
    /// Value passed to `openapi-generator-cli -g` (or iac-forge `--backend`).
    pub generator: &'static str,
    /// Which category this generator belongs to.
    pub category: Category,
    /// One-line description.
    pub description: &'static str,
}

/// Full static registry of every supported generator.
///
/// Includes all 40+ openapi-generator-cli generators from `openapi-sdk.nix`,
/// plus the 6 IaC backends from `iac-forge`.
pub static REGISTRY: &[GeneratorInfo] = &[
    // ── Client SDKs ──────────────────────────────────────────────────
    GeneratorInfo { name: "go",                  generator: "go",                    category: Category::Sdk, description: "Go client SDK" },
    GeneratorInfo { name: "python",              generator: "python",                category: Category::Sdk, description: "Python client SDK" },
    GeneratorInfo { name: "javascript",          generator: "javascript",            category: Category::Sdk, description: "JavaScript client SDK" },
    GeneratorInfo { name: "typescript",          generator: "typescript-fetch",       category: Category::Sdk, description: "TypeScript client (fetch)" },
    GeneratorInfo { name: "typescript-axios",    generator: "typescript-axios",       category: Category::Sdk, description: "TypeScript client (Axios)" },
    GeneratorInfo { name: "typescript-node",     generator: "typescript-node",        category: Category::Sdk, description: "TypeScript client (Node.js)" },
    GeneratorInfo { name: "typescript-angular",  generator: "typescript-angular",     category: Category::Sdk, description: "TypeScript client (Angular)" },
    GeneratorInfo { name: "java",                generator: "java",                   category: Category::Sdk, description: "Java client SDK" },
    GeneratorInfo { name: "ruby",                generator: "ruby",                   category: Category::Sdk, description: "Ruby client SDK" },
    GeneratorInfo { name: "csharp",              generator: "csharp",                 category: Category::Sdk, description: "C# client SDK (.NET 6)" },
    GeneratorInfo { name: "rust",                generator: "rust",                   category: Category::Sdk, description: "Rust client SDK (reqwest)" },
    GeneratorInfo { name: "kotlin",              generator: "kotlin",                 category: Category::Sdk, description: "Kotlin client SDK" },
    GeneratorInfo { name: "swift",               generator: "swift6",                 category: Category::Sdk, description: "Swift 6 client SDK" },
    GeneratorInfo { name: "dart",                generator: "dart",                   category: Category::Sdk, description: "Dart client SDK" },
    GeneratorInfo { name: "php",                 generator: "php",                    category: Category::Sdk, description: "PHP client SDK" },
    GeneratorInfo { name: "perl",                generator: "perl",                   category: Category::Sdk, description: "Perl client SDK" },
    GeneratorInfo { name: "elixir",              generator: "elixir",                 category: Category::Sdk, description: "Elixir client SDK" },
    GeneratorInfo { name: "scala",               generator: "scala-sttp",             category: Category::Sdk, description: "Scala client SDK (sttp)" },
    GeneratorInfo { name: "haskell",             generator: "haskell-http-client",    category: Category::Sdk, description: "Haskell client SDK" },
    GeneratorInfo { name: "c",                   generator: "c",                      category: Category::Sdk, description: "C client SDK" },
    GeneratorInfo { name: "cpp",                 generator: "cpp-restsdk",            category: Category::Sdk, description: "C++ client SDK (restsdk)" },
    GeneratorInfo { name: "lua",                 generator: "lua",                    category: Category::Sdk, description: "Lua client SDK" },
    GeneratorInfo { name: "r",                   generator: "r",                      category: Category::Sdk, description: "R client SDK" },
    GeneratorInfo { name: "ocaml",               generator: "ocaml",                  category: Category::Sdk, description: "OCaml client SDK" },
    GeneratorInfo { name: "clojure",             generator: "clojure",                category: Category::Sdk, description: "Clojure client SDK" },
    GeneratorInfo { name: "elm",                 generator: "elm",                    category: Category::Sdk, description: "Elm client SDK" },
    GeneratorInfo { name: "powershell",          generator: "powershell",             category: Category::Sdk, description: "PowerShell client SDK" },
    GeneratorInfo { name: "bash",                generator: "bash",                   category: Category::Sdk, description: "Bash client SDK (curl)" },

    // ── Server stubs ─────────────────────────────────────────────────
    GeneratorInfo { name: "go-server",           generator: "go-server",              category: Category::Server, description: "Go server stub" },
    GeneratorInfo { name: "python-fastapi",      generator: "python-fastapi",         category: Category::Server, description: "Python FastAPI server stub" },
    GeneratorInfo { name: "rust-axum",           generator: "rust-axum",              category: Category::Server, description: "Rust Axum server stub" },
    GeneratorInfo { name: "spring",              generator: "spring",                 category: Category::Server, description: "Java Spring Boot server stub" },
    GeneratorInfo { name: "kotlin-spring",       generator: "kotlin-spring",          category: Category::Server, description: "Kotlin Spring server stub" },

    // ── Schema generators ────────────────────────────────────────────
    GeneratorInfo { name: "graphql-schema",      generator: "graphql-schema",         category: Category::Schema, description: "GraphQL schema from OpenAPI" },
    GeneratorInfo { name: "protobuf-schema",     generator: "protobuf-schema",        category: Category::Schema, description: "Protobuf schema from OpenAPI" },
    GeneratorInfo { name: "mysql-schema",        generator: "mysql-schema",           category: Category::Schema, description: "MySQL DDL from OpenAPI models" },
    GeneratorInfo { name: "postgresql-schema",   generator: "postgresql-schema",      category: Category::Schema, description: "PostgreSQL DDL from OpenAPI models" },

    // ── Documentation generators ─────────────────────────────────────
    GeneratorInfo { name: "markdown",            generator: "markdown",               category: Category::Doc, description: "Markdown API documentation" },
    GeneratorInfo { name: "html",                generator: "html2",                  category: Category::Doc, description: "HTML API documentation" },
    GeneratorInfo { name: "asciidoc",            generator: "asciidoc",               category: Category::Doc, description: "AsciiDoc API documentation" },
    GeneratorInfo { name: "plantuml",            generator: "plantuml",               category: Category::Doc, description: "PlantUML diagrams from OpenAPI" },

    // ── IaC backends (via iac-forge) ─────────────────────────────────
    GeneratorInfo { name: "terraform",           generator: "terraform",              category: Category::Iac, description: "Terraform provider (Go, plugin-framework)" },
    GeneratorInfo { name: "pulumi",              generator: "pulumi",                 category: Category::Iac, description: "Pulumi provider (schema.json)" },
    GeneratorInfo { name: "crossplane",          generator: "crossplane",             category: Category::Iac, description: "Crossplane provider (CRD YAML)" },
    GeneratorInfo { name: "ansible",             generator: "ansible",                category: Category::Iac, description: "Ansible collection (Python modules)" },
    GeneratorInfo { name: "pangea",              generator: "pangea",                 category: Category::Iac, description: "Pangea DSL resources (Ruby)" },
    GeneratorInfo { name: "steampipe",           generator: "steampipe",              category: Category::Iac, description: "Steampipe plugin tables (Go)" },

    // ── Helm chart backends (via helm-forge / iac-forge) ──────────────
    GeneratorInfo { name: "helm",                generator: "helm",                   category: Category::Helm, description: "Helm charts from resource specs (pleme-lib)" },

    // ── MCP server backends (via mcp-forge) ─────────────────────────
    GeneratorInfo { name: "mcp-rust",            generator: "mcp-rust",               category: Category::Mcp, description: "Rust MCP server (rmcp 0.15, CLI + stdio)" },

    // ── Completion generators (via completion-forge) ────────────────
    GeneratorInfo { name: "skim-tab",            generator: "skim-tab",               category: Category::Completion, description: "skim-tab YAML completion spec" },
    GeneratorInfo { name: "fish",                generator: "fish",                   category: Category::Completion, description: "Fish shell completion file" },
];

/// Look up a generator by its friendly name.
#[must_use]
pub fn find(name: &str) -> Option<&'static GeneratorInfo> {
    REGISTRY.iter().find(|g| g.name == name)
}

/// Return all generators for a given category.
#[must_use]
pub fn by_category(category: Category) -> Vec<&'static GeneratorInfo> {
    REGISTRY
        .iter()
        .filter(|g| g.category == category)
        .collect()
}

/// Return all friendly names for a given category.
#[must_use]
pub fn names_for_category(category: Category) -> Vec<&'static str> {
    by_category(category).iter().map(|g| g.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Registry contents ───────────────────────────────────────────────

    #[test]
    fn registry_is_non_empty() {
        assert!(!REGISTRY.is_empty());
    }

    #[test]
    fn registry_contains_expected_sdk_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Sdk)
            .map(|g| g.name)
            .collect();

        for expected in ["go", "python", "rust", "java", "typescript", "ruby", "csharp"] {
            assert!(
                names.contains(&expected),
                "SDK registry missing {expected}"
            );
        }
    }

    #[test]
    fn registry_contains_expected_server_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Server)
            .map(|g| g.name)
            .collect();

        for expected in ["go-server", "python-fastapi", "rust-axum", "spring"] {
            assert!(
                names.contains(&expected),
                "Server registry missing {expected}"
            );
        }
    }

    #[test]
    fn registry_contains_expected_iac_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Iac)
            .map(|g| g.name)
            .collect();

        for expected in [
            "terraform",
            "pulumi",
            "crossplane",
            "ansible",
            "pangea",
            "steampipe",
        ] {
            assert!(
                names.contains(&expected),
                "IaC registry missing {expected}"
            );
        }
    }

    #[test]
    fn registry_contains_expected_schema_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Schema)
            .map(|g| g.name)
            .collect();

        for expected in ["graphql-schema", "protobuf-schema", "mysql-schema", "postgresql-schema"]
        {
            assert!(
                names.contains(&expected),
                "Schema registry missing {expected}"
            );
        }
    }

    #[test]
    fn registry_contains_expected_doc_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Doc)
            .map(|g| g.name)
            .collect();

        for expected in ["markdown", "html", "asciidoc", "plantuml"] {
            assert!(
                names.contains(&expected),
                "Doc registry missing {expected}"
            );
        }
    }

    #[test]
    fn registry_contains_expected_helm_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Helm)
            .map(|g| g.name)
            .collect();

        assert!(names.contains(&"helm"), "Helm registry missing helm");
    }

    #[test]
    fn registry_contains_expected_mcp_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Mcp)
            .map(|g| g.name)
            .collect();

        assert!(
            names.contains(&"mcp-rust"),
            "MCP registry missing mcp-rust"
        );
    }

    #[test]
    fn registry_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for g in REGISTRY {
            assert!(
                seen.insert(g.name),
                "duplicate registry name: {}",
                g.name
            );
        }
    }

    #[test]
    fn registry_generators_are_non_empty() {
        for g in REGISTRY {
            assert!(!g.name.is_empty(), "empty name in registry");
            assert!(!g.generator.is_empty(), "empty generator for {}", g.name);
            assert!(
                !g.description.is_empty(),
                "empty description for {}",
                g.name
            );
        }
    }

    // ── find() ──────────────────────────────────────────────────────────

    #[test]
    fn find_existing_generator() {
        let info = find("go").expect("go should exist");
        assert_eq!(info.name, "go");
        assert_eq!(info.generator, "go");
        assert_eq!(info.category, Category::Sdk);
    }

    #[test]
    fn find_rust_axum_server() {
        let info = find("rust-axum").expect("rust-axum should exist");
        assert_eq!(info.category, Category::Server);
        assert_eq!(info.generator, "rust-axum");
    }

    #[test]
    fn find_terraform_iac() {
        let info = find("terraform").expect("terraform should exist");
        assert_eq!(info.category, Category::Iac);
    }

    #[test]
    fn find_mcp_rust() {
        let info = find("mcp-rust").expect("mcp-rust should exist");
        assert_eq!(info.category, Category::Mcp);
    }

    #[test]
    fn find_nonexistent_returns_none() {
        assert!(find("nonexistent-generator").is_none());
    }

    #[test]
    fn find_empty_string_returns_none() {
        assert!(find("").is_none());
    }

    // ── by_category() ───────────────────────────────────────────────────

    #[test]
    fn by_category_sdk_returns_only_sdks() {
        let sdks = by_category(Category::Sdk);
        assert!(!sdks.is_empty());
        for g in &sdks {
            assert_eq!(g.category, Category::Sdk);
        }
    }

    #[test]
    fn by_category_server_returns_only_servers() {
        let servers = by_category(Category::Server);
        assert!(!servers.is_empty());
        for g in &servers {
            assert_eq!(g.category, Category::Server);
        }
    }

    #[test]
    fn by_category_iac_returns_only_iac() {
        let iac = by_category(Category::Iac);
        assert!(!iac.is_empty());
        for g in &iac {
            assert_eq!(g.category, Category::Iac);
        }
    }

    #[test]
    fn by_category_schema_returns_only_schemas() {
        let schemas = by_category(Category::Schema);
        assert!(!schemas.is_empty());
        for g in &schemas {
            assert_eq!(g.category, Category::Schema);
        }
    }

    #[test]
    fn by_category_doc_returns_only_docs() {
        let docs = by_category(Category::Doc);
        assert!(!docs.is_empty());
        for g in &docs {
            assert_eq!(g.category, Category::Doc);
        }
    }

    #[test]
    fn by_category_helm_returns_only_helm() {
        let helm = by_category(Category::Helm);
        assert!(!helm.is_empty());
        for g in &helm {
            assert_eq!(g.category, Category::Helm);
        }
    }

    #[test]
    fn by_category_mcp_returns_only_mcp() {
        let mcp = by_category(Category::Mcp);
        assert!(!mcp.is_empty());
        for g in &mcp {
            assert_eq!(g.category, Category::Mcp);
        }
    }

    #[test]
    fn registry_contains_expected_completion_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Completion)
            .map(|g| g.name)
            .collect();

        for expected in ["skim-tab", "fish"] {
            assert!(
                names.contains(&expected),
                "Completion registry missing {expected}"
            );
        }
    }

    #[test]
    fn by_category_completion_returns_only_completion() {
        let completion = by_category(Category::Completion);
        assert!(!completion.is_empty());
        for g in &completion {
            assert_eq!(g.category, Category::Completion);
        }
    }

    #[test]
    fn by_category_counts_are_consistent() {
        let total: usize = [
            by_category(Category::Sdk).len(),
            by_category(Category::Server).len(),
            by_category(Category::Schema).len(),
            by_category(Category::Doc).len(),
            by_category(Category::Iac).len(),
            by_category(Category::Helm).len(),
            by_category(Category::Mcp).len(),
            by_category(Category::Completion).len(),
        ]
        .iter()
        .sum();

        assert_eq!(total, REGISTRY.len());
    }

    // ── names_for_category() ────────────────────────────────────────────

    #[test]
    fn names_for_category_sdk() {
        let names = names_for_category(Category::Sdk);
        assert!(!names.is_empty());
        assert!(names.contains(&"go"));
        assert!(names.contains(&"python"));
        assert!(names.contains(&"rust"));
    }

    #[test]
    fn names_for_category_iac() {
        let names = names_for_category(Category::Iac);
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"terraform"));
        assert!(names.contains(&"steampipe"));
    }

    #[test]
    fn names_for_category_matches_by_category_length() {
        for cat in [
            Category::Sdk,
            Category::Server,
            Category::Schema,
            Category::Doc,
            Category::Iac,
            Category::Helm,
            Category::Mcp,
            Category::Completion,
        ] {
            assert_eq!(
                names_for_category(cat).len(),
                by_category(cat).len(),
                "length mismatch for {cat}"
            );
        }
    }

    // ── Category Display ────────────────────────────────────────────────

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", Category::Sdk), "SDK");
        assert_eq!(format!("{}", Category::Server), "Server");
        assert_eq!(format!("{}", Category::Schema), "Schema");
        assert_eq!(format!("{}", Category::Doc), "Doc");
        assert_eq!(format!("{}", Category::Iac), "IaC");
        assert_eq!(format!("{}", Category::Helm), "Helm");
        assert_eq!(format!("{}", Category::Mcp), "MCP");
        assert_eq!(format!("{}", Category::Completion), "Completion");
    }
}
