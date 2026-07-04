use crate::error::Error;
use crate::rng::{random_index, random_index_from};
use crate::symbols::{ELEMENT_SYMBOLS, SYMBOL_COUNT};

/// Default number of symbols in a token: 10 symbols ≈ 67 bits of entropy.
const DEFAULT_LENGTH: usize = 10;

/// Maximum number of symbols in a token (`2^16`). A defensive upper bound so
/// an unbounded or attacker-influenced `length` cannot turn one call into an
/// out-of-memory / unbounded-loop denial of service. 65536 symbols is already
/// ~439 kbits of entropy, so this never constrains real use.
pub const MAX_LENGTH: usize = 65536;

/// Build a token of `length` symbols, drawing each index from `sample`. Both
/// public entry points funnel through here so length validation and the
/// concatenation loop live once; `sample` is fallible so a CSPRNG failure
/// propagates instead of panicking.
fn build_token<F>(length: Option<usize>, mut sample: F) -> Result<String, Error>
where
    F: FnMut() -> Result<u32, Error>,
{
    let length = length.unwrap_or(DEFAULT_LENGTH);

    if !(1..=MAX_LENGTH).contains(&length) {
        return Err(Error::InvalidLength {
            got: length,
            max: MAX_LENGTH,
        });
    }

    let mut token = String::with_capacity(length * 2);
    for _ in 0..length {
        let index = sample()?;
        token.push_str(ELEMENT_SYMBOLS[index as usize]);
    }
    Ok(token)
}

/// The seam [`generate`] is built on, exposed for deterministic derivation:
/// supply your own 16-bit source instead of the CSPRNG to derive a stable
/// token from a seed. Same rejection sampling, so no modulo bias for any
/// source.
///
/// `next16` must return an integer in `[0, 65536)` and be inexhaustible - a
/// fixed digest slice can run dry mid-token, since rejection sampling
/// resamples. The token inherits the entropy of `next16`, NOT `104^length`.
/// See the README.
///
/// `length` defaults to [`DEFAULT_LENGTH`] (10) when `None`.
///
/// # Errors
///
/// Returns [`Error::InvalidLength`] if `length` is `Some` value outside
/// `[1, MAX_LENGTH]`.
pub fn generate_from<F>(mut next16: F, length: Option<usize>) -> Result<String, Error>
where
    F: FnMut() -> u16,
{
    build_token(length, || {
        random_index_from(SYMBOL_COUNT as u32, &mut next16)
    })
}

/// Generate a token from element symbols.
///
/// Returns `length` symbols (default 10) concatenated with no delimiter, e.g.
/// `"FeAuRnCuXe"`. Each symbol is drawn uniformly from the 104-symbol
/// vocabulary using a CSPRNG with rejection sampling, contributing ~6.7 bits
/// of entropy.
///
/// Convenience wrapper over [`generate_from`] with the platform CSPRNG as the
/// source; reach for `generate_from` only when you need deterministic
/// derivation.
///
/// # Errors
///
/// Returns [`Error::InvalidLength`] if `length` is `Some` value outside
/// `[1, MAX_LENGTH]`, or [`Error::NoSecureRandom`] if the platform has no
/// secure random source.
pub fn generate(length: Option<usize>) -> Result<String, Error> {
    build_token(length, || random_index(SYMBOL_COUNT as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate;
    use std::collections::HashSet;

    fn is_canonical_token_of_length(token: &str, length: usize) -> bool {
        if token.len() != length * 2 {
            return false;
        }
        token
            .as_bytes()
            .chunks_exact(2)
            .all(|c| c[0].is_ascii_uppercase() && c[1].is_ascii_lowercase())
    }

    #[test]
    fn produces_10_concatenated_canonical_symbols_by_default() {
        for _ in 0..200 {
            let token = generate(None).unwrap();
            assert!(is_canonical_token_of_length(&token, 10));
        }
    }

    #[test]
    fn draws_every_segment_from_the_vocabulary() {
        let known: HashSet<&str> = ELEMENT_SYMBOLS.into_iter().collect();
        for _ in 0..200 {
            let token = generate(None).unwrap();
            for chunk in token.as_bytes().chunks_exact(2) {
                let s = std::str::from_utf8(chunk).unwrap();
                assert!(known.contains(s));
            }
        }
    }

    #[test]
    fn respects_the_length_option() {
        for length in [1usize, 3, 8, 32] {
            assert_eq!(generate(Some(length)).unwrap().len(), length * 2);
        }
    }

    #[test]
    fn rejects_invalid_length() {
        assert!(generate(Some(0)).is_err());
        assert!(generate(Some(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn caps_length_to_guard_against_unbounded_allocation_dos() {
        assert_eq!(generate(Some(MAX_LENGTH)).unwrap().len(), MAX_LENGTH * 2);
        assert!(generate(Some(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn round_trips_through_validate() {
        for length in [1usize, 5, 12] {
            assert!(validate(&generate(Some(length)).unwrap()));
        }
    }

    /// Build a 16-bit source yielding the given values in order, then panics
    /// if drawn from again.
    fn words_from(values: &[u16]) -> impl FnMut() -> u16 + '_ {
        let mut i = 0;
        move || {
            let v = values[i];
            i += 1;
            v
        }
    }

    #[test]
    fn generate_from_is_deterministic_for_a_deterministic_source() {
        let seed = [1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let a = generate_from(words_from(&seed), None).unwrap();
        let b = generate_from(words_from(&seed), None).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn generate_from_maps_each_accepted_word_to_element_symbols_word_mod_symbol_count() {
        // All values < 65520 are accepted, so word % 104 selects the symbol.
        let words = [0u16, 1, 103, 104, 150, 65519];
        let token = generate_from(words_from(&words), Some(words.len())).unwrap();
        let expected: String = words
            .iter()
            .map(|&w| ELEMENT_SYMBOLS[w as usize % SYMBOL_COUNT])
            .collect();
        assert_eq!(token, expected);
    }

    #[test]
    fn generate_from_inherits_no_bias_rejection_sampling() {
        // 65530 >= 65520 is rejected; the next word (7) is used: 7 % 104 = 7.
        let token = generate_from(words_from(&[65530, 7]), Some(1)).unwrap();
        assert_eq!(token, ELEMENT_SYMBOLS[7]);
    }

    #[test]
    fn generate_from_draws_only_from_the_vocabulary_and_round_trips() {
        let known: HashSet<&str> = ELEMENT_SYMBOLS.into_iter().collect();
        let mut counter: u16 = 0;
        for length in [1usize, 8, 32] {
            let mut src = counter;
            let token = generate_from(
                || {
                    src = src.wrapping_add(12345);
                    src
                },
                Some(length),
            )
            .unwrap();
            counter = src;
            assert_eq!(token.len(), length * 2);
            for chunk in token.as_bytes().chunks_exact(2) {
                assert!(known.contains(std::str::from_utf8(chunk).unwrap()));
            }
            assert!(validate(&token));
        }
    }

    #[test]
    fn generate_from_validates_length_the_same_way_generate_does() {
        assert!(generate_from(|| 0, Some(0)).is_err());
        assert!(generate_from(|| 0, Some(MAX_LENGTH + 1)).is_err());
        assert_eq!(generate_from(|| 0, Some(3)).unwrap().len(), 6);
    }

    #[test]
    fn generate_backs_generate_from_over_the_csprng() {
        assert!(validate(&generate(None).unwrap()));
    }
}
