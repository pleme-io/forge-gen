use std::fmt;

/// Error returned when a string cannot be parsed into a [`Category`].
#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown category: {input}")]
pub struct ParseCategoryError {
    input: String,
}

/// Category of a code generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Category {
    Sdk,
    Server,
    Schema,
    Doc,
    Iac,
    Helm,
    Mcp,
    Grpc,
    Completion,
}

impl Category {
    /// All known category variants, in canonical order.
    pub const ALL: &[Self] = &[
        Self::Sdk,
        Self::Server,
        Self::Schema,
        Self::Doc,
        Self::Iac,
        Self::Helm,
        Self::Mcp,
        Self::Grpc,
        Self::Completion,
    ];

    /// Lowercase string representation matching the canonical CLI form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::Server => "server",
            Self::Schema => "schema",
            Self::Doc => "doc",
            Self::Iac => "iac",
            Self::Helm => "helm",
            Self::Mcp => "mcp",
            Self::Grpc => "grpc",
            Self::Completion => "completion",
        }
    }
}

impl std::str::FromStr for Category {
    type Err = ParseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sdk" | "sdks" => Ok(Self::Sdk),
            "server" | "servers" => Ok(Self::Server),
            "schema" | "schemas" => Ok(Self::Schema),
            "doc" | "docs" => Ok(Self::Doc),
            "iac" => Ok(Self::Iac),
            "helm" => Ok(Self::Helm),
            "mcp" => Ok(Self::Mcp),
            "grpc" => Ok(Self::Grpc),
            "completion" | "completions" => Ok(Self::Completion),
            _ => Err(ParseCategoryError {
                input: s.to_string(),
            }),
        }
    }
}

impl From<Category> for &'static str {
    fn from(cat: Category) -> Self {
        cat.as_str()
    }
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
            Self::Grpc => write!(f, "gRPC"),
            Self::Completion => write!(f, "Completion"),
        }
    }
}

