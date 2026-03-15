use fst::Set;

/// FST-backed word dictionary for fast membership testing.
pub struct FstDictionary {
    set: Set<Vec<u8>>,
}

impl FstDictionary {
    /// Load a dictionary from pre-built FST bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, fst::Error> {
        let set = Set::new(bytes)?;
        Ok(Self { set })
    }

    /// Build FST bytes from a list of words. Words are normalized to uppercase,
    /// sorted, and deduplicated. Returns the raw FST bytes for saving to disk.
    pub fn build(words: &[&str]) -> Result<Vec<u8>, fst::Error> {
        let mut sorted: Vec<String> = words
            .iter()
            .map(|w| w.trim().to_ascii_uppercase())
            .filter(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic()))
            .collect();
        sorted.sort();
        sorted.dedup();

        let mut builder = fst::SetBuilder::memory();
        for word in &sorted {
            builder.insert(word.as_bytes())?;
        }
        builder.into_inner()
    }

    /// Check if a word is in the dictionary. Case-insensitive.
    pub fn contains(&self, word: &str) -> bool {
        let normalized = word.trim().to_ascii_uppercase();
        self.set.contains(normalized.as_bytes())
    }

    /// Number of words in the dictionary.
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_query() {
        let words = vec!["hello", "world", "HELLO", "test"];
        let bytes = FstDictionary::build(&words).unwrap();
        let dict = FstDictionary::from_bytes(bytes).unwrap();

        assert!(dict.contains("hello"));
        assert!(dict.contains("HELLO"));
        assert!(dict.contains("Hello"));
        assert!(dict.contains("world"));
        assert!(dict.contains("test"));
        assert!(!dict.contains("missing"));
        assert_eq!(dict.len(), 3); // hello, test, world (deduped)
    }

    #[test]
    fn empty_and_whitespace_filtered() {
        let words = vec!["", "  ", "valid", " spaced "];
        let bytes = FstDictionary::build(&words).unwrap();
        let dict = FstDictionary::from_bytes(bytes).unwrap();

        assert!(dict.contains("valid"));
        assert!(dict.contains("spaced"));
        assert_eq!(dict.len(), 2);
    }

    #[test]
    fn non_alpha_filtered() {
        let words = vec!["good", "bad-word", "also.bad", "fine"];
        let bytes = FstDictionary::build(&words).unwrap();
        let dict = FstDictionary::from_bytes(bytes).unwrap();

        assert!(dict.contains("good"));
        assert!(dict.contains("fine"));
        assert!(!dict.contains("bad-word"));
        assert_eq!(dict.len(), 2);
    }
}
