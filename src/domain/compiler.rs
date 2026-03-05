use std::collections::HashSet;

use crate::elements::{CharClass, Element, Quantifier};

use super::model::FragmentRegistry;

#[derive(Debug)]
pub struct CompileResult {
    pub regex: String,
    pub flavor: String,
    pub explanation: String,
}

pub fn compile(
    registry: &FragmentRegistry,
    name: &str,
    flavor: &str,
    anchored: bool,
) -> Result<CompileResult, String> {
    let mut visited = HashSet::new();
    let regex = compile_fragment(registry, name, &mut visited)?;
    let final_regex = if anchored {
        format!("^{regex}$")
    } else {
        regex
    };
    Ok(CompileResult {
        regex: final_regex,
        flavor: flavor.to_string(),
        explanation: format!("compiled from fragment {name:?}"),
    })
}

fn compile_fragment(
    registry: &FragmentRegistry,
    name: &str,
    visited: &mut HashSet<String>,
) -> Result<String, String> {
    if !visited.insert(name.to_string()) {
        return Err(format!("cycle detected: fragment {name:?} references itself"));
    }
    let fragment = registry
        .get(name)
        .ok_or_else(|| format!("fragment {name:?} not found"))?;
    let parts: Vec<_> = fragment.elements.iter()
        .map(|elem| compile_element(registry, elem, visited))
        .collect::<Result<_, _>>()?;
    visited.remove(name);
    Ok(parts.join(""))
}

fn compile_element(
    registry: &FragmentRegistry,
    elem: &Element,
    visited: &mut HashSet<String>,
) -> Result<String, String> {
    match elem {
        Element::Ref(name) => compile_fragment(registry, name, visited),
        Element::Literal(s) => Ok(escape_regex(s)),
        Element::AnyClass(class, quant) => {
            Ok(format!("{}{}", class_to_regex(class), quantifier_to_str(quant)))
        }
        Element::NoneClass(class, quant) => {
            Ok(format!("{}{}", negated_class_to_regex(class), quantifier_to_str(quant)))
        }
        Element::Chars(set, quant) => {
            Ok(format!("[{}]{}", set, quantifier_to_str(quant)))
        }
        Element::NotChars(set, quant) => {
            Ok(format!("[^{}]{}", set, quantifier_to_str(quant)))
        }
        Element::Optional(name) => {
            let compiled = compile_fragment(registry, name, visited)?;
            Ok(format!("(?:{compiled})?"))
        }
        Element::Alternation(names) => {
            let alts: Vec<_> = names.iter()
                .map(|name| compile_fragment(registry, name, visited))
                .collect::<Result<_, _>>()?;
            Ok(format!("(?:{})", alts.join("|")))
        }
        Element::Capture(name) => {
            let compiled = compile_fragment(registry, name, visited)?;
            Ok(format!("({compiled})"))
        }
        Element::NamedCapture(label, name) => {
            let compiled = compile_fragment(registry, name, visited)?;
            Ok(format!("(?P<{label}>{compiled})"))
        }
        Element::SepBy(name, sep_str) => {
            let compiled_name = compile_fragment(registry, name, visited)?;
            let sep_elem = crate::elements::parse_element(sep_str)
                .map_err(|e| format!("invalid separator element: {e}"))?;
            let compiled_sep = compile_element(registry, &sep_elem, visited)?;
            Ok(format!("{compiled_name}(?:{compiled_sep}{compiled_name})*"))
        }
        Element::Raw(regex) => Ok(regex.clone()),
    }
}

fn class_to_regex(class: &CharClass) -> &'static str {
    match class {
        CharClass::Digit => r"\d",
        CharClass::Alpha => "[a-zA-Z]",
        CharClass::Alphanumeric => "[a-zA-Z0-9]",
        CharClass::Word => r"\w",
        CharClass::Whitespace => r"\s",
        CharClass::Any => ".",
    }
}

