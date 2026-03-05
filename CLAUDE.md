# fcp-regex

## Project Overview
MCP server that lets LLMs build regexes via named fragment composition.
Rust implementation using regex-syntax crate for compilation (Tier 2 — native library).

## Architecture
4-layer architecture:
1. **MCP Server (Intent Layer)** — `src/mcp/server.rs` — Registers 4 MCP tools via rmcp, dispatches to domain
2. **Domain (Verbs)** — `src/domain/` — Fragment registry, define/compile/from handlers, query handlers
3. **Elements (DSL Parser)** — `src/elements/` — Parse element syntax (lit:, any:, cap:, etc.)
4. **FCP Core** — `src/fcpcore/` — Tokenizer, parsed-op, verb registry, event log, session, formatter

## Key Directories
- `src/mcp/` — MCP server setup, tool handlers
- `src/domain/` — Domain: model (fragment registry), verbs, query, mutation, format
- `src/elements/` — Element parser: literal, class, quantifier, reference, composition
- `src/fcpcore/` — Shared FCP framework (Rust port of fcp-core)
- `src/library/` — Curated pattern library (~55 patterns, embedded data)

## Commands
- `cargo test` — Run all tests
- `cargo build` — Build debug binary
- `cargo build --release` — Build release binary
- `cargo clippy -- -D warnings` — Lint check
- `make test` / `make build` / `make release` — via Makefile

## Conventions
- Rust 2021 edition, standard library style
- `rmcp` for MCP protocol (stdio transport)
- `regex-syntax` crate for regex AST construction and flavor translation
- Tests colocated with source (`#[cfg(test)]` modules)
- Integration tests in `tests/`
