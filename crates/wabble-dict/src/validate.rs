/// Normalize a word for dictionary lookup (uppercase, trimmed).
pub fn normalize_word(word: &str) -> String {
    word.trim().to_ascii_uppercase()
}

/// Check if a string contains only ASCII alphabetic characters.
pub fn is_valid_word_chars(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize() {
        assert_eq!(normalize_word("  hello  "), "HELLO");
        assert_eq!(normalize_word("World"), "WORLD");
    }

    #[test]
    fn valid_chars() {
        assert!(is_valid_word_chars("hello"));
        assert!(is_valid_word_chars("WORLD"));
        assert!(!is_valid_word_chars(""));
        assert!(!is_valid_word_chars("hello world"));
        assert!(!is_valid_word_chars("test123"));
        assert!(!is_valid_word_chars("hyphen-ated"));
    }
}
