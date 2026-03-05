use super::LibraryPattern;

pub fn patterns() -> &'static [LibraryPattern] {
    &PATTERNS
}

static PATTERNS: [LibraryPattern; 4] = [
    LibraryPattern {
        name: "rfc4122:uuid",
        source: "RFC 4122",
        flavor: "pcre2",
        regex: r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        structure: "8-4-4-4-12 hex digits",
        test_match: &["550e8400-e29b-41d4-a716-446655440000", "6ba7b810-9dad-11d1-80b4-00c04fd430c8"],
        test_no_match: &["not-a-uuid", "550e8400e29b41d4a716446655440000"],
        flavor_notes: "",
        aliases: &["uuid"],
        category: "identifiers",
    },
    LibraryPattern {
        name: "uuid:v4",
        source: "RFC 4122",
        flavor: "pcre2",
        regex: r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}",
        structure: "8-4-4-4-12 hex with version 4 and variant 1",
        test_match: &["550e8400-e29b-41d4-a716-446655440000", "f47ac10b-58cc-4372-a567-0e02b2c3d479"],
        test_no_match: &["550e8400-e29b-51d4-a716-446655440000", "not-a-uuid"],
        flavor_notes: "Version nibble = 4, variant bits = 10xx",
        aliases: &["uuid-v4"],
        category: "identifiers",
    },
    LibraryPattern {
        name: "semver",
        source: "semver.org 2.0.0",
        flavor: "pcre2",
        regex: r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?(?:\+[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?",
        structure: "MAJOR.MINOR.PATCH[-prerelease][+build]",
        test_match: &["1.0.0", "2.1.3-alpha.1+build.123"],
        test_no_match: &["1.0", "v1.0.0", "01.0.0"],
        flavor_notes: "",
        aliases: &["semantic-version"],
        category: "identifiers",
    },
    LibraryPattern {
        name: "slug",
        source: "common convention",
        flavor: "pcre2",
        regex: r"[a-z0-9]+(?:-[a-z0-9]+)*",
        structure: "lowercase alphanumeric separated by hyphens",
        test_match: &["hello-world", "my-post-123"],
        test_no_match: &["Hello-World", "has spaces", "--double-dash"],
        flavor_notes: "",
        aliases: &["url-slug"],
        category: "identifiers",
    },
];
