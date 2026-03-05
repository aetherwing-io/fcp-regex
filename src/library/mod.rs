pub mod uri;
pub mod email;
pub mod datetime;
pub mod identifiers;
pub mod network;
pub mod http;
pub mod common;
pub mod data;

/// A curated regex pattern from the library.
#[derive(Debug, Clone)]
pub struct LibraryPattern {
    pub name: &'static str,
    pub source: &'static str,
    pub flavor: &'static str,
    pub regex: &'static str,
    pub structure: &'static str,
    pub test_match: &'static [&'static str],
    pub test_no_match: &'static [&'static str],
    pub flavor_notes: &'static str,
    pub aliases: &'static [&'static str],
    pub category: &'static str,
}

/// Get all patterns from all categories.
pub fn all_patterns() -> Vec<&'static LibraryPattern> {
    let mut result = Vec::new();
    for module in all_modules() {
        for p in module {
            result.push(p);
        }
    }
    result
}

/// Look up a pattern by name or alias.
pub fn get_pattern(name: &str) -> Option<&'static LibraryPattern> {
    for module in all_modules() {
        for p in module {
            if p.name == name {
                return Some(p);
            }
            for alias in p.aliases {
                if *alias == name {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// List categories with counts.
pub fn list_categories() -> Vec<(&'static str, usize)> {
    let modules = all_modules();
    let categories = [
        "uri", "email", "datetime", "identifiers", "network", "http", "common", "data",
    ];
    categories
        .iter()
        .zip(modules.iter())
        .map(|(cat, m)| (*cat, m.len()))
        .collect()
}

/// List patterns in a category.
pub fn list_category(category: &str) -> Vec<&'static LibraryPattern> {
    for module in all_modules() {
        if let Some(first) = module.first() {
            if first.category == category {
                return module.iter().collect();
            }
        }
    }
    Vec::new()
}

fn all_modules() -> [&'static [LibraryPattern]; 8] {
    [
        uri::patterns(),
        email::patterns(),
        datetime::patterns(),
        identifiers::patterns(),
        network::patterns(),
        http::patterns(),
        common::patterns(),
        data::patterns(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_pattern_count() {
        let all = all_patterns();
        assert!(all.len() >= 50, "expected >= 50 patterns, got {}", all.len());
    }

    #[test]
    fn test_lookup_by_name() {
        assert!(get_pattern("rfc4122:uuid").is_some());
        assert!(get_pattern("semver").is_some());
        assert!(get_pattern("ipv4").is_some());
    }

    #[test]
    fn test_lookup_by_alias() {
        let p = get_pattern("uuid").expect("uuid alias should resolve");
        assert_eq!(p.name, "rfc4122:uuid");
        let p = get_pattern("email").expect("email alias should resolve");
        assert_eq!(p.name, "rfc5322:addr-spec");
    }

    #[test]
    fn test_list_categories() {
        let cats = list_categories();
        assert_eq!(cats.len(), 8);
        let names: Vec<&str> = cats.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"uri"));
        assert!(names.contains(&"network"));
    }

    #[test]
    fn test_list_category() {
        let uri = list_category("uri");
        assert!(uri.len() >= 8, "expected >= 8 uri patterns");
        let network = list_category("network");
        assert!(network.len() >= 6, "expected >= 6 network patterns");
    }

    #[test]
    fn test_all_regexes_compile() {
        for p in all_patterns() {
            let result = regex_syntax::Parser::new().parse(p.regex);
            assert!(
                result.is_ok(),
                "Pattern {} has invalid regex: {} — error: {:?}",
                p.name,
                p.regex,
                result.err()
            );
        }
    }

    #[test]
    fn test_unknown_pattern() {
        assert!(get_pattern("nonexistent").is_none());
    }
}
