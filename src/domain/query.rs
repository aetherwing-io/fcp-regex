use super::compiler;
use super::model::FragmentRegistry;
use crate::library;

pub fn handle_query(q: &str, registry: &FragmentRegistry) -> String {
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return "! empty query".to_string();
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap();
    let rest = parts.next().unwrap_or("").trim();

    match command {
        "show" | "describe" => cmd_show(rest, registry),
        "test" => cmd_test(rest, registry),
        "explain" => cmd_explain(rest),
        "list" => cmd_list(rest, registry),
        "get" => cmd_get(rest),
        "map" => cmd_map(registry),
        "stats" => cmd_stats(registry),
        "status" => "= session active".to_string(),
        "history" => cmd_history(rest),
        _ => {
            let known = ["show", "describe", "test", "explain", "list", "get", "map", "stats", "status", "history"];
            let msg = format!("! unknown query command {:?}", command);
            match crate::fcpcore::formatter::suggest(command, &known) {
                Some(s) => format!("{}\n  try: {}", msg, s),
                None => msg,
            }
        }
    }
}

fn cmd_show(name: &str, registry: &FragmentRegistry) -> String {
    if name.is_empty() {
        return "! show requires a fragment name".to_string();
    }
    let fragment = match registry.get(name) {
        Some(f) => f,
        None => return format!("! fragment {:?} not found", name),
    };

    let mut lines = Vec::new();
    lines.push(format!("= FRAGMENT: {}", name));
    lines.push(format!("  ELEMENTS ({}): {:?}", fragment.elements.len(), fragment.elements));

    match compiler::compile(registry, name, "pcre", false) {
        Ok(result) => {
            lines.push(format!("  REGEX: {}", result.regex));
        }
        Err(e) => {
            lines.push(format!("  COMPILE ERROR: {}", e));
        }
    }
    lines.join("\n")
}

fn cmd_test(rest: &str, registry: &FragmentRegistry) -> String {
    // Parse: NAME against:STRING
    let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
    if parts.is_empty() || parts[0].is_empty() {
        return "! test requires: test NAME against:STRING".to_string();
    }
    let name = parts[0];
    let args = if parts.len() > 1 { parts[1].trim() } else { "" };

    let test_string = if let Some(stripped) = args.strip_prefix("against:") {
        stripped
    } else {
        return "! test requires against:STRING parameter".to_string();
    };

    let compiled = match compiler::compile(registry, name, "pcre", false) {
        Ok(r) => r,
        Err(e) => return format!("! compile error: {}", e),
    };

    match regex::Regex::new(&compiled.regex) {
        Ok(re) => {
            if re.is_match(test_string) {
                format!("= MATCH — {:?} matches /{}/", test_string, compiled.regex)
            } else {
                format!("= NO MATCH — {:?} does not match /{}/", test_string, compiled.regex)
            }
        }
        Err(e) => format!("! regex error: {}", e),
    }
}

fn cmd_explain(rest: &str) -> String {
    if rest.is_empty() {
        return "! explain requires a regex string".to_string();
    }
    format!("= REGEX: {}\n  explain not yet implemented", rest)
}

fn cmd_list(rest: &str, registry: &FragmentRegistry) -> String {
    let trimmed = rest.trim();

    if trimmed == "library" {
        let cats = library::list_categories();
        let mut lines = vec!["= LIBRARY CATEGORIES:".to_string()];
        for (name, count) in &cats {
            lines.push(format!("  {} ({})", name, count));
        }
        return lines.join("\n");
    }

    if trimmed.starts_with("library") {
        // Check for category: param
        let lib_rest = trimmed.strip_prefix("library").unwrap().trim();
        if let Some(cat) = lib_rest.strip_prefix("category:") {
            let patterns = library::list_category(cat);
            if patterns.is_empty() {
                return format!("! unknown library category {:?}", cat);
            }
            let mut lines = vec![format!("= LIBRARY — {}:", cat)];
            for p in &patterns {
                lines.push(format!("  {}", p.name));
            }
            return lines.join("\n");
        }
    }

    if trimmed.is_empty() {
        let fragments = registry.list();
        if fragments.is_empty() {
            return "= no fragments defined".to_string();
        }
        let mut lines = vec!["= FRAGMENTS:".to_string()];
        for frag in &fragments {
            lines.push(format!("  {} ({} elements)", frag.name, frag.elements.len()));
        }
        return lines.join("\n");
    }

    format!("! unknown list subcommand {:?}", trimmed)
}

fn cmd_get(name: &str) -> String {
    if name.is_empty() {
        return "! get requires a pattern name".to_string();
    }
    match library::get_pattern(name) {
        Some(p) => {
            let mut lines = Vec::new();
            lines.push(format!("= PATTERN: {}", p.name));
            lines.push(format!("  SOURCE: {}", p.source));
            lines.push(format!("  FLAVOR: {}", p.flavor));
            lines.push(format!("  REGEX: {}", p.regex));
            lines.push(format!("  STRUCTURE: {}", p.structure));
            if !p.test_match.is_empty() {
                lines.push(format!("  TEST CASES (match): {}", p.test_match.join(", ")));
            }
            if !p.test_no_match.is_empty() {
                lines.push(format!("  TEST CASES (no match): {}", p.test_no_match.join(", ")));
            }
            if !p.flavor_notes.is_empty() {
                lines.push(format!("  FLAVOR NOTES: {}", p.flavor_notes));
            }
            lines.join("\n")
        }
        None => format!("! pattern {:?} not found", name),
    }
}

fn cmd_map(registry: &FragmentRegistry) -> String {
    let lib_count = library::all_patterns().len();
    format!(
        "= fragments: {}, library: {} patterns available",
        registry.len(),
        lib_count
    )
}

