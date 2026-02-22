/// Decode a URL percent-encoded string (e.g., `%20` → space, `%27` → `'`).
///
/// This handles standard percent-encoding as produced by Python's `urllib.parse.quote()`.
/// The `+` character is NOT treated as space (use `%20` for spaces).
pub fn url_decode(input: &str) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes();

    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars
                .next()
                .ok_or_else(|| format!("Incomplete percent-encoding at end of: {input}"))?;
            let lo = chars
                .next()
                .ok_or_else(|| format!("Incomplete percent-encoding at end of: {input}"))?;
            let hex = [hi, lo];
            let hex_str = std::str::from_utf8(&hex)
                .map_err(|_| format!("Invalid percent-encoding in: {input}"))?;
            let decoded = u8::from_str_radix(hex_str, 16)
                .map_err(|_| format!("Invalid hex '{hex_str}' in percent-encoding: {input}"))?;
            bytes.push(decoded);
        } else {
            bytes.push(b);
        }
    }

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 after percent-decoding: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_encoding() {
        assert_eq!(url_decode("hello").unwrap(), "hello");
    }

    #[test]
    fn test_space() {
        assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
    }

    #[test]
    fn test_single_quote() {
        assert_eq!(url_decode("it%27s").unwrap(), "it's");
    }

    #[test]
    fn test_double_quote() {
        assert_eq!(url_decode("say%20%22hi%22").unwrap(), "say \"hi\"");
    }

    #[test]
    fn test_ampersand() {
        assert_eq!(url_decode("a%26b").unwrap(), "a&b");
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(
            url_decode("K-On%21%20%28Movie%29").unwrap(),
            "K-On! (Movie)"
        );
    }

    #[test]
    fn test_plus_not_decoded_as_space() {
        assert_eq!(url_decode("a+b").unwrap(), "a+b");
    }

    #[test]
    fn test_japanese_utf8() {
        // "覚え" in UTF-8 is E8 A6 9A E3 81 88
        assert_eq!(url_decode("%E8%A6%9A%E3%81%88").unwrap(), "覚え");
    }

    #[test]
    fn test_mixed_encoded_and_plain() {
        assert_eq!(
            url_decode("Fuwa%20Fuwa%20Time%20%285-nin%20Ver.%29").unwrap(),
            "Fuwa Fuwa Time (5-nin Ver.)"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(url_decode("").unwrap(), "");
    }

    #[test]
    fn test_incomplete_percent_at_end() {
        assert!(url_decode("hello%2").is_err());
    }

    #[test]
    fn test_incomplete_percent_single_char() {
        assert!(url_decode("hello%").is_err());
    }

    #[test]
    fn test_invalid_hex() {
        assert!(url_decode("hello%ZZ").is_err());
    }

    #[test]
    fn test_all_special_shell_chars() {
        // Characters that commonly cause shell quoting issues
        assert_eq!(
            url_decode("%27%22%60%24%5C%26%7C%3B%3C%3E%28%29%7B%7D%5B%5D%21%23%2A%3F%7E").unwrap(),
            "'\"`$\\&|;<>(){}[]!#*?~"
        );
    }
}
