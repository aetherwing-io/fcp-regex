# fcp-regex

MCP server for regex construction via named fragment composition. Part of the [FCP](https://github.com/aetherwing-io/fcp) ecosystem.

LLMs understand regex semantics but fail at regex syntax. fcp-regex lets them build regexes by defining named fragments and composing them by reference — no raw regex syntax, no paren balancing, no escaping burden.

## Quick Example

```
regex([
  "define digits any:digit+",
  "define version digits lit:. digits lit:. digits",
  "define prerelease lit:- chars:a-zA-Z0-9-.+",
  "define semver version opt:prerelease",
  "compile semver anchored:true"
])
```

Produces: `^\d+\.\d+\.\d+(?:-[a-zA-Z0-9\-.]+)?$`

## Tools

| Tool | Purpose |
|------|---------|
| `regex(ops: string[])` | Define fragments, import patterns, compile to regex |
| `regex_query(q: string)` | Show, test, explain, list fragments |
| `regex_session(action: string)` | Session lifecycle (new/close/undo/redo) |
| `regex_help()` | Reference card |

## Build

```bash
cargo build
cargo test
```

## License

MIT
