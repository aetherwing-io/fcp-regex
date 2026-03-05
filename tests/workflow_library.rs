use fcp_regex::domain::compiler;
use fcp_regex::domain::model::FragmentRegistry;
use fcp_regex::domain::mutation;
use fcp_regex::elements::Element;
use fcp_regex_core::parse::parse_op;
use fcp_regex::library;

#[test]
fn test_import_from_library_and_compose() {
    let mut reg = FragmentRegistry::new();

    // Import ipv4 from library (using alias without colon)
    let op = parse_op("from ipv4").unwrap();
    let (msg, _) = mutation::handle_from(&op, &mut reg);
    assert!(msg.contains("imported"), "got: {}", msg);

    // Import semver
    let op = parse_op("from semver as:sv").unwrap();
    let (msg, _) = mutation::handle_from(&op, &mut reg);
    assert!(msg.contains("imported"), "got: {}", msg);
    assert!(reg.contains("sv"));

    // Define a composite fragment referencing imported patterns
    reg.define(
        "combined",
        vec![
            Element::Ref("sv".to_string()),
            Element::Literal(" on ".to_string()),
            Element::Ref("ipv4".to_string()),
        ],
    )
    .unwrap();

    // Compile
    let result = compiler::compile(&reg, "combined", "pcre", false).unwrap();
    assert!(!result.regex.is_empty());

    // The regex should be parseable
    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("1.2.3 on 192.168.1.1"));
}

#[test]
fn test_library_direct_use() {
    let mut reg = FragmentRegistry::new();

    // Import directly using the library API to work around colon-in-name parsing
    let scheme_pattern = library::get_pattern("rfc3986:scheme").unwrap();
    reg.define("scheme", vec![Element::Raw(scheme_pattern.regex.to_string())])
        .unwrap();

    let authority_pattern = library::get_pattern("rfc3986:authority").unwrap();
    reg.define(
        "authority",
        vec![Element::Raw(authority_pattern.regex.to_string())],
    )
    .unwrap();

    reg.define(
        "url",
        vec![
            Element::Ref("scheme".to_string()),
            Element::Literal("://".to_string()),
            Element::Ref("authority".to_string()),
        ],
    )
    .unwrap();

    let result = compiler::compile(&reg, "url", "pcre", false).unwrap();
    let re = regex::Regex::new(&result.regex).unwrap();
    assert!(re.is_match("http://example.com"));
}

#[test]
fn test_library_list_and_get() {
    let cats = library::list_categories();
    assert!(cats.len() >= 8);

    let all = library::all_patterns();
    assert!(all.len() >= 50);

    // Get a specific pattern
    let semver = library::get_pattern("semver").unwrap();
    assert!(!semver.regex.is_empty());

    // Verify it compiles
    let re = regex::Regex::new(semver.regex).unwrap();
    assert!(re.is_match("1.2.3"));
}

#[test]
fn test_library_patterns_all_compile_as_regex() {
    for pattern in library::all_patterns() {
        let result = regex::Regex::new(pattern.regex);
        assert!(
            result.is_ok(),
            "library pattern {} has invalid regex: {} — {:?}",
            pattern.name,
            pattern.regex,
            result.err()
        );
    }
}
