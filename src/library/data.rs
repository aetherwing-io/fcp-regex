use super::LibraryPattern;

pub fn patterns() -> &'static [LibraryPattern] {
    &PATTERNS
}

static PATTERNS: [LibraryPattern; 4] = [
    LibraryPattern {
        name: "json:string",
        source: "RFC 8259",
        flavor: "pcre2",
        regex: r#""(?:[^"\\]|\\["\\/bfnrt]|\\u[0-9a-fA-F]{4})*""#,
        structure: r#"" *(char / escape) ""#,
        test_match: &[r#""hello""#, r#""line\nbreak""#, r#""unicode\u0041""#],
        test_no_match: &["no quotes", r#""unterminated"#],
        flavor_notes: "",
        aliases: &["json-string"],
        category: "data",
    },
    LibraryPattern {
        name: "json:number",
        source: "RFC 8259",
        flavor: "pcre2",
        regex: r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?",
        structure: "[minus] int [frac] [exp]",
        test_match: &["42", "-3.14", "1.5e10"],
        test_no_match: &["01", "+5", ".5"],
        flavor_notes: "",
        aliases: &["json-number"],
        category: "data",
    },
    LibraryPattern {
        name: "xml:name",
        source: "XML 1.0 Section 2.3",
        flavor: "pcre2",
        regex: r"[a-zA-Z_][a-zA-Z0-9_.\-]*(?::[a-zA-Z_][a-zA-Z0-9_.\-]*)?",
        structure: "NameStartChar (NameChar)* [: NameStartChar (NameChar)*]",
        test_match: &["element", "ns:tag", "_private"],
        test_no_match: &["1starts-with-digit", "-hyphen-start"],
        flavor_notes: "ASCII subset only; full XML allows Unicode name chars",
        aliases: &["xml-name"],
        category: "data",
    },
    LibraryPattern {
        name: "csv-field",
        source: "RFC 4180",
        flavor: "pcre2",
        regex: r#""(?:[^"]|"")*"|[^,"\r\n]*"#,
        structure: r#"quoted-field / unquoted-field"#,
        test_match: &["simple", r#""has ""quotes""  inside""#, r#""has,comma""#],
        test_no_match: &[],
        flavor_notes: "Matches a single CSV field",
        aliases: &["csv"],
        category: "data",
    },
];
