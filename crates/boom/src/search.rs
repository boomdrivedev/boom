//! A search of the claims that stand by their words (claim-search;
//! record 0009): the words an agent gives, each searched as written,
//! never as a query language (design 2).

/// The words searched: those given, less each holding no letter or
/// digit, in the order given.
pub fn words(given: &[String]) -> Vec<String> {
    given
        .iter()
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .cloned()
        .collect()
}

/// The FTS5 query of `words`: each written as a string, its `"`
/// doubled, so FTS5 reads no operator in it, and the strings joined by
/// spaces, which FTS5 reads as every one of them.
pub fn query(words: &[String]) -> String {
    words
        .iter()
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn given(words: &[&str]) -> Vec<String> {
        words.iter().map(|&word| word.to_owned()).collect()
    }

    #[test]
    fn a_word_is_written_as_a_string() {
        assert_eq!(query(&given(&["currency"])), "\"currency\"");
        assert_eq!(query(&given(&["OR"])), "\"OR\"");
        assert_eq!(query(&given(&["currency*"])), "\"currency*\"");
        assert_eq!(query(&given(&["core.2"])), "\"core.2\"");
        assert_eq!(query(&given(&["(NEAR"])), "\"(NEAR\"");
    }

    #[test]
    fn a_quote_in_a_word_is_doubled() {
        assert_eq!(query(&given(&["\"amount"])), "\"\"\"amount\"");
        assert_eq!(query(&given(&["say\"so\""])), "\"say\"\"so\"\"\"");
    }

    #[test]
    fn a_word_with_no_letter_or_digit_is_dropped() {
        assert_eq!(
            words(&given(&["***", "amount", "-", "\"\"", "naïve", "2", "…"])),
            given(&["amount", "naïve", "2"])
        );
        assert!(words(&given(&["***", "--", "()"])).is_empty());
    }

    #[test]
    fn the_words_are_joined_so_each_must_be_found() {
        assert_eq!(
            query(&given(&["amount", "currency", "core.2"])),
            "\"amount\" \"currency\" \"core.2\""
        );
    }
}