/// Metadata for a single generator target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
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
/// 52 generators across 9 categories: SDK (28), Server (5), Schema (4),
/// Doc (4), `IaC` (6), Helm (1), MCP (1), gRPC (1), Completion (2).
pub static REGISTRY: &[GeneratorInfo] = &[
    // ── Client SDKs ──────────────────────────────────────────────────
    GeneratorInfo {
        name: "go",
        generator: "go",
        category: Category::Sdk,
        description: "Go client SDK",
    },
    GeneratorInfo {
        name: "python",
        generator: "python",
        category: Category::Sdk,
        description: "Python client SDK",
    },
    GeneratorInfo {
        name: "javascript",
        generator: "javascript",
        category: Category::Sdk,
        description: "JavaScript client SDK",
    },
    GeneratorInfo {
        name: "typescript",
        generator: "typescript-fetch",
        category: Category::Sdk,
        description: "TypeScript client (fetch)",
    },
    GeneratorInfo {
        name: "typescript-axios",
        generator: "typescript-axios",
        category: Category::Sdk,
        description: "TypeScript client (Axios)",
    },
    GeneratorInfo {
        name: "typescript-node",
        generator: "typescript-node",
        category: Category::Sdk,
        description: "TypeScript client (Node.js)",
    },
    GeneratorInfo {
        name: "typescript-angular",
        generator: "typescript-angular",
        category: Category::Sdk,
        description: "TypeScript client (Angular)",
    },
    GeneratorInfo {
        name: "java",
        generator: "java",
        category: Category::Sdk,
        description: "Java client SDK",
    },
    GeneratorInfo {
        name: "ruby",
        generator: "ruby",
        category: Category::Sdk,
        description: "Ruby client SDK",
    },
    GeneratorInfo {
        name: "csharp",
        generator: "csharp",
        category: Category::Sdk,
        description: "C# client SDK (.NET 6)",
    },
    GeneratorInfo {
        name: "rust",
        generator: "rust",
        category: Category::Sdk,
        description: "Rust client SDK (reqwest)",
    },
    GeneratorInfo {
        name: "kotlin",
        generator: "kotlin",
        category: Category::Sdk,
        description: "Kotlin client SDK",
    },
    GeneratorInfo {
        name: "swift",
        generator: "swift6",
        category: Category::Sdk,
        description: "Swift 6 client SDK",
    },
    GeneratorInfo {
        name: "dart",
        generator: "dart",
        category: Category::Sdk,
        description: "Dart client SDK",
    },
    GeneratorInfo {
        name: "php",
        generator: "php",
        category: Category::Sdk,
        description: "PHP client SDK",
    },
    GeneratorInfo {
        name: "perl",
        generator: "perl",
        category: Category::Sdk,
        description: "Perl client SDK",
    },
    GeneratorInfo {
        name: "elixir",
        generator: "elixir",
        category: Category::Sdk,
        description: "Elixir client SDK",
    },
    GeneratorInfo {
        name: "scala",
        generator: "scala-sttp",
        category: Category::Sdk,
        description: "Scala client SDK (sttp)",
    },
    GeneratorInfo {
        name: "haskell",
        generator: "haskell-http-client",
        category: Category::Sdk,
        description: "Haskell client SDK",
    },
    GeneratorInfo {
        name: "c",
        generator: "c",
        category: Category::Sdk,
        description: "C client SDK",
    },
    GeneratorInfo {
        name: "cpp",
        generator: "cpp-restsdk",
        category: Category::Sdk,
        description: "C++ client SDK (restsdk)",
    },
    GeneratorInfo {
        name: "lua",
        generator: "lua",
        category: Category::Sdk,
        description: "Lua client SDK",
    },
    GeneratorInfo {
        name: "r",
        generator: "r",
        category: Category::Sdk,
        description: "R client SDK",
    },
    GeneratorInfo {
        name: "ocaml",
        generator: "ocaml",
        category: Category::Sdk,
        description: "OCaml client SDK",
    },
    GeneratorInfo {
        name: "clojure",
        generator: "clojure",
        category: Category::Sdk,
        description: "Clojure client SDK",
    },
    GeneratorInfo {
        name: "elm",
        generator: "elm",
        category: Category::Sdk,
        description: "Elm client SDK",
    },
    GeneratorInfo {
        name: "powershell",
        generator: "powershell",
        category: Category::Sdk,
        description: "PowerShell client SDK",
    },
    GeneratorInfo {
        name: "bash",
        generator: "bash",
        category: Category::Sdk,
        description: "Bash client SDK (curl)",
    },
    // ── Server stubs ─────────────────────────────────────────────────
    GeneratorInfo {
        name: "go-server",
        generator: "go-server",
        category: Category::Server,
        description: "Go server stub",
    },
    GeneratorInfo {
        name: "python-fastapi",
        generator: "python-fastapi",
        category: Category::Server,
        description: "Python FastAPI server stub",
    },
    GeneratorInfo {
        name: "rust-axum",
        generator: "rust-axum",
        category: Category::Server,
        description: "Rust Axum server stub",
    },
    GeneratorInfo {
        name: "spring",
        generator: "spring",
        category: Category::Server,
        description: "Java Spring Boot server stub",
    },
    GeneratorInfo {
        name: "kotlin-spring",
        generator: "kotlin-spring",
        category: Category::Server,
        description: "Kotlin Spring server stub",
    },
    // ── Schema generators ────────────────────────────────────────────
    GeneratorInfo {
        name: "graphql-schema",
        generator: "graphql-schema",
        category: Category::Schema,
        description: "GraphQL schema from OpenAPI",
    },
    GeneratorInfo {
        name: "protobuf-schema",
        generator: "protobuf-schema",
        category: Category::Schema,
        description: "Protobuf schema from OpenAPI",
    },
    GeneratorInfo {
        name: "mysql-schema",
        generator: "mysql-schema",
        category: Category::Schema,
        description: "MySQL DDL from OpenAPI models",
    },
    GeneratorInfo {
        name: "postgresql-schema",
        generator: "postgresql-schema",
        category: Category::Schema,
        description: "PostgreSQL DDL from OpenAPI models",
    },
    // ── Documentation generators ─────────────────────────────────────
    GeneratorInfo {
        name: "markdown",
        generator: "markdown",
        category: Category::Doc,
        description: "Markdown API documentation",
    },
    GeneratorInfo {
        name: "html",
        generator: "html2",
        category: Category::Doc,
        description: "HTML API documentation",
    },
    GeneratorInfo {
        name: "asciidoc",
        generator: "asciidoc",
        category: Category::Doc,
        description: "AsciiDoc API documentation",
    },
    GeneratorInfo {
        name: "plantuml",
        generator: "plantuml",
        category: Category::Doc,
        description: "PlantUML diagrams from OpenAPI",
    },
    // ── IaC backends (via iac-forge) ─────────────────────────────────
    GeneratorInfo {
        name: "terraform",
        generator: "terraform",
        category: Category::Iac,
        description: "Terraform provider (Go, plugin-framework)",
    },
    GeneratorInfo {
        name: "pulumi",
        generator: "pulumi",
        category: Category::Iac,
        description: "Pulumi provider (schema.json)",
    },
    GeneratorInfo {
        name: "crossplane",
        generator: "crossplane",
        category: Category::Iac,
        description: "Crossplane provider (CRD YAML)",
    },
    GeneratorInfo {
        name: "ansible",
        generator: "ansible",
        category: Category::Iac,
        description: "Ansible collection (Python modules)",
    },
    GeneratorInfo {
        name: "pangea",
        generator: "pangea",
        category: Category::Iac,
        description: "Pangea DSL resources (Ruby)",
    },
    GeneratorInfo {
        name: "steampipe",
        generator: "steampipe",
        category: Category::Iac,
        description: "Steampipe plugin tables (Go)",
    },
    // ── Helm chart backends (via helm-forge / iac-forge) ──────────────
    GeneratorInfo {
        name: "helm",
        generator: "helm",
        category: Category::Helm,
        description: "Helm charts from resource specs (pleme-lib)",
    },
    // ── MCP server backends (via mcp-forge) ─────────────────────────
    GeneratorInfo {
        name: "mcp-rust",
        generator: "mcp-rust",
        category: Category::Mcp,
        description: "Rust MCP server (rmcp 0.15, CLI + stdio)",
    },
    // ── gRPC server backends (via grpc-forge) ───────────────────────
    GeneratorInfo {
        name: "grpc-rust",
        generator: "grpc-rust",
        category: Category::Grpc,
        description: "Rust tonic gRPC server (typed proto3 + scaffold)",
    },
    // ── Completion generators (via completion-forge) ────────────────
    GeneratorInfo {
        name: "skim-tab",
        generator: "skim-tab",
        category: Category::Completion,
        description: "skim-tab YAML completion spec",
    },
    GeneratorInfo {
        name: "fish",
        generator: "fish",
        category: Category::Completion,
        description: "Fish shell completion file",
    },
];

