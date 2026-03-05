use crate::elements::parse_element;
use crate::fcpcore::parsed_op::ParsedOp;
use crate::fcpcore::tokenizer::tokenize;
use crate::library;

use super::compiler;
use super::model::{FragmentRegistry, RegexEvent};
use crate::elements::Element;

/// Extract element tokens from a define op's raw string.
/// Skips the verb and name (first two tokens), returns the rest as raw tokens
/// so that element prefixes like `any:digit+` are preserved (the FCP parser
/// would split them into params).
fn extract_element_tokens(raw: &str) -> Vec<String> {
    let tokens = tokenize(raw);
    if tokens.len() > 2 {
        tokens[2..].to_vec()
    } else {
        Vec::new()
    }
}

pub fn handle_define(op: &ParsedOp, registry: &mut FragmentRegistry) -> (String, Option<RegexEvent>) {
    let element_tokens = extract_element_tokens(&op.raw);
    if op.positionals.is_empty() {
        return ("ERROR: define requires a name".to_string(), None);
    }
    let name = &op.positionals[0];
    if element_tokens.is_empty() {
        return ("ERROR: define requires at least one element".to_string(), None);
    }
    let elements: Vec<_> = match element_tokens.iter().map(|t| parse_element(t)).collect::<Result<_, _>>() {
        Ok(elems) => elems,
        Err(e) => return (format!("ERROR: {e}"), None),
    };
    match registry.define(name, elements) {
        Ok(event) => {
            let count = match &event {
                RegexEvent::Define { new, .. } => new.len(),
                _ => 0,
            };
            (
                format!("+ fragment {name:?} defined ({count} elements)"),
                Some(event),
            )
        }
        Err(e) => (format!("ERROR: {e}"), None),
    }
}

pub fn handle_from(op: &ParsedOp, registry: &mut FragmentRegistry) -> (String, Option<RegexEvent>) {
    if op.positionals.is_empty() {
        return ("ERROR: from requires a library source name".to_string(), None);
    }
    let source = &op.positionals[0];
    let Some(pattern) = library::get_pattern(source) else {
        return (format!("ERROR: library pattern {source:?} not found"), None);
    };
    let alias = op.params.get("as").map(|s| s.as_str()).unwrap_or(pattern.name);
    let elements = vec![Element::Raw(pattern.regex.to_string())];
    match registry.define(alias, elements) {
        Ok(event) => (
            format!("+ imported {alias:?} from {source}"),
            Some(event),
        ),
        Err(e) => (format!("ERROR: {e}"), None),
    }
}

pub fn handle_compile(op: &ParsedOp, registry: &FragmentRegistry) -> (String, Option<RegexEvent>) {
    if op.positionals.is_empty() {
        return ("ERROR: compile requires a fragment name".to_string(), None);
    }
    let name = &op.positionals[0];
    let flavor = op.params.get("flavor").map(|s| s.as_str()).unwrap_or("pcre");
    let anchored = op
        .params
        .get("anchored")
        .map(|s| s == "true")
        .unwrap_or(false);
    match compiler::compile(registry, name, flavor, anchored) {
        Ok(result) => (format!("= {name}: {}", result.regex), None),
        Err(e) => (format!("ERROR: {e}"), None),
    }
}

pub fn handle_drop(op: &ParsedOp, registry: &mut FragmentRegistry) -> (String, Option<RegexEvent>) {
    if op.positionals.is_empty() {
        return ("ERROR: drop requires a fragment name".to_string(), None);
    }
    let name = &op.positionals[0];
    match registry.drop(name) {
        Ok(event) => (
            format!("- fragment {name:?} dropped"),
            Some(event),
        ),
        Err(e) => (format!("ERROR: {e}"), None),
    }
}

