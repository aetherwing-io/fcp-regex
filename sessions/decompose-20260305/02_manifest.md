# Manifest: fcp-regex Implementation

## Items

### M1: FCP Core Module
Copy and adapt from fcp-rust/src/fcpcore/. Files:
- `src/fcpcore/mod.rs` — module exports
- `src/fcpcore/tokenizer.rs` — quote-aware tokenizer (split on whitespace, handle quotes)
- `src/fcpcore/parsed_op.rs` — ParsedOp struct, token classification (verb, positionals, params, selectors)
- `src/fcpcore/verb_registry.rs` — verb name → handler dispatch
- `src/fcpcore/event_log.rs` — cursor-based event log for undo/redo
- `src/fcpcore/session.rs` — session state (model, event_log, file_path), new/close/status/undo/redo/checkpoint
- `src/fcpcore/formatter.rs` — response prefix formatting (+, *, -, =, !)

Tests: unit tests for tokenizer, parsed_op, event_log (port from fcp-rust)

### M2: Element Parser
Parse element syntax from whitespace-delimited tokens within `define` ops.
- `src/elements/mod.rs` — Element enum + parse_element() function
- Element variants:
  - `Ref(name)` — bare name reference to another fragment
  - `Literal(chars)` — `lit:<chars>`, auto-escape regex metacharacters
  - `AnyClass(class, quant)` — `any:<class><quant>`
  - `NoneClass(class, quant)` — `none:<class><quant>`
  - `Chars(set, quant)` — `chars:<set><quant>`
  - `NotChars(set, quant)` — `not:<set><quant>`
  - `Optional(name)` — `opt:<name>`
  - `Alternation(names)` — `alt:<a>|<b>|<c>`
  - `Capture(name)` — `cap:<name>`
  - `NamedCapture(label, name)` — `cap:<label>/<name>`
  - `SepBy(name, lit)` — `sep:<name>/<lit>`
  - `Raw(regex)` — `raw:<regex>`
- Quantifier parsing: `+`, `*`, `?`, `{3}`, `{1,3}`, `{3,}`
- Character class names: digit, alpha, alphanumeric, word, whitespace, any

Tests: parse each element variant, quantifier parsing, error cases

### M3: Fragment Registry (Domain Model)
- `src/domain/model.rs` — Fragment struct, FragmentRegistry (HashMap<String, Fragment>)
- Fragment = { name: String, elements: Vec<Element> }
- FragmentRegistry methods: define, drop, rename, get, list, contains
- Event types: DefineEvent, DropEvent, RenameEvent (for undo/redo)

Tests: CRUD operations, rename updates references, event generation

### M4: Compiler
- `src/domain/compiler.rs` — compile fragment graph → regex string
- Walk fragment references recursively, detect cycles
- Map elements to regex-syntax HIR nodes
- Flavor translation: Python re, PCRE, JS, Go RE2, Rust regex
- `anchored:true` wraps in `^...$`
- Generate example matches/rejects (bounded enumeration via regex crate)

Tests: semver example, userinfo example, cycle detection, flavor output, anchoring

### M5: Mutation Verbs
- `src/domain/mutation.rs` — handlers for define, from, compile, drop, rename
- define: parse positionals as name + elements via M2, store in registry via M3
- from: look up library pattern via M6, store in registry
- compile: invoke compiler via M4, format response
- drop: remove from registry
- rename: update registry + all references

Tests: each verb handler, error cases (unknown fragment, bad element syntax)

### M6: Pattern Library
- `src/library/mod.rs` — LibraryPattern struct, LIBRARY constant, lookup functions
- `src/library/patterns/` — one file per category (uri.rs, email.rs, datetime.rs, identifiers.rs, network.rs, http.rs, common.rs, data.rs)
- LibraryPattern = { name, source, flavor, regex, structure, test_match, test_no_match, flavor_notes, aliases }
- ~55 patterns total across 8 categories
- Lookup by name, by alias, by category
- `list library` returns categories with counts
- `list library category:<name>` returns patterns in category
- `get <pattern>` returns formatted output

Tests: lookup by name, by alias, category listing, pattern count

### M7: Query Verbs
- `src/domain/query.rs` — handlers for show, test, explain, list, get, map, stats, status, describe, history
- show: display fragment tree + compiled regex
- test: compile fragment, run against test string, report match/no-match
- explain: parse raw regex string → suggest equivalent define ops (reverse direction)
- list: all fragments in session
- list library / list library category:<name>: delegate to M6
- get: delegate to M6
- map/stats/status/describe/history: standard FCP queries

Tests: each query handler, explain reverse parsing

### M8: MCP Server + Tool Wiring
- `src/mcp/server.rs` — register 4 tools via rmcp macros
- `src/mcp/mod.rs` — module exports
- `src/main.rs` — tokio runtime, stdio transport, server startup
- Tool handlers:
  - `regex(ops: string[])` → parse each op, dispatch to mutation verbs
  - `regex_query(q: string)` → dispatch to query verbs
  - `regex_session(action: string)` → dispatch to session commands
  - `regex_help()` → return reference card
- Reference card: full verb syntax, element table, response prefix legend

Tests: integration tests in tests/ — full workflows through MCP tools

### M9: Reference Card
- `src/reference_card.rs` — REFERENCE_CARD constant string
- Embedded in regex tool description
- Contains: all verb syntax, element syntax table, character classes, quantifiers, response prefixes, example workflows

### M10: Integration Tests
- `tests/workflow_semver.rs` — full semver build+compile workflow
- `tests/workflow_userinfo.rs` — full userinfo build+compile workflow
- `tests/workflow_library.rs` — import from library, compose, compile
- `tests/workflow_undo_redo.rs` — define, undo, redo, verify state
- `tests/workflow_explain.rs` — explain a raw regex, verify output

## Dependency Order
M1 → M2 → M3 → M4 → M5, M6 (parallel) → M7 → M8 → M9 → M10
