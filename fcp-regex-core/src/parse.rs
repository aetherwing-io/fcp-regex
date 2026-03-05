//! Minimal FCP operation parser — tokenizer + parsed op.
//!
//! Extracted from fcpcore for use without MCP dependencies.

use std::collections::HashMap;

/// A successfully parsed FCP operation.
#[derive(Debug, Clone)]
pub struct ParsedOp {
    pub verb: String,
    pub positionals: Vec<String>,
    pub params: HashMap<String, String>,
    pub raw: String,
}

/// Parse an FCP operation string into a structured ParsedOp.
pub fn parse_op(input: &str) -> Result<ParsedOp, String> {
    let raw = input.trim().to_string();
    let tokens = tokenize(&raw);

    if tokens.is_empty() {
        return Err("empty operation".to_string());
    }

    let verb = tokens[0].to_lowercase();
    let mut positionals = Vec::new();
    let mut params = HashMap::new();

    for token in &tokens[1..] {
        if is_key_value(token) {
            let (key, value) = parse_key_value(token);
            params.insert(key, value);
        } else {
            positionals.push(token.clone());
        }
    }

    Ok(ParsedOp {
        verb,
        positionals,
        params,
        raw,
    })
}

/// Tokenize splits an FCP operation string into tokens, handling quoted strings.
pub fn tokenize(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let n = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < n {
        while i < n && bytes[i] == b' ' {
            i += 1;
        }
        if i >= n {
            break;
        }

        if bytes[i] == b'"' {
            i += 1;
            let mut token = String::new();
            while i < n && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < n {
                    let next = bytes[i + 1];
                    if next == b'n' {
                        token.push('\n');
                        i += 2;
                    } else {
                        i += 1;
                        token.push(bytes[i] as char);
                        i += 1;
                    }
                } else {
                    token.push(bytes[i] as char);
                    i += 1;
                }
            }
            if i < n {
                i += 1;
            }
            tokens.push(token);
        } else {
            let mut token = String::new();
            while i < n && bytes[i] != b' ' {
                if bytes[i] == b'"' {
                    token.push('"');
                    i += 1;
                    while i < n && bytes[i] != b'"' {
                        if bytes[i] == b'\\' && i + 1 < n {
                            let next = bytes[i + 1];
                            if next == b'n' {
                                token.push('\n');
                                i += 2;
                            } else {
                                i += 1;
                                token.push(bytes[i] as char);
                                i += 1;
                            }
                        } else {
                            token.push(bytes[i] as char);
                            i += 1;
                        }
                    }
                    if i < n {
                        token.push('"');
                        i += 1;
                    }
                } else {
                    token.push(bytes[i] as char);
                    i += 1;
                }
            }
            tokens.push(token.replace("\\n", "\n"));
        }
    }

    tokens
}

/// Returns true if the token is a key:value pair.
fn is_key_value(token: &str) -> bool {
    if token.starts_with('@') {
        return false;
    }
    if matches!(token, "->" | "<->" | "--") {
        return false;
    }
    let idx = match token.find(':') {
        Some(i) => i,
        None => return false,
    };
    if idx == 0 || idx >= token.len() - 1 {
        return false;
    }
    let key = &token[..idx];
    key.chars()
        .all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'))
}

/// Parse a key:value token into its key and value parts.
fn parse_key_value(token: &str) -> (String, String) {
    let idx = token.find(':').unwrap();
    let key = token[..idx].to_string();
    let mut value = token[idx + 1..].to_string();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value = value[1..value.len() - 1].to_string();
    }
    (key, value)
}

/// Finds the closest candidate for a misspelled input using Levenshtein distance.
pub fn suggest(input: &str, candidates: &[&str]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let input_lower = input.to_lowercase();
    let mut best: Option<&str> = None;
    let mut best_dist = 999;

    for &candidate in candidates {
        let dist = levenshtein(&input_lower, &candidate.to_lowercase());
        if dist < best_dist {
            best_dist = dist;
            best = Some(candidate);
        }
    }

    if best_dist <= 3 {
        best.map(|s| s.to_string())
    } else {
        None
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    let mut prev: Vec<usize> = (0..=n).collect();

    for i in 1..=m {
        let mut prev_diag = prev[0];
        prev[0] = i;
        #[allow(clippy::needless_range_loop)]
        for j in 1..=n {
            let temp = prev[j];
            if a_bytes[i - 1] == b_bytes[j - 1] {
                prev[j] = prev_diag;
            } else {
                let min_val = prev_diag.min(prev[j - 1]).min(prev[j]);
                prev[j] = 1 + min_val;
            }
            prev_diag = temp;
        }
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_op_simple() {
        let r = parse_op("define digits any:digit+").unwrap();
        assert_eq!(r.verb, "define");
        assert_eq!(r.positionals, vec!["digits"]);
        // "any:digit+" is parsed as key:value (any → digit+)
        assert_eq!(r.params["any"], "digit+");
    }

    #[test]
    fn test_parse_op_with_params() {
        let r = parse_op("compile digits anchored:true").unwrap();
        assert_eq!(r.verb, "compile");
        assert_eq!(r.positionals, vec!["digits"]);
        assert_eq!(r.params["anchored"], "true");
    }

    #[test]
    fn test_parse_op_empty() {
        assert!(parse_op("").is_err());
    }

    #[test]
    fn test_tokenize_quoted() {
        let got = tokenize(r#"label A "hello world""#);
        assert_eq!(got, vec!["label", "A", "hello world"]);
    }

    #[test]
    fn test_suggest_close_match() {
        assert_eq!(suggest("defne", &["define", "compile"]), Some("define".to_string()));
    }

    #[test]
    fn test_suggest_no_match() {
        assert_eq!(suggest("zzzzz", &["define", "compile"]), None);
    }
}
