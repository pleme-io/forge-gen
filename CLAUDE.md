# Forge Gen — Unified Code Generator from OpenAPI Specs

## Build & Test

```bash
cargo build
cargo run -- generate --spec openapi.yaml --sdks rust,go --mcp mcp-rust
cargo run -- list   # show all registered generators
```

## Architecture

Unified CLI orchestrating multiple code generation backends from a single OpenAPI spec.
Invokes external tools (openapi-generator-cli, iac-forge, mcp-forge) in parallel.

### Generator Categories

| Category | Backend Tool | Generators |
|----------|-------------|------------|
| **SDK** | openapi-generator-cli | go, python, javascript, typescript, java, ruby, csharp, rust, kotlin, swift, dart, php, perl, elixir (14 targets) |
| **Server** | openapi-generator-cli | go-server, python-flask, rust-server, spring, nodejs-express, aspnetcore (6 targets) |
| **Schema** | openapi-generator-cli | protobuf-schema, avro-schema, graphql-schema, json-schema (4 targets) |
| **Doc** | openapi-generator-cli | openapi, openapi-yaml, html2, asciidoc, markdown (5 targets) |
| **IaC** | iac-forge | terraform, pulumi, crossplane, ansible, pangea, steampipe (6 targets) |
| **MCP** | mcp-forge | mcp-rust (Rust MCP server with rmcp 0.15) |

### Modules

| Module | Purpose |
|--------|---------|
| `commands/generate.rs` | Generate command: parallel task dispatch to external tools |
| `commands/list.rs` | List all registered generators with categories |
| `manifest.rs` | `forge-gen.toml` manifest loading + CLI merge |
| `registry.rs` | Static generator registry (name, generator ID, category) |

### CLI Flags

```
--spec <path>       OpenAPI spec (YAML or JSON)
--output <dir>      Output directory (default: ./generated)
--sdks <list>       Comma-separated SDK targets or "all"
--servers <list>    Comma-separated server targets or "all"
--iac <list>        Comma-separated IaC backends or "all"
--mcp <list>        Comma-separated MCP targets or "all"
--mcp-name <name>   Project name for MCP generation
--schemas <list>    Comma-separated schema targets or "all"
--docs <list>       Comma-separated doc targets or "all"
--manifest <path>   Path to forge-gen.toml (default: ./forge-gen.toml)
--parallel          Run generators in parallel (default: true)
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
```

## Design Decisions

- **External tool dispatch** — does NOT embed generators; invokes `openapi-generator-cli`, `iac-forge`, `mcp-forge` as subprocesses
- **Parallel by default** — JoinSet-based concurrent execution
- **Registry is static** — all generators known at compile time
- **Nix build** — substrate `rust-tool-release-flake.nix` pattern