fn cmd_stats(registry: &FragmentRegistry) -> String {
    let fragments = registry.list();
    let lib_count = library::all_patterns().len();
    let categories = library::list_categories();

    let mut lines = Vec::new();
    lines.push("= STATS:".to_string());
    lines.push(format!("  Fragments: {}", fragments.len()));
    let total_elements: usize = fragments.iter().map(|f| f.elements.len()).sum();
    lines.push(format!("  Total elements: {}", total_elements));
    lines.push(format!("  Library patterns: {}", lib_count));
    lines.push(format!("  Library categories: {}", categories.len()));
    lines.join("\n")
}

fn cmd_history(_rest: &str) -> String {
    "= history not yet implemented".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{CharClass, Element, Quantifier};

    fn setup_registry() -> FragmentRegistry {
        let mut reg = FragmentRegistry::new();
        reg.define(
            "digits",
            vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)],
        )
        .unwrap();
        reg.define(
            "version",
            vec![
                Element::Ref("digits".to_string()),
                Element::Literal(".".to_string()),
                Element::Ref("digits".to_string()),
                Element::Literal(".".to_string()),
                Element::Ref("digits".to_string()),
            ],
        )
        .unwrap();
        reg
    }

    #[test]
    fn test_empty_query() {
        let reg = FragmentRegistry::new();
        assert!(handle_query("", &reg).starts_with("!"));
    }

    #[test]
    fn test_unknown_command() {
        let reg = FragmentRegistry::new();
        assert!(handle_query("foobar", &reg).contains("unknown"));
    }

    #[test]
    fn test_show() {
        let reg = setup_registry();
        let result = handle_query("show digits", &reg);
        assert!(result.contains("FRAGMENT: digits"));
        assert!(result.contains("REGEX:"));
        assert!(result.contains(r"\d+"));
    }

    #[test]
    fn test_show_not_found() {
        let reg = FragmentRegistry::new();
        let result = handle_query("show nope", &reg);
        assert!(result.contains("not found"));
    }

    #[test]
    fn test_show_no_name() {
        let reg = FragmentRegistry::new();
        let result = handle_query("show", &reg);
        assert!(result.contains("requires"));
    }

    #[test]
    fn test_describe_alias() {
        let reg = setup_registry();
        let result = handle_query("describe digits", &reg);
        assert!(result.contains("FRAGMENT: digits"));
    }

    #[test]
    fn test_test_match() {
        let reg = setup_registry();
        let result = handle_query("test digits against:123", &reg);
        assert!(result.contains("MATCH"));
        assert!(!result.contains("NO MATCH"));
    }

    #[test]
    fn test_test_no_match() {
        let reg = setup_registry();
        let result = handle_query("test digits against:abc", &reg);
        assert!(result.contains("NO MATCH"));
    }

    #[test]
    fn test_test_missing_against() {
        let reg = setup_registry();
        let result = handle_query("test digits", &reg);
        assert!(result.contains("requires"));
    }

    #[test]
    fn test_test_missing_name() {
        let reg = FragmentRegistry::new();
        let result = handle_query("test", &reg);
        assert!(result.contains("requires"));
    }

    #[test]
    fn test_explain_placeholder() {
        let reg = FragmentRegistry::new();
        let result = handle_query("explain \\d+", &reg);
        assert!(result.contains("not yet implemented"));
        assert!(result.contains("\\d+"));
    }

    #[test]
    fn test_list_empty() {
        let reg = FragmentRegistry::new();
        let result = handle_query("list", &reg);
        assert!(result.contains("no fragments"));
    }

    #[test]
    fn test_list_fragments() {
        let reg = setup_registry();
        let result = handle_query("list", &reg);
        assert!(result.contains("digits"));
        assert!(result.contains("version"));
    }

    #[test]
    fn test_list_library() {
        let reg = FragmentRegistry::new();
        let result = handle_query("list library", &reg);
        assert!(result.contains("LIBRARY CATEGORIES"));
        assert!(result.contains("uri"));
    }

    #[test]
    fn test_list_library_category() {
        let reg = FragmentRegistry::new();
        let result = handle_query("list library category:uri", &reg);
        assert!(result.contains("LIBRARY"));
    }

    #[test]
    fn test_list_library_unknown_category() {
        let reg = FragmentRegistry::new();
        let result = handle_query("list library category:nonexistent", &reg);
        assert!(result.contains("unknown"));
    }

    #[test]
    fn test_get_pattern() {
        let reg = FragmentRegistry::new();
        let result = handle_query("get semver", &reg);
        assert!(result.contains("PATTERN:"));
        assert!(result.contains("REGEX:"));
    }

    #[test]
    fn test_get_unknown() {
        let reg = FragmentRegistry::new();
        let result = handle_query("get nonexistent", &reg);
        assert!(result.contains("not found"));
    }

    #[test]
    fn test_get_no_name() {
        let reg = FragmentRegistry::new();
        let result = handle_query("get", &reg);
        assert!(result.contains("requires"));
    }

    #[test]
    fn test_map() {
        let reg = setup_registry();
        let result = handle_query("map", &reg);
        assert!(result.contains("fragments: 2"));
        assert!(result.contains("library:"));
    }

    #[test]
    fn test_stats() {
        let reg = setup_registry();
        let result = handle_query("stats", &reg);
        assert!(result.contains("STATS:"));
        assert!(result.contains("Fragments: 2"));
        assert!(result.contains("Total elements:"));
    }

    #[test]
    fn test_status() {
        let reg = FragmentRegistry::new();
        let result = handle_query("status", &reg);
        assert!(result.contains("session active"));
    }

    #[test]
    fn test_history_placeholder() {
        let reg = FragmentRegistry::new();
        let result = handle_query("history 10", &reg);
        assert!(result.contains("not yet implemented"));
    }
}
