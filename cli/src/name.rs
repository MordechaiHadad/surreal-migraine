use crate::consts::{INVALID_CHARS_RE, PREFIX_RE, UNDERSCORE_RE};

/// Sanitize a migration name into a filesystem-safe component.
/// Replaces whitespace with `_` and removes invalid Windows chars.
pub fn sanitize_name(name: &str) -> String {
    let s = name.trim().replace(|c: char| c.is_whitespace(), "_");
    let out = INVALID_CHARS_RE.replace_all(&s, "").to_string();
    let out = UNDERSCORE_RE.replace_all(&out, "_").to_string();
    tracing::trace!(original = name, sanitized = %out);
    out.trim_matches('_').to_string()
}

/// Parse a leading numeric prefix like "001_foo.surql" -> Some(1)
pub fn parse_numeric_prefix(file_name: &str) -> Option<u64> {
    PREFIX_RE
        .captures(file_name)
        .and_then(|caps| caps.get(1).and_then(|m| m.as_str().parse::<u64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_basic() {
        assert_eq!(sanitize_name("Create Users"), "Create_Users");
        assert_eq!(sanitize_name("  space  test  "), "space_test");
        assert_eq!(sanitize_name("weird:/\\name"), "weirdname");
    }

    #[test]
    fn parse_prefix_ok() {
        assert_eq!(parse_numeric_prefix("001_init.surql"), Some(1));
        assert_eq!(parse_numeric_prefix("000_foo.surql"), Some(0));
        assert_eq!(parse_numeric_prefix("10_bar.surql"), Some(10));
    }

    #[test]
    fn parse_prefix_none() {
        assert_eq!(parse_numeric_prefix("init.surql"), None);
        assert_eq!(parse_numeric_prefix("abc_123.surql"), None);
    }
}
