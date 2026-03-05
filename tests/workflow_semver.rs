use fcp_regex::domain::compiler;
use fcp_regex::domain::model::FragmentRegistry;
use fcp_regex::elements::{CharClass, Element, Quantifier};

#[test]
fn test_semver_workflow() {
    let mut reg = FragmentRegistry::new();

    // Define digits
    reg.define(
        "digits",
        vec![Element::AnyClass(CharClass::Digit, Quantifier::OneOrMore)],
    )
    .unwrap();

    // Define version: digits.digits.digits
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

    // Define prerelease: -[a-zA-Z0-9-.]+
    reg.define(
        "prerelease",
        vec![
            Element::Literal("-".to_string()),
            Element::Chars("a-zA-Z0-9\\-.".to_string(), Quantifier::OneOrMore),
        ],
    )
    .unwrap();

    // Define semver: version + optional prerelease
    reg.define(
        "semver",
        vec![
            Element::Ref("version".to_string()),
            Element::Optional("prerelease".to_string()),
        ],
    )
    .unwrap();

    // Compile anchored
    let result = compiler::compile(&reg, "semver", "pcre", true).unwrap();
    assert_eq!(result.regex, r"^\d+\.\d+\.\d+(?:-[a-zA-Z0-9\-.]+)?$");

    // Verify the compiled regex actually works
    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("1.2.3"));
    assert!(re.is_match("0.0.1"));
    assert!(re.is_match("1.2.3-beta.1"));
    assert!(re.is_match("10.20.30-alpha"));
    assert!(!re.is_match("1.2"));
    assert!(!re.is_match("abc"));
    assert!(!re.is_match("v1.2.3")); // no v prefix
}

#[test]
fn test_semver_unanchored_finds_in_text() {
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

    let result = compiler::compile(&reg, "version", "pcre", false).unwrap();
    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("version is 1.2.3 ok"));
}
