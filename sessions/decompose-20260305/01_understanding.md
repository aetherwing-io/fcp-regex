# Understanding: fcp-regex Implementation

## Goal
Build an MCP server (Rust binary) that lets LLMs construct regexes via named fragment composition. Standard FCP 4-tool architecture.

## Components
1. **FCP Core (Rust port)** — tokenizer, parsed_op, verb_registry, event_log, session, formatter (copy from fcp-rust/src/fcpcore/)
2. **Element Parser** — parse DSL elements: lit:, any:, none:, chars:, not:, opt:, alt:, cap:, sep:, raw:, and bare name refs
3. **Fragment Registry** — HashMap<String, Fragment> session state with event log integration
4. **Compiler** — walk fragment graph → regex-syntax AST → string output, with flavor translation
5. **Pattern Library** — ~55 curated patterns as embedded Rust data (structs, not strings)
6. **MCP Server** — 4 tools (regex, regex_query, regex_session, regex_help) via rmcp
7. **Domain Verbs** — mutation: define/from/compile/drop/rename; query: show/test/explain/list/get/map/stats/status/history
8. **Reference Card** — embedded in regex tool description, returned by regex_help

## Dependencies (what must exist first)
- FCP Core module (everything depends on tokenizer + parsed_op)
- Element parser (mutation verbs depend on it)
- Fragment registry (compiler depends on it)

## Deliverables
- `cargo build` produces working binary
- `cargo test` passes all tests
- 4 MCP tools functional over stdio
- Pattern library with ~55 patterns queryable
- Full reference card embedded in tool description

## No Ambiguities
The spec is complete — all verbs, element syntax, response format, and library categories are defined.
