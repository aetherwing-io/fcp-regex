use fcp_regex::domain::compiler;
use fcp_regex::domain::model::FragmentRegistry;
use fcp_regex::elements::{CharClass, Element, Quantifier};

#[test]
fn test_userinfo_workflow() {
    let mut reg = FragmentRegistry::new();

    // user: alphanumeric+
    reg.define(
        "user",
        vec![Element::AnyClass(
            CharClass::Alphanumeric,
            Quantifier::OneOrMore,
        )],
    )
    .unwrap();

    // pass: word characters
    reg.define(
        "pass",
        vec![Element::AnyClass(CharClass::Word, Quantifier::OneOrMore)],
    )
    .unwrap();

    // colon-pass: :pass
    reg.define(
        "colon-pass",
        vec![
            Element::Literal(":".to_string()),
            Element::Ref("pass".to_string()),
        ],
    )
    .unwrap();

    // credentials: user + optional colon-pass
    reg.define(
        "credentials",
        vec![
            Element::Ref("user".to_string()),
            Element::Optional("colon-pass".to_string()),
        ],
    )
    .unwrap();

    // userinfo-at: credentials@
    reg.define(
        "userinfo-at",
        vec![
            Element::Ref("credentials".to_string()),
            Element::Literal("@".to_string()),
        ],
    )
    .unwrap();

    // userinfo-prefix: optional userinfo-at
    reg.define(
        "userinfo-prefix",
        vec![Element::Optional("userinfo-at".to_string())],
    )
    .unwrap();

    let result = compiler::compile(&reg, "userinfo-prefix", "pcre", true).unwrap();

    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("user@"));
    assert!(re.is_match("user:pass@"));
    assert!(re.is_match("admin:secret123@"));
    assert!(re.is_match("")); // optional means empty matches too
}

#[test]
fn test_credentials_standalone() {
    let mut reg = FragmentRegistry::new();
    reg.define(
        "user",
        vec![Element::AnyClass(
            CharClass::Alphanumeric,
            Quantifier::OneOrMore,
        )],
    )
    .unwrap();
    reg.define(
        "pass",
        vec![Element::AnyClass(CharClass::Word, Quantifier::OneOrMore)],
    )
    .unwrap();
    reg.define(
        "colon-pass",
        vec![
            Element::Literal(":".to_string()),
            Element::Ref("pass".to_string()),
        ],
    )
    .unwrap();
    reg.define(
        "credentials",
        vec![
            Element::Ref("user".to_string()),
            Element::Optional("colon-pass".to_string()),
        ],
    )
    .unwrap();

    let result = compiler::compile(&reg, "credentials", "pcre", true).unwrap();
    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("admin"));
    assert!(re.is_match("admin:pass123"));
    assert!(!re.is_match(":pass")); // no user
}