fn negated_class_to_regex(class: &CharClass) -> &'static str {
    match class {
        CharClass::Digit => r"\D",
        CharClass::Alpha => "[^a-zA-Z]",
        CharClass::Alphanumeric => "[^a-zA-Z0-9]",
        CharClass::Word => r"\W",
        CharClass::Whitespace => r"\S",
        CharClass::Any => "[^\\s\\S]", // matches nothing — negation of "any"
    }
}

fn quantifier_to_str(quant: &Quantifier) -> String {
    match quant {
        Quantifier::One => String::new(),
        Quantifier::ZeroOrMore => "*".to_string(),
        Quantifier::OneOrMore => "+".to_string(),
        Quantifier::ZeroOrOne => "?".to_string(),
        Quantifier::Exact(n) => format!("{{{n}}}"),
        Quantifier::Range(a, b) => format!("{{{a},{b}}}"),
        Quantifier::AtLeast(n) => format!("{{{n},}}"),
    }
}

fn escape_regex(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for ch in s.chars() {
        if matches!(ch, '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '^' | '$' | '|') {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;

    fn setup_semver() -> FragmentRegistry {
        let mut reg = FragmentRegistry::new();
        reg.define("digits", vec![
            Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore),
        ]).unwrap();
        reg.define("version", vec![
            Element::Ref("digits".to_string()),
            Element::Literal(".".to_string()),
            Element::Ref("digits".to_string()),
            Element::Literal(".".to_string()),
            Element::Ref("digits".to_string()),
        ]).unwrap();
        reg.define("prerelease", vec![
            Element::Literal("-".to_string()),
            Element::Chars("a-zA-Z0-9\\-.".to_string(), Quantifier::OneOrMore),
        ]).unwrap();
        reg.define("semver", vec![
            Element::Ref("version".to_string()),
            Element::Optional("prerelease".to_string()),
        ]).unwrap();
        reg
    }

    #[test]
    fn test_compile_semver_anchored() {
        let reg = setup_semver();
        let result = compile(&reg, "semver", "pcre", true).unwrap();
        assert_eq!(result.regex, r"^\d+\.\d+\.\d+(?:-[a-zA-Z0-9\-.]+)?$");
    }

    #[test]
    fn test_compile_semver_unanchored() {
        let reg = setup_semver();
        let result = compile(&reg, "semver", "pcre", false).unwrap();
        assert_eq!(result.regex, r"\d+\.\d+\.\d+(?:-[a-zA-Z0-9\-.]+)?");
    }

    #[test]
    fn test_compile_literal_escaping() {
        let mut reg = FragmentRegistry::new();
        reg.define("dot", vec![Element::Literal(".".to_string())]).unwrap();
        let result = compile(&reg, "dot", "pcre", false).unwrap();
        assert_eq!(result.regex, r"\.");
    }

    #[test]
    fn test_compile_cycle_detection() {
        let mut reg = FragmentRegistry::new();
        reg.define("a", vec![Element::Ref("b".to_string())]).unwrap();
        reg.define("b", vec![Element::Ref("a".to_string())]).unwrap();
        let result = compile(&reg, "a", "pcre", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cycle"));
    }

    #[test]
    fn test_compile_not_found() {
        let reg = FragmentRegistry::new();
        assert!(compile(&reg, "nope", "pcre", false).is_err());
    }

    #[test]
    fn test_compile_sep_by() {
        let mut reg = FragmentRegistry::new();
        reg.define("octet", vec![
            Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore),
        ]).unwrap();
        reg.define("ipv4", vec![
            Element::SepBy("octet".to_string(), "lit:.".to_string()),
        ]).unwrap();
        let result = compile(&reg, "ipv4", "pcre", false).unwrap();
        assert_eq!(result.regex, r"\d+(?:\.\d+)*");
    }

    #[test]
    fn test_compile_capture() {
        let mut reg = FragmentRegistry::new();
        reg.define("digits", vec![
            Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore),
        ]).unwrap();
        reg.define("cap-test", vec![
            Element::Capture("digits".to_string()),
        ]).unwrap();
        let result = compile(&reg, "cap-test", "pcre", false).unwrap();
        assert_eq!(result.regex, r"(\d+)");
    }

    #[test]
    fn test_compile_named_capture() {
        let mut reg = FragmentRegistry::new();
        reg.define("digits", vec![
            Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore),
        ]).unwrap();
        reg.define("ncap-test", vec![
            Element::NamedCapture("major".to_string(), "digits".to_string()),
        ]).unwrap();
        let result = compile(&reg, "ncap-test", "pcre", false).unwrap();
        assert_eq!(result.regex, r"(?P<major>\d+)");
    }

    #[test]
    fn test_compile_alternation() {
        let mut reg = FragmentRegistry::new();
        reg.define("http", vec![Element::Literal("http".to_string())]).unwrap();
        reg.define("https", vec![Element::Literal("https".to_string())]).unwrap();
        reg.define("ftp", vec![Element::Literal("ftp".to_string())]).unwrap();
        reg.define("scheme", vec![
            Element::Alternation(vec!["http".to_string(), "https".to_string(), "ftp".to_string()]),
        ]).unwrap();
        let result = compile(&reg, "scheme", "pcre", false).unwrap();
        assert_eq!(result.regex, "(?:http|https|ftp)");
    }

    #[test]
    fn test_compile_raw() {
        let mut reg = FragmentRegistry::new();
        reg.define("custom", vec![
            Element::Raw(r"\d{4}-\d{2}".to_string()),
        ]).unwrap();
        let result = compile(&reg, "custom", "pcre", false).unwrap();
        assert_eq!(result.regex, r"\d{4}-\d{2}");
    }

    #[test]
    fn test_compile_none_class() {
        let mut reg = FragmentRegistry::new();
        reg.define("non-digit", vec![
            Element::NoneClass(CharClass::Digit, Quantifier::OneOrMore),
        ]).unwrap();
        let result = compile(&reg, "non-digit", "pcre", false).unwrap();
        assert_eq!(result.regex, r"\D+");
    }

    #[test]
    fn test_compile_not_chars() {
        let mut reg = FragmentRegistry::new();
        reg.define("no-space", vec![
            Element::NotChars("\\s".to_string(), Quantifier::OneOrMore),
        ]).unwrap();
        let result = compile(&reg, "no-space", "pcre", false).unwrap();
        assert_eq!(result.regex, r"[^\s]+");
    }

    #[test]
    fn test_compile_flavor_set() {
        let reg = setup_semver();
        let result = compile(&reg, "semver", "rust", false).unwrap();
        assert_eq!(result.flavor, "rust");
    }

    #[test]
    fn test_compile_deep_nesting() {
        let mut reg = FragmentRegistry::new();
        reg.define("a", vec![Element::Literal("x".to_string())]).unwrap();
        reg.define("b", vec![Element::Ref("a".to_string())]).unwrap();
        reg.define("c", vec![Element::Ref("b".to_string())]).unwrap();
        reg.define("d", vec![Element::Ref("c".to_string())]).unwrap();
        let result = compile(&reg, "d", "pcre", false).unwrap();
        assert_eq!(result.regex, "x");
    }

    #[test]
    fn test_compile_diamond_ref() {
        // a -> b, a -> c, b -> d, c -> d (diamond, NOT a cycle)
        let mut reg = FragmentRegistry::new();
        reg.define("d", vec![Element::Literal("x".to_string())]).unwrap();
        reg.define("b", vec![Element::Ref("d".to_string())]).unwrap();
        reg.define("c", vec![Element::Ref("d".to_string())]).unwrap();
        reg.define("a", vec![
            Element::Ref("b".to_string()),
            Element::Ref("c".to_string()),
        ]).unwrap();
        let result = compile(&reg, "a", "pcre", false).unwrap();
        assert_eq!(result.regex, "xx");
    }
}