/// Look up a generator by its friendly name.
#[must_use]
pub fn find(name: &str) -> Option<&'static GeneratorInfo> {
    REGISTRY.iter().find(|g| g.name == name)
}

/// Return all generators for a given category.
#[must_use]
pub fn by_category(category: Category) -> Vec<&'static GeneratorInfo> {
    REGISTRY.iter().filter(|g| g.category == category).collect()
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

        for expected in [
            "go",
            "python",
            "rust",
            "java",
            "typescript",
            "ruby",
            "csharp",
        ] {
            assert!(names.contains(&expected), "SDK registry missing {expected}");
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
            assert!(names.contains(&expected), "IaC registry missing {expected}");
        }
    }

    #[test]
    fn registry_contains_expected_schema_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Schema)
            .map(|g| g.name)
            .collect();

        for expected in [
            "graphql-schema",
            "protobuf-schema",
            "mysql-schema",
            "postgresql-schema",
        ] {
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
            assert!(names.contains(&expected), "Doc registry missing {expected}");
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

        assert!(names.contains(&"mcp-rust"), "MCP registry missing mcp-rust");
    }

    #[test]
    fn registry_contains_expected_grpc_generators() {
        let names: Vec<&str> = REGISTRY
            .iter()
            .filter(|g| g.category == Category::Grpc)
            .map(|g| g.name)
            .collect();

        assert!(
            names.contains(&"grpc-rust"),
            "gRPC registry missing grpc-rust"
        );
    }

    #[test]
    fn registry_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for g in REGISTRY {
            assert!(seen.insert(g.name), "duplicate registry name: {}", g.name);
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
    fn find_grpc_rust() {
        let info = find("grpc-rust").expect("grpc-rust should exist");
        assert_eq!(info.category, Category::Grpc);
        assert_eq!(info.generator, "grpc-rust");
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
    fn by_category_grpc_returns_only_grpc() {
        let grpc = by_category(Category::Grpc);
        assert!(!grpc.is_empty());
        for g in &grpc {
            assert_eq!(g.category, Category::Grpc);
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
        let total: usize = Category::ALL
            .iter()
            .map(|cat| by_category(*cat).len())
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
        for &cat in Category::ALL {
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
        assert_eq!(format!("{}", Category::Grpc), "gRPC");
        assert_eq!(format!("{}", Category::Completion), "Completion");
    }

    // ── Category FromStr ─────────────────────────────────────────────

    #[test]
    fn category_from_str() {
        assert_eq!("sdk".parse::<Category>().unwrap(), Category::Sdk);
        assert_eq!("servers".parse::<Category>().unwrap(), Category::Server);
        assert_eq!(
            "completion".parse::<Category>().unwrap(),
            Category::Completion
        );
        assert!("nonexistent".parse::<Category>().is_err());
    }

    // ── Category FromStr: exhaustive variant + alias coverage ────────────

    #[test]
    fn category_from_str_all_singular_forms() {
        assert_eq!("sdk".parse::<Category>().unwrap(), Category::Sdk);
        assert_eq!("server".parse::<Category>().unwrap(), Category::Server);
        assert_eq!("schema".parse::<Category>().unwrap(), Category::Schema);
        assert_eq!("doc".parse::<Category>().unwrap(), Category::Doc);
        assert_eq!("iac".parse::<Category>().unwrap(), Category::Iac);
        assert_eq!("helm".parse::<Category>().unwrap(), Category::Helm);
        assert_eq!("mcp".parse::<Category>().unwrap(), Category::Mcp);
        assert_eq!("grpc".parse::<Category>().unwrap(), Category::Grpc);
        assert_eq!(
            "completion".parse::<Category>().unwrap(),
            Category::Completion
        );
    }

    #[test]
    fn category_from_str_all_plural_forms() {
        assert_eq!("sdks".parse::<Category>().unwrap(), Category::Sdk);
        assert_eq!("servers".parse::<Category>().unwrap(), Category::Server);
        assert_eq!("schemas".parse::<Category>().unwrap(), Category::Schema);
        assert_eq!("docs".parse::<Category>().unwrap(), Category::Doc);
        assert_eq!(
            "completions".parse::<Category>().unwrap(),
            Category::Completion
        );
    }

    #[test]
    fn category_from_str_case_insensitive() {
        assert_eq!("SDK".parse::<Category>().unwrap(), Category::Sdk);
        assert_eq!("IAC".parse::<Category>().unwrap(), Category::Iac);
        assert_eq!("Helm".parse::<Category>().unwrap(), Category::Helm);
        assert_eq!("MCP".parse::<Category>().unwrap(), Category::Mcp);
        assert_eq!("SCHEMAS".parse::<Category>().unwrap(), Category::Schema);
        assert_eq!("DOCS".parse::<Category>().unwrap(), Category::Doc);
    }

    #[test]
    fn category_from_str_error_message_contains_input() {
        let err = "bogus".parse::<Category>().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("bogus"),
            "error message should include the invalid input, got: {msg}"
        );
    }

    #[test]
    fn category_from_str_empty_string_errors() {
        assert!("".parse::<Category>().is_err());
    }

    #[test]
    fn category_from_str_whitespace_errors() {
        assert!(" sdk ".parse::<Category>().is_err());
    }

    // ── find() edge cases ───────────────────────────────────────────────

    #[test]
    fn find_is_case_sensitive() {
        assert!(find("Go").is_none(), "find should be case-sensitive");
        assert!(find("PYTHON").is_none());
    }

    #[test]
    fn find_every_registry_entry() {
        for g in REGISTRY {
            assert!(
                find(g.name).is_some(),
                "find({}) should return Some",
                g.name
            );
        }
    }

    // ── by_category completeness ────────────────────────────────────────

    #[test]
    fn sdk_count_is_28() {
        assert_eq!(by_category(Category::Sdk).len(), 28);
    }

    #[test]
    fn server_count_is_5() {
        assert_eq!(by_category(Category::Server).len(), 5);
    }

    #[test]
    fn schema_count_is_4() {
        assert_eq!(by_category(Category::Schema).len(), 4);
    }

    #[test]
    fn doc_count_is_4() {
        assert_eq!(by_category(Category::Doc).len(), 4);
    }

    #[test]
    fn iac_count_is_6() {
        assert_eq!(by_category(Category::Iac).len(), 6);
    }

    #[test]
    fn helm_count_is_1() {
        assert_eq!(by_category(Category::Helm).len(), 1);
    }

    #[test]
    fn mcp_count_is_1() {
        assert_eq!(by_category(Category::Mcp).len(), 1);
    }

    #[test]
    fn grpc_count_is_1() {
        assert_eq!(by_category(Category::Grpc).len(), 1);
    }

    #[test]
    fn completion_count_is_2() {
        assert_eq!(by_category(Category::Completion).len(), 2);
    }

    #[test]
    fn total_registry_count_is_52() {
        assert_eq!(REGISTRY.len(), 52);
    }

    // ── GeneratorInfo field correctness: spot-check representative entries ──

    #[test]
    fn swift_generator_uses_swift6() {
        let info = find("swift").unwrap();
        assert_eq!(info.generator, "swift6");
    }

    #[test]
    fn typescript_generator_uses_fetch() {
        let info = find("typescript").unwrap();
        assert_eq!(info.generator, "typescript-fetch");
    }

    #[test]
    fn html_generator_uses_html2() {
        let info = find("html").unwrap();
        assert_eq!(info.generator, "html2");
    }

    #[test]
    fn scala_generator_uses_sttp() {
        let info = find("scala").unwrap();
        assert_eq!(info.generator, "scala-sttp");
    }

    #[test]
    fn haskell_generator_uses_http_client() {
        let info = find("haskell").unwrap();
        assert_eq!(info.generator, "haskell-http-client");
    }

    #[test]
    fn cpp_generator_uses_restsdk() {
        let info = find("cpp").unwrap();
        assert_eq!(info.generator, "cpp-restsdk");
    }

    // ── names_for_category edge cases ───────────────────────────────────

    #[test]
    fn names_for_category_helm_returns_single_entry() {
        let names = names_for_category(Category::Helm);
        assert_eq!(names, vec!["helm"]);
    }

    #[test]
    fn names_for_category_mcp_returns_single_entry() {
        let names = names_for_category(Category::Mcp);
        assert_eq!(names, vec!["mcp-rust"]);
    }

    #[test]
    fn names_for_category_grpc_returns_single_entry() {
        let names = names_for_category(Category::Grpc);
        assert_eq!(names, vec!["grpc-rust"]);
    }

    #[test]
    fn names_for_category_completion() {
        let names = names_for_category(Category::Completion);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"skim-tab"));
        assert!(names.contains(&"fish"));
    }

    // ── ParseCategoryError ──────────────────────────────────────────────

    #[test]
    fn parse_category_error_is_typed() {
        let err = "nope".parse::<Category>().unwrap_err();
        assert_eq!(err.to_string(), "unknown category: nope");
    }

    #[test]
    fn parse_category_error_implements_std_error() {
        let err = "bad".parse::<Category>().unwrap_err();
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn parse_category_error_debug_includes_input() {
        let err = "xyz".parse::<Category>().unwrap_err();
        let debug = format!("{err:?}");
        assert!(debug.contains("xyz"));
    }

    // ── Category round-trip ─────────────────────────────────────────────

    #[test]
    fn category_display_roundtrip_lowercase_parses_back() {
        for &cat in Category::ALL {
            let displayed = cat.to_string().to_lowercase();
            let parsed: Category = displayed.parse().unwrap_or_else(|e| {
                panic!("failed to round-trip {cat}: {e}");
            });
            assert_eq!(parsed, cat, "round-trip failed for {cat}");
        }
    }

    #[test]
    fn category_all_contains_every_variant() {
        assert_eq!(Category::ALL.len(), 9);
        assert!(Category::ALL.contains(&Category::Sdk));
        assert!(Category::ALL.contains(&Category::Grpc));
        assert!(Category::ALL.contains(&Category::Completion));
    }

    #[test]
    fn category_is_hashable() {
        let mut set = std::collections::HashSet::new();
        set.insert(Category::Sdk);
        set.insert(Category::Sdk);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn category_as_str_roundtrips_through_from_str() {
        for &cat in Category::ALL {
            let s = cat.as_str();
            let parsed: Category = s.parse().unwrap();
            assert_eq!(parsed, cat, "as_str/FromStr round-trip failed for {cat}");
        }
    }

    #[test]
    fn category_as_str_values() {
        assert_eq!(Category::Sdk.as_str(), "sdk");
        assert_eq!(Category::Iac.as_str(), "iac");
        assert_eq!(Category::Completion.as_str(), "completion");
    }

    #[test]
    fn category_into_static_str() {
        let s: &'static str = Category::Helm.into();
        assert_eq!(s, "helm");
    }
}
