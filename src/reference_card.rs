pub const REFERENCE_CARD: &str = r#"Execute regex fragment operations. Each op: VERB [args...] [key:value ...]

FRAGMENTS:
  define NAME ELEMENT [ELEMENT...]  Create named pattern fragment
  from SOURCE [as:ALIAS]           Import from pattern library
  compile NAME [flavor:F] [anchored:bool]  Emit regex string
  drop NAME                        Remove fragment
  rename OLD NEW                   Rename fragment

ELEMENTS:
  <name>        Reference another fragment
  lit:<chars>   Literal (auto-escaped)
  any:<C><Q>    Character class     none:<C><Q>  Negated class
  chars:<S><Q>  Custom char set     not:<S><Q>   Negated set
  opt:<name>    Optional            alt:<a>|<b>  Alternation
  cap:<name>    Capture group       cap:<L>/<N>  Named capture
  sep:<N>/<L>   Separated repeat    raw:<regex>  Raw regex

CLASSES: digit alpha alphanumeric word whitespace any
QUANTIFIERS: + (1+) * (0+) ? (0..1) {N} {N,M} {N,}

QUERIES (via regex_query):
  show NAME         Fragment tree + regex
  test NAME against:STR   Test match
  list              All fragments
  list library      Pattern categories
  get PATTERN       Library pattern detail

SESSION (via regex_session):
  new "Title" [flavor:pcre]   Start session
  close / status / undo / redo

RESPONSE PREFIXES: + created  * modified  - deleted  = result  ! error"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_card_not_empty() {
        assert!(!REFERENCE_CARD.is_empty());
    }

    #[test]
    fn test_reference_card_under_2000_chars() {
        assert!(
            REFERENCE_CARD.len() < 2000,
            "reference card is {} chars, should be under 2000",
            REFERENCE_CARD.len()
        );
    }

    #[test]
    fn test_reference_card_contains_verbs() {
        assert!(REFERENCE_CARD.contains("define"));
        assert!(REFERENCE_CARD.contains("compile"));
        assert!(REFERENCE_CARD.contains("from"));
        assert!(REFERENCE_CARD.contains("drop"));
        assert!(REFERENCE_CARD.contains("rename"));
    }

    #[test]
    fn test_reference_card_contains_elements() {
        assert!(REFERENCE_CARD.contains("lit:"));
        assert!(REFERENCE_CARD.contains("any:"));
        assert!(REFERENCE_CARD.contains("cap:"));
        assert!(REFERENCE_CARD.contains("raw:"));
    }

    #[test]
    fn test_reference_card_contains_queries() {
        assert!(REFERENCE_CARD.contains("show"));
        assert!(REFERENCE_CARD.contains("test"));
        assert!(REFERENCE_CARD.contains("list"));
        assert!(REFERENCE_CARD.contains("get"));
    }

    #[test]
    fn test_reference_card_contains_session() {
        assert!(REFERENCE_CARD.contains("new"));
        assert!(REFERENCE_CARD.contains("close"));
        assert!(REFERENCE_CARD.contains("undo"));
        assert!(REFERENCE_CARD.contains("redo"));
    }
}
