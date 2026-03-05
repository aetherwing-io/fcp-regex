/// A single element within a `define` op.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Ref(String),
    Literal(String),
    AnyClass(CharClass, Quantifier),
    NoneClass(CharClass, Quantifier),
    Chars(String, Quantifier),
    NotChars(String, Quantifier),
    Optional(String),
    Alternation(Vec<String>),
    Capture(String),
    NamedCapture(String, String),
    SepBy(String, String),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharClass {
    Digit,
    Alpha,
    Alphanumeric,
    Word,
    Whitespace,
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    One,
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
    Exact(usize),
    Range(usize, usize),
    AtLeast(usize),
}

/// Validate that a fragment name contains only [a-zA-Z0-9-].
pub fn validate_fragment_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("fragment name cannot be empty".to_string());
    }
    if let Some(ch) = name.chars().find(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-')) {
        return Err(format!(
            "invalid character {ch:?} in fragment name {name:?} (allowed: a-z, A-Z, 0-9, -)",
        ));
    }
    Ok(())
}

/// Parse a quantifier suffix from the end of a string.
/// Returns (remaining_string, quantifier).
fn parse_quantifier(s: &str) -> (&str, Quantifier) {
    // Check for {N}, {N,M}, {N,} at the end
    if let Some(brace_start) = s.rfind('{') {
        let tail = &s[brace_start..];
        if tail.ends_with('}') {
            let inner = &tail[1..tail.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let left = &inner[..comma_pos];
                let right = &inner[comma_pos + 1..];
                if let Ok(min) = left.parse::<usize>() {
                    if right.is_empty() {
                        return (&s[..brace_start], Quantifier::AtLeast(min));
                    } else if let Ok(max) = right.parse::<usize>() {
                        return (&s[..brace_start], Quantifier::Range(min, max));
                    }
                }
            } else if let Ok(n) = inner.parse::<usize>() {
                return (&s[..brace_start], Quantifier::Exact(n));
            }
        }
    }

    // Check for trailing +, *, ?
    if let Some(stripped) = s.strip_suffix('+') {
        return (stripped, Quantifier::OneOrMore);
    }
    if let Some(stripped) = s.strip_suffix('*') {
        return (stripped, Quantifier::ZeroOrMore);
    }
    if let Some(stripped) = s.strip_suffix('?') {
        return (stripped, Quantifier::ZeroOrOne);
    }

    (s, Quantifier::One)
}

fn parse_char_class(s: &str) -> Result<CharClass, String> {
    match s {
        "digit" => Ok(CharClass::Digit),
        "alpha" => Ok(CharClass::Alpha),
        "alphanumeric" => Ok(CharClass::Alphanumeric),
        "word" => Ok(CharClass::Word),
        "whitespace" => Ok(CharClass::Whitespace),
        "any" => Ok(CharClass::Any),
        _ => Err(format!("unknown character class {s:?}")),
    }
}

