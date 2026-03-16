use std::fmt;

/// Category of a code generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Sdk,
    Server,
    Schema,
    Doc,
    Iac,
    Mcp,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdk => write!(f, "SDK"),
            Self::Server => write!(f, "Server"),
            Self::Schema => write!(f, "Schema"),
            Self::Doc => write!(f, "Doc"),
            Self::Iac => write!(f, "IaC"),
            Self::Mcp => write!(f, "MCP"),
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

    // ── MCP server backends (via mcp-forge) ─────────────────────────
    GeneratorInfo { name: "mcp-rust",            generator: "mcp-rust",               category: Category::Mcp, description: "Rust MCP server (rmcp 0.15, CLI + stdio)" },
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
