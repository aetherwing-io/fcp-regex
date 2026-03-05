//! fcp-regex-core — Core regex fragment composition library.
//!
//! Pure domain logic for building regex patterns from named fragments.
//! No MCP, no server, no I/O dependencies.
//!
//! # Quick start
//! ```
//! use fcp_regex_core::execute_ops;
//!
//! let results = execute_ops(&[
//!     "define digits any:digit+",
//!     "define version digits lit:. digits lit:. digits",
//!     "compile version anchored:true",
//! ]);
//! assert!(results[2].contains(r"^\d+\.\d+\.\d+$"));
//! ```

pub mod domain;
pub mod elements;
pub mod library;
pub mod parse;

// Re-export key types for convenience
pub use domain::compiler::{compile, CompileResult};
pub use domain::model::{Fragment, FragmentRegistry, RegexEvent};
pub use domain::mutation;
pub use domain::query;
pub use parse::{parse_op, ParsedOp};

/// Execute a sequence of FCP regex operations, returning one result string per op.
///
/// Creates an ephemeral FragmentRegistry, processes each op string, and returns
/// the output text. This is the main entry point for embedded (non-MCP) usage.
pub fn execute_ops(ops: &[&str]) -> Vec<String> {
    let mut registry = FragmentRegistry::new();
    let mut results = Vec::with_capacity(ops.len());

    for op_str in ops {
        let result = execute_single_op(op_str, &mut registry);
        results.push(result);
    }

    results
}

/// Execute a single op string against a registry.
fn execute_single_op(op_str: &str, registry: &mut FragmentRegistry) -> String {
    let op = match parse::parse_op(op_str) {
        Ok(op) => op,
        Err(e) => return format!("ERROR: {e}"),
    };

    match op.verb.as_str() {
        "define" => {
            let (msg, _event) = mutation::handle_define(&op, registry);
            msg
        }
        "from" => {
            let (msg, _event) = mutation::handle_from(&op, registry);
            msg
        }
        "compile" => {
            let (msg, _event) = mutation::handle_compile(&op, registry);
            msg
        }
        "drop" => {
            let (msg, _event) = mutation::handle_drop(&op, registry);
            msg
        }
        "rename" => {
            let (msg, _event) = mutation::handle_rename(&op, registry);
            msg
        }
        // Query verbs
        "show" | "describe" | "test" | "explain" | "list" | "get" | "map" | "stats" | "status" | "history" => {
            query::handle_query(op_str.trim(), registry)
        }
        _ => {
            let known = ["define", "from", "compile", "drop", "rename", "show", "test", "list", "get"];
            let msg = format!("ERROR: unknown verb {:?}", op.verb);
            match parse::suggest(&op.verb, &known) {
                Some(s) => format!("{msg}\n  try: {s}"),
                None => msg,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_ops_define_and_compile() {
        let results = execute_ops(&[
            "define digits any:digit+",
            "compile digits",
        ]);
        assert!(results[0].starts_with("+ fragment"));
        assert!(results[1].contains(r"\d+"));
    }

    #[test]
    fn test_execute_ops_anchored() {
        let results = execute_ops(&[
            "define digits any:digit+",
            "define version digits lit:. digits lit:. digits",
            "compile version anchored:true",
        ]);
        assert!(results[2].contains(r"^\d+\.\d+\.\d+$"), "got: {}", results[2]);
    }

    #[test]
    fn test_execute_ops_from_library() {
        let results = execute_ops(&[
            "from semver",
            "compile semver",
        ]);
        assert!(results[0].contains("imported"));
        assert!(results[1].starts_with("= semver:"));
    }

    #[test]
    fn test_execute_ops_query() {
        let results = execute_ops(&[
            "define digits any:digit+",
            "list",
        ]);
        assert!(results[1].contains("digits"));
    }

    #[test]
    fn test_execute_ops_error() {
        let results = execute_ops(&[
            "compile nonexistent",
        ]);
        assert!(results[0].starts_with("ERROR:"));
    }

    #[test]
    fn test_execute_ops_unknown_verb() {
        let results = execute_ops(&["foobar"]);
        assert!(results[0].contains("unknown verb"));
    }

    #[test]
    fn test_execute_ops_empty() {
        let results = execute_ops(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_execute_ops_drop_and_rename() {
        let results = execute_ops(&[
            "define digits any:digit+",
            "define nums any:digit+",
            "drop nums",
            "rename digits d",
            "compile d",
        ]);
        assert!(results[2].contains("dropped"));
        assert!(results[3].contains("renamed"));
        assert!(results[4].contains(r"\d+"));
    }
}
