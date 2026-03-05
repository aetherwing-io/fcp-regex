# fcp-regex

MCP server for regex construction via named fragment composition.

## What It Does

LLMs understand regex semantics perfectly — they know what they want to match. But they consistently fail at regex syntax: unbalanced groups, unescaped metacharacters, broken alternations. fcp-regex solves this by letting LLMs build regexes through named fragments with a flat element syntax. No raw regex, no paren balancing, no escaping burden. Built on the [FCP](https://github.com/aetherwing-io/fcp) framework.

Written in Rust using [rmcp](https://github.com/anthropics/rmcp) for MCP transport and [regex-syntax](https://docs.rs/regex-syntax) for compilation.

## Quick Example

```
regex_session('new "Semver Parser" flavor:pcre')

regex([
  "define digits any:digit+",
  "define version digits lit:. digits lit:. digits",
  "define prerelease lit:- chars:a-zA-Z0-9-.+",
  "define semver version opt:prerelease",
  "compile semver anchored:true"
])
```

Produces: `^\d+\.\d+\.\d+(?:-[a-zA-Z0-9\-.]+)?$`

```
regex_query('test semver against:1.2.3-alpha.1')
→ MATCH

regex_query('test semver against:v1.2')
→ NO MATCH
```

### Available MCP Tools

| Tool | Purpose |
|------|---------|
| `regex(ops)` | Batch mutations — define fragments, import patterns, compile to regex |
| `regex_query(q)` | Inspect — show, test, explain, list fragments and library |
| `regex_session(action)` | Lifecycle — new, close, status, undo, redo |
| `regex_help()` | Full reference card |

### Verb Reference — Mutations

| Verb | Syntax |
|------|--------|
| `define` | `define NAME ELEMENT [ELEMENT...]` |
| `from` | `from SOURCE [as:ALIAS]` |
| `compile` | `compile NAME [flavor:F] [anchored:bool]` |
| `drop` | `drop NAME` |
| `rename` | `rename OLD NEW` |

### Verb Reference — Queries

| Verb | Syntax |
|------|--------|
| `show` | `show NAME` — fragment tree + compiled regex |
| `test` | `test NAME against:STRING` — test match |
| `list` | `list` — all fragments |
| `list library` | `list library [category:NAME]` — pattern library |
| `get` | `get PATTERN` — library pattern detail |
| `map` | `map` — fragment + library overview |
| `stats` | `stats` — session statistics |

### Verb Reference — Session

| Verb | Syntax |
|------|--------|
| `new` | `new "Title" [flavor:pcre]` |
| `close` | `close` |
| `status` | `status` |
| `undo` | `undo [to:CHECKPOINT]` |
| `redo` | `redo` |
| `checkpoint` | `checkpoint NAME` |

### Element Syntax

| Element | Meaning | Example |
|---------|---------|---------|
| `<name>` | Reference another fragment | `digits` |
| `lit:<chars>` | Literal (auto-escaped) | `lit:.` `lit:@` |
| `any:<class><quant>` | Character class | `any:digit+` |
| `none:<class><quant>` | Negated class | `none:whitespace+` |
| `chars:<set><quant>` | Custom character set | `chars:a-z0-9+` |
| `not:<set><quant>` | Negated custom set | `not:@/+` |
| `opt:<name>` | Optional | `opt:prerelease` |
| `alt:<a>\|<b>` | Alternation | `alt:ipv4\|ipv6` |
| `cap:<name>` | Capture group | `cap:domain` |
| `cap:<label>/<name>` | Named capture | `cap:port/digits` |
| `sep:<name>/<lit>` | Separated repeat | `sep:octet/lit:.` |
| `raw:<regex>` | Raw regex passthrough | `raw:(?<=\b)` |

**Classes**: `digit` `alpha` `alphanumeric` `word` `whitespace` `any`

**Quantifiers**: `+` (1+) `*` (0+) `?` (0..1) `{N}` `{N,M}` `{N,}`

### Pattern Library

56 curated patterns across 8 categories: uri, email, datetime, identifiers, network, http, common, data.

```
regex(["from semver"])
regex_query('list library')
regex_query('get rfc3986:uri')
```

## Installation

### Build from source

```bash
cargo install --git https://github.com/aetherwing-io/fcp-regex
```

### MCP Client Configuration

```json
{
  "mcpServers": {
    "fcp-regex": {
      "command": "fcp-regex"
    }
  }
}
```

## Architecture

```
Cargo.toml                     Workspace root
├── src/                       MCP server (fcp-regex binary + lib)
│   ├── main.rs                Entry point — MCP stdio + Slipstream bridge
│   ├── bridge.rs              Slipstream daemon connectivity
│   ├── mcp/server.rs          RegexServer (4 rmcp tool handlers)
│   ├── fcpcore/               FCP framework (Rust port)
│   │   ├── tokenizer.rs       DSL tokenizer
│   │   ├── parsed_op.rs       Operation parser
│   │   ├── verb_registry.rs   Verb spec registry
│   │   ├── event_log.rs       Event sourcing (undo/redo)
│   │   ├── session.rs         Session lifecycle
│   │   └── formatter.rs       Response formatter
│   └── reference_card.rs      Embedded reference card
│
└── fcp-regex-core/            Core library (no MCP dependencies)
    └── src/
        ├── elements/           Element parser (lit:, any:, cap:, etc.)
        ├── domain/
        │   ├── model.rs        FragmentRegistry + RegexEvent
        │   ├── compiler.rs     Fragment graph → regex string
        │   ├── mutation.rs     define/from/compile/drop/rename handlers
        │   └── query.rs        show/test/list/get handlers
        └── library/            56 curated patterns (8 categories)
```

## Development

```bash
cargo test          # Run all tests
cargo build         # Build binary
cargo clippy        # Run lints
make help           # Show Makefile targets
```

## License

MIT
