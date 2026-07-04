use std::collections::HashSet;
use std::sync::OnceLock;

use crate::symbols::ELEMENT_SYMBOLS;

/// Width of every element symbol, in bytes (every symbol is pure ASCII).
const SYMBOL_WIDTH: usize = 2;

fn known() -> &'static HashSet<&'static str> {
    static KNOWN: OnceLock<HashSet<&'static str>> = OnceLock::new();
    KNOWN.get_or_init(|| ELEMENT_SYMBOLS.into_iter().collect())
}

/// Validate that a token is a concatenation of known element symbols.
///
/// This is a syntax gate, not authentication: it confirms the token is
/// well-formed against the public vocabulary, nothing more. A `true` result
/// does not mean the token is authorized - verifying that is the caller's job
/// (look the token up and compare the stored secret in constant time).
/// Because it only inspects public, syntactic structure, its early return
/// leaks nothing secret.
///
/// Validation is strict and case-sensitive: only canonical chemical casing is
/// accepted (`"FeAu"` is valid; `"feau"` and `"FEAU"` are not). There is no
/// trimming and no checksum - an unknown segment fails before any database
/// lookup. Every element symbol is exactly two ASCII characters, so the token
/// is split into 2-byte chunks; an empty token, a token whose length is not a
/// whole number of symbols, or any chunk that is not an element symbol all
/// return `false`.
pub fn validate(token: &str) -> bool {
    let bytes = token.as_bytes();

    if bytes.is_empty() || bytes.len() % SYMBOL_WIDTH != 0 {
        return false;
    }

    let known = known();
    bytes
        .chunks_exact(SYMBOL_WIDTH)
        .all(|chunk| match std::str::from_utf8(chunk) {
            Ok(symbol) => known.contains(symbol),
            Err(_) => false,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_canonical_token() {
        assert!(validate("FeAuRnCuXe"));
        assert!(validate("Fe"));
    }

    #[test]
    fn rejects_an_empty_token() {
        assert!(!validate(""));
    }

    #[test]
    fn rejects_a_token_that_is_not_a_whole_number_of_symbols() {
        assert!(!validate("Fea"));
        assert!(!validate("FeAuR"));
    }

    #[test]
    fn is_case_sensitive() {
        assert!(!validate("feau"));
        assert!(!validate("FEAU"));
        assert!(!validate("FeAU"));
    }

    #[test]
    fn rejects_well_formed_but_unknown_symbols() {
        assert!(!validate("XxFe"));
    }

    #[test]
    fn rejects_excluded_single_letter_symbols() {
        assert!(!validate("U"));
    }

    #[test]
    fn rejects_surrounding_whitespace() {
        assert!(!validate(" FeAu"));
        assert!(!validate("FeAu "));
    }

    #[test]
    fn rejects_delimiters() {
        assert!(!validate("Fe-Au"));
        assert!(!validate("Fe.Au"));
    }

    #[test]
    fn rejects_non_ascii_without_panicking() {
        assert!(!validate("Fé"));
        assert!(!validate("日本語abc"));
    }
}