pub fn handle_rename(op: &ParsedOp, registry: &mut FragmentRegistry) -> (String, Option<RegexEvent>) {
    if op.positionals.len() < 2 {
        return ("ERROR: rename requires old and new names".to_string(), None);
    }
    let old = &op.positionals[0];
    let new = &op.positionals[1];
    match registry.rename(old, new) {
        Ok(event) => (
            format!("* fragment {old:?} renamed to {new:?}"),
            Some(event),
        ),
        Err(e) => (format!("ERROR: {e}"), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fcpcore::parsed_op::parse_op;

    fn make_registry() -> FragmentRegistry {
        FragmentRegistry::new()
    }

    #[test]
    fn test_handle_define() {
        let mut reg = make_registry();
        let op = parse_op("define digits any:digit+").unwrap();
        let (msg, event) = handle_define(&op, &mut reg);
        assert!(msg.starts_with("+ fragment"));
        assert!(msg.contains("1 elements"));
        assert!(event.is_some());
        assert!(reg.contains("digits"));
    }

    #[test]
    fn test_handle_define_multiple_elements() {
        let mut reg = make_registry();
        let op = parse_op("define version any:digit+ lit:. any:digit+").unwrap();
        let (msg, event) = handle_define(&op, &mut reg);
        assert!(msg.contains("3 elements"));
        assert!(event.is_some());
    }

    #[test]
    fn test_handle_define_no_name() {
        let mut reg = make_registry();
        let op = parse_op("define").unwrap();
        let (msg, event) = handle_define(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_define_no_elements() {
        let mut reg = make_registry();
        let op = parse_op("define myname").unwrap();
        let (msg, event) = handle_define(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_define_bad_element() {
        let mut reg = make_registry();
        let op = parse_op("define x any:").unwrap();
        let (msg, event) = handle_define(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_from() {
        let mut reg = make_registry();
        let op = parse_op("from semver").unwrap();
        let (msg, event) = handle_from(&op, &mut reg);
        assert!(msg.contains("imported"));
        assert!(event.is_some());
        // It's stored under the pattern's canonical name
        assert!(reg.len() == 1);
    }

    #[test]
    fn test_handle_from_with_alias() {
        let mut reg = make_registry();
        let op = parse_op("from semver as:sv").unwrap();
        let (msg, event) = handle_from(&op, &mut reg);
        assert!(msg.contains("imported"));
        assert!(event.is_some());
        assert!(reg.contains("sv"));
    }

    #[test]
    fn test_handle_from_not_found() {
        let mut reg = make_registry();
        let op = parse_op("from nonexistent-pattern-xyz").unwrap();
        let (msg, event) = handle_from(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_from_no_name() {
        let mut reg = make_registry();
        let op = parse_op("from").unwrap();
        let (msg, event) = handle_from(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_compile() {
        let mut reg = make_registry();
        let define_op = parse_op("define digits any:digit+").unwrap();
        handle_define(&define_op, &mut reg);
        let op = parse_op("compile digits").unwrap();
        let (msg, _) = handle_compile(&op, &reg);
        assert!(msg.starts_with("= digits:"));
        assert!(msg.contains(r"\d+"));
    }

    #[test]
    fn test_handle_compile_anchored() {
        let mut reg = make_registry();
        let define_op = parse_op("define digits any:digit+").unwrap();
        handle_define(&define_op, &mut reg);
        let op = parse_op("compile digits anchored:true").unwrap();
        let (msg, _) = handle_compile(&op, &reg);
        assert!(msg.contains(r"^\d+$"));
    }

    #[test]
    fn test_handle_compile_not_found() {
        let reg = make_registry();
        let op = parse_op("compile nope").unwrap();
        let (msg, _) = handle_compile(&op, &reg);
        assert!(msg.starts_with("ERROR:"));
    }

    #[test]
    fn test_handle_compile_no_name() {
        let reg = make_registry();
        let op = parse_op("compile").unwrap();
        let (msg, _) = handle_compile(&op, &reg);
        assert!(msg.starts_with("ERROR:"));
    }

    #[test]
    fn test_handle_drop() {
        let mut reg = make_registry();
        let define_op = parse_op("define digits any:digit+").unwrap();
        handle_define(&define_op, &mut reg);
        let op = parse_op("drop digits").unwrap();
        let (msg, event) = handle_drop(&op, &mut reg);
        assert!(msg.contains("dropped"));
        assert!(event.is_some());
        assert!(!reg.contains("digits"));
    }

    #[test]
    fn test_handle_drop_not_found() {
        let mut reg = make_registry();
        let op = parse_op("drop nope").unwrap();
        let (msg, event) = handle_drop(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_drop_no_name() {
        let mut reg = make_registry();
        let op = parse_op("drop").unwrap();
        let (msg, event) = handle_drop(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_rename() {
        let mut reg = make_registry();
        let define_op = parse_op("define digits any:digit+").unwrap();
        handle_define(&define_op, &mut reg);
        let op = parse_op("rename digits nums").unwrap();
        let (msg, event) = handle_rename(&op, &mut reg);
        assert!(msg.contains("renamed"));
        assert!(event.is_some());
        assert!(!reg.contains("digits"));
        assert!(reg.contains("nums"));
    }

    #[test]
    fn test_handle_rename_not_found() {
        let mut reg = make_registry();
        let op = parse_op("rename nope new").unwrap();
        let (msg, event) = handle_rename(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }

    #[test]
    fn test_handle_rename_missing_args() {
        let mut reg = make_registry();
        let op = parse_op("rename onlyone").unwrap();
        let (msg, event) = handle_rename(&op, &mut reg);
        assert!(msg.starts_with("ERROR:"));
        assert!(event.is_none());
    }
}
