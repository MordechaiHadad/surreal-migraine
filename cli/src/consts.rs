use regex::Regex;
use std::sync::LazyLock;

/// Matches a leading numeric prefix like `000_`, capturing the numeric part without leading zeros.
pub static PREFIX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:0*)(\d+)_").unwrap());

/// Collapses consecutive underscores.
pub static UNDERSCORE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"_+").unwrap());

/// Characters invalid on Windows file names (double-quote included).
pub static INVALID_CHARS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[/\\:*?"<>|]"#).unwrap());
