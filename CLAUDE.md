# Forge Gen — Unified Code Generator from OpenAPI Specs

## Build & Test

```bash
cargo build
cargo test           # 74 tests
cargo run -- generate --spec openapi.yaml --sdks rust,go --mcp mcp-rust
cargo run -- list    # show all registered generators
cargo run -- list --category completion  # filter by category
```

## Architecture

Unified CLI orchestrating multiple code generation backends from a single OpenAPI spec.
Invokes external tools (openapi-generator-cli, iac-forge, mcp-forge, completion-forge) in parallel.

### Generator Categories

| Category | Backend Tool | Generators |
|----------|-------------|------------|
| **SDK** | openapi-generator-cli | go, python, javascript, typescript (+4 variants), java, ruby, csharp, rust, kotlin, swift, dart, php, perl, elixir, scala, haskell, c, cpp, lua, r, ocaml, clojure, elm, powershell, bash (28 targets) |
| **Server** | openapi-generator-cli | go-server, python-fastapi, rust-axum, spring, kotlin-spring (5 targets) |
| **Schema** | openapi-generator-cli | graphql-schema, protobuf-schema, mysql-schema, postgresql-schema (4 targets) |
| **Doc** | openapi-generator-cli | markdown, html, asciidoc, plantuml (4 targets) |
| **IaC** | iac-forge | terraform, pulumi, crossplane, ansible, pangea, steampipe (6 targets) |
| **Helm** | iac-forge | helm (1 target) |
| **MCP** | mcp-forge | mcp-rust (Rust MCP server with rmcp 0.15) |
| **Completion** | completion-forge | skim-tab (YAML), fish (shell completions) |

### Modules

| Module | Purpose |
|--------|---------|
| `commands/generate.rs` | Generate command: parallel task dispatch to external tools |
| `commands/list.rs` | List all registered generators with categories |
| `commands/init.rs` | Create starter `forge-gen.toml` manifest |
| `commands/validate.rs` | Parse and summarize OpenAPI spec |
| `manifest.rs` | `forge-gen.toml` manifest loading + CLI merge |
| `registry.rs` | Static generator registry (63 generators across 8 categories) |

### CLI Flags

```
--spec <path>             OpenAPI spec (YAML or JSON)
--output <dir>            Output directory (default: ./generated)
--sdks <list>             Comma-separated SDK targets or "all"
--servers <list>          Comma-separated server targets or "all"
--iac <list>              Comma-separated IaC backends or "all"
--helm <list>             Comma-separated Helm targets or "all"
--mcp <list>              Comma-separated MCP targets or "all"
--mcp-name <name>         Project name for MCP generation
--completions <list>      Comma-separated completion formats or "all"
--completion-name <name>  CLI command name for completion generation
--schemas <list>          Comma-separated schema targets or "all"
--docs <list>             Comma-separated doc targets or "all"
--manifest <path>         Path to forge-gen.toml (default: ./forge-gen.toml)
--parallel                Run generators in parallel (default: true)
```

### Manifest (`forge-gen.toml`)

```toml
[spec]
path = "openapi.yaml"

[output]
dir = "./generated"

[sdks]
targets = ["go", "rust", "typescript"]

[mcp]
targets = ["mcp-rust"]
name = "my-api"

[iac]
backends = ["terraform", "pulumi"]

[completions]
targets = ["skim-tab", "fish"]
name = "my-tool"
icon = "☁"
grouping = "auto"          # auto, tag, path, or operation-id
aliases = ["mt"]
```

## Design Decisions

- **External tool dispatch** — does NOT embed generators; invokes `openapi-generator-cli`, `iac-forge`, `mcp-forge`, `completion-forge` as subprocesses
- **Parallel by default** — JoinSet-based concurrent execution
- **Registry is static** — all generators known at compile time (63 total across 8 categories)
- **Nix build** — substrate `rust-tool-release-flake.nix` pattern