/// Parse a single element token into an Element.
pub fn parse_element(token: &str) -> Result<Element, String> {
    if let Some(rest) = token.strip_prefix("lit:") {
        if rest.is_empty() {
            return Err("lit: requires characters".to_string());
        }
        return Ok(Element::Literal(rest.to_string()));
    }

    if let Some(rest) = token.strip_prefix("any:") {
        if rest.is_empty() {
            return Err("any: requires a character class".to_string());
        }
        let (class_str, quant) = parse_quantifier(rest);
        let class = parse_char_class(class_str)?;
        return Ok(Element::AnyClass(class, quant));
    }

    if let Some(rest) = token.strip_prefix("none:") {
        if rest.is_empty() {
            return Err("none: requires a character class".to_string());
        }
        let (class_str, quant) = parse_quantifier(rest);
        let class = parse_char_class(class_str)?;
        return Ok(Element::NoneClass(class, quant));
    }

    if let Some(rest) = token.strip_prefix("chars:") {
        if rest.is_empty() {
            return Err("chars: requires a character set".to_string());
        }
        let (set_str, quant) = parse_quantifier(rest);
        return Ok(Element::Chars(set_str.to_string(), quant));
    }

    if let Some(rest) = token.strip_prefix("not:") {
        if rest.is_empty() {
            return Err("not: requires a character set".to_string());
        }
        let (set_str, quant) = parse_quantifier(rest);
        return Ok(Element::NotChars(set_str.to_string(), quant));
    }

    if let Some(rest) = token.strip_prefix("opt:") {
        if rest.is_empty() {
            return Err("opt: requires a name".to_string());
        }
        return Ok(Element::Optional(rest.to_string()));
    }

    if let Some(rest) = token.strip_prefix("alt:") {
        if rest.is_empty() {
            return Err("alt: requires alternatives separated by |".to_string());
        }
        let names: Vec<String> = rest.split('|').map(|s| s.to_string()).collect();
        if names.len() < 2 {
            return Err("alt: requires at least 2 alternatives".to_string());
        }
        return Ok(Element::Alternation(names));
    }

    if let Some(rest) = token.strip_prefix("cap:") {
        if rest.is_empty() {
            return Err("cap: requires a name".to_string());
        }
        if let Some((label, name)) = rest.split_once('/') {
            if label.is_empty() || name.is_empty() {
                return Err("cap: named capture requires label/name".to_string());
            }
            return Ok(Element::NamedCapture(label.to_string(), name.to_string()));
        }
        return Ok(Element::Capture(rest.to_string()));
    }

    if let Some(rest) = token.strip_prefix("sep:") {
        if rest.is_empty() {
            return Err("sep: requires name/separator".to_string());
        }
        let (name, sep) = rest.split_once('/').ok_or("sep: requires name/separator format")?;
        if name.is_empty() || sep.is_empty() {
            return Err("sep: requires both name and separator".to_string());
        }
        return Ok(Element::SepBy(name.to_string(), sep.to_string()));
    }

    if let Some(rest) = token.strip_prefix("raw:") {
        if rest.is_empty() {
            return Err("raw: requires a regex pattern".to_string());
        }
        return Ok(Element::Raw(rest.to_string()));
    }

    // Bare name reference
    Ok(Element::Ref(token.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Ref --
    #[test]
    fn test_parse_ref() {
        assert_eq!(parse_element("digits").unwrap(), Element::Ref("digits".to_string()));
    }

    #[test]
    fn test_parse_ref_with_hyphens() {
        assert_eq!(parse_element("my-fragment").unwrap(), Element::Ref("my-fragment".to_string()));
    }

    // -- Literal --
    #[test]
    fn test_parse_literal() {
        assert_eq!(parse_element("lit:.").unwrap(), Element::Literal(".".to_string()));
    }

    #[test]
    fn test_parse_literal_multi_char() {
        assert_eq!(parse_element("lit:://").unwrap(), Element::Literal("://".to_string()));
    }

    #[test]
    fn test_parse_literal_empty_error() {
        assert!(parse_element("lit:").is_err());
    }

    // -- AnyClass --
    #[test]
    fn test_parse_any_digit() {
        assert_eq!(
            parse_element("any:digit").unwrap(),
            Element::AnyClass(CharClass::Digit, Quantifier::One)
        );
    }

    #[test]
    fn test_parse_any_digit_plus() {
        assert_eq!(
            parse_element("any:digit+").unwrap(),
            Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_any_alpha_star() {
        assert_eq!(
            parse_element("any:alpha*").unwrap(),
            Element::AnyClass(CharClass::Alpha, Quantifier::ZeroOrMore)
        );
    }

    #[test]
    fn test_parse_any_word_question() {
        assert_eq!(
            parse_element("any:word?").unwrap(),
            Element::AnyClass(CharClass::Word, Quantifier::ZeroOrOne)
        );
    }

    #[test]
    fn test_parse_any_whitespace() {
        assert_eq!(
            parse_element("any:whitespace+").unwrap(),
            Element::AnyClass(CharClass::Whitespace, Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_any_alphanumeric() {
        assert_eq!(
            parse_element("any:alphanumeric+").unwrap(),
            Element::AnyClass(CharClass::Alphanumeric, Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_any_any() {
        assert_eq!(
            parse_element("any:any*").unwrap(),
            Element::AnyClass(CharClass::Any, Quantifier::ZeroOrMore)
        );
    }

    #[test]
    fn test_parse_any_exact() {
        assert_eq!(
            parse_element("any:digit{3}").unwrap(),
            Element::AnyClass(CharClass::Digit, Quantifier::Exact(3))
        );
    }

    #[test]
    fn test_parse_any_range() {
        assert_eq!(
            parse_element("any:digit{1,3}").unwrap(),
            Element::AnyClass(CharClass::Digit, Quantifier::Range(1, 3))
        );
    }

    #[test]
    fn test_parse_any_at_least() {
        assert_eq!(
            parse_element("any:digit{3,}").unwrap(),
            Element::AnyClass(CharClass::Digit, Quantifier::AtLeast(3))
        );
    }

    #[test]
    fn test_parse_any_unknown_class() {
        assert!(parse_element("any:foobar").is_err());
    }

    #[test]
    fn test_parse_any_empty() {
        assert!(parse_element("any:").is_err());
    }

    // -- NoneClass --
    #[test]
    fn test_parse_none_digit() {
        assert_eq!(
            parse_element("none:digit+").unwrap(),
            Element::NoneClass(CharClass::Digit, Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_none_word() {
        assert_eq!(
            parse_element("none:word").unwrap(),
            Element::NoneClass(CharClass::Word, Quantifier::One)
        );
    }

    #[test]
    fn test_parse_none_empty() {
        assert!(parse_element("none:").is_err());
    }

    // -- Chars --
    #[test]
    fn test_parse_chars() {
        assert_eq!(
            parse_element("chars:a-zA-Z0-9").unwrap(),
            Element::Chars("a-zA-Z0-9".to_string(), Quantifier::One)
        );
    }

    #[test]
    fn test_parse_chars_with_quantifier() {
        assert_eq!(
            parse_element("chars:a-zA-Z0-9-.+").unwrap(),
            Element::Chars("a-zA-Z0-9-.".to_string(), Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_chars_empty() {
        assert!(parse_element("chars:").is_err());
    }

    // -- NotChars --
    #[test]
    fn test_parse_not_chars() {
        assert_eq!(
            parse_element("not:@#$+").unwrap(),
            Element::NotChars("@#$".to_string(), Quantifier::OneOrMore)
        );
    }

    #[test]
    fn test_parse_not_empty() {
        assert!(parse_element("not:").is_err());
    }

    // -- Optional --
    #[test]
    fn test_parse_optional() {
        assert_eq!(
            parse_element("opt:prerelease").unwrap(),
            Element::Optional("prerelease".to_string())
        );
    }

    #[test]
    fn test_parse_optional_empty() {
        assert!(parse_element("opt:").is_err());
    }

    // -- Alternation --
    #[test]
    fn test_parse_alternation() {
        assert_eq!(
            parse_element("alt:http|https|ftp").unwrap(),
            Element::Alternation(vec!["http".to_string(), "https".to_string(), "ftp".to_string()])
        );
    }

    #[test]
    fn test_parse_alternation_two() {
        assert_eq!(
            parse_element("alt:yes|no").unwrap(),
            Element::Alternation(vec!["yes".to_string(), "no".to_string()])
        );
    }

    #[test]
    fn test_parse_alternation_single_error() {
        assert!(parse_element("alt:only").is_err());
    }

    #[test]
    fn test_parse_alternation_empty() {
        assert!(parse_element("alt:").is_err());
    }

    // -- Capture --
    #[test]
    fn test_parse_capture() {
        assert_eq!(
            parse_element("cap:version").unwrap(),
            Element::Capture("version".to_string())
        );
    }

    #[test]
    fn test_parse_named_capture() {
        assert_eq!(
            parse_element("cap:major/digits").unwrap(),
            Element::NamedCapture("major".to_string(), "digits".to_string())
        );
    }

    #[test]
    fn test_parse_capture_empty() {
        assert!(parse_element("cap:").is_err());
    }

    #[test]
    fn test_parse_named_capture_empty_label() {
        assert!(parse_element("cap:/name").is_err());
    }

    #[test]
    fn test_parse_named_capture_empty_name() {
        assert!(parse_element("cap:label/").is_err());
    }

    // -- SepBy --
    #[test]
    fn test_parse_sep_by() {
        assert_eq!(
            parse_element("sep:octet/lit:.").unwrap(),
            Element::SepBy("octet".to_string(), "lit:.".to_string())
        );
    }

    #[test]
    fn test_parse_sep_by_empty() {
        assert!(parse_element("sep:").is_err());
    }

    #[test]
    fn test_parse_sep_by_no_slash() {
        assert!(parse_element("sep:foo").is_err());
    }

    #[test]
    fn test_parse_sep_by_empty_parts() {
        assert!(parse_element("sep:/").is_err());
    }

    // -- Raw --
    #[test]
    fn test_parse_raw() {
        assert_eq!(
            parse_element("raw:\\d{4}-\\d{2}").unwrap(),
            Element::Raw("\\d{4}-\\d{2}".to_string())
        );
    }

    #[test]
    fn test_parse_raw_empty() {
        assert!(parse_element("raw:").is_err());
    }

    // -- Name validation --
    #[test]
    fn test_validate_name_valid() {
        assert!(validate_fragment_name("my-fragment").is_ok());
        assert!(validate_fragment_name("digits").is_ok());
        assert!(validate_fragment_name("v2").is_ok());
        assert!(validate_fragment_name("A-Z").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_fragment_name("").is_err());
        assert!(validate_fragment_name("has space").is_err());
        assert!(validate_fragment_name("has_underscore").is_err());
        assert!(validate_fragment_name("has.dot").is_err());
        assert!(validate_fragment_name("has:colon").is_err());
    }
}
