use super::LibraryPattern;

pub fn patterns() -> &'static [LibraryPattern] {
    &PATTERNS
}

static PATTERNS: [LibraryPattern; 5] = [
    LibraryPattern {
        name: "rfc5322:addr-spec",
        source: "RFC 5322",
        flavor: "pcre2",
        regex: r"[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*",
        structure: "local-part @ domain",
        test_match: &["user@example.com", "first.last+tag@sub.domain.org"],
        test_no_match: &["@no-local.com", "no-at-sign", "user@"],
        flavor_notes: "Practical subset; does not handle quoted local parts",
        aliases: &["email"],
        category: "email",
    },
    LibraryPattern {
        name: "rfc5322:local-part",
        source: "RFC 5322",
        flavor: "pcre2",
        regex: r"[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+",
        structure: "dot-atom / quoted-string (dot-atom only)",
        test_match: &["user", "first.last+tag"],
        test_no_match: &["", " spaces"],
        flavor_notes: "Dot-atom subset only; no quoted strings",
        aliases: &["local-part"],
        category: "email",
    },
    LibraryPattern {
        name: "rfc5322:domain",
        source: "RFC 5322",
        flavor: "pcre2",
        regex: r"[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*",
        structure: "dot-atom (label . label ...)",
        test_match: &["example.com", "sub.domain.org"],
        test_no_match: &["-invalid.com", ""],
        flavor_notes: "",
        aliases: &["domain"],
        category: "email",
    },
    LibraryPattern {
        name: "rfc5322:mailbox",
        source: "RFC 5322",
        flavor: "pcre2",
        regex: r#"(?:[a-zA-Z0-9 .!#$%&'*+/=?^_`{|}~-]+\s*)?<?[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*>?"#,
        structure: "[display-name] addr-spec | [display-name] < addr-spec >",
        test_match: &["user@example.com", "John Doe <john@example.com>"],
        test_no_match: &["<>", "no-email-here"],
        flavor_notes: "Simplified; does not handle all RFC 5322 display-name forms",
        aliases: &["mailbox"],
        category: "email",
    },
    LibraryPattern {
        name: "rfc5322:display-name",
        source: "RFC 5322",
        flavor: "pcre2",
        regex: r#"[a-zA-Z0-9 .!#$%&'*+/=?^_`{|}~-]+"#,
        structure: "word *(word)",
        test_match: &["John Doe", "Alice B."],
        test_no_match: &["", "<brackets>"],
        flavor_notes: "Simplified atom-only form",
        aliases: &["display-name"],
        category: "email",
    },
];
