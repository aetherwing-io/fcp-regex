use fcp_regex::domain::model::FragmentRegistry;
use fcp_regex::domain::query;

#[test]
fn test_explain_placeholder() {
    let reg = FragmentRegistry::new();
    let result = query::handle_query("explain \\d+\\.\\d+", &reg);
    assert!(result.contains("not yet implemented"));
    assert!(result.contains("\\d+\\.\\d+"));
}

#[test]
fn test_explain_empty() {
    let reg = FragmentRegistry::new();
    let result = query::handle_query("explain", &reg);
    assert!(result.contains("requires"));
}
