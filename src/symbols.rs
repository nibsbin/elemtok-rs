//! The closed vocabulary for elemtok.
//!
//! The 104 two-letter IUPAC element symbols (elements 1-118, canonical
//! casing).
//!
//! Element symbols appear extensively in LLM training data and form a closed,
//! formally defined set, which means models learn the full inventory and
//! reproduce symbols accurately. The 14 single-letter symbols
//! (H, B, C, N, O, F, P, S, K, V, W, Y, I, U) are excluded; mixed-width
//! tokens break positional uniformity and introduce boundary ambiguity.
//!
//! The vocabulary is public and fixed; the search space is always
//! `104^length`.

/// The 104-symbol vocabulary, in canonical two-letter chemical casing, in
/// atomic-number order (periods 1-7; there is no thirteenth period).
pub const ELEMENT_SYMBOLS: [&str; 104] = [
    "He", "Li", "Be", "Ne", "Na", "Mg", "Al", "Si", "Cl", "Ar", "Ca", "Sc", "Ti", "Cr", "Mn", "Fe",
    "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Zr", "Nb", "Mo", "Tc",
    "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd",
    "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "Re", "Os", "Ir",
    "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "Np", "Pu",
    "Am", "Cm", "Bk", "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds",
    "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og",
];

/// Size of the vocabulary. 104 - there is no thirteenth period.
pub const SYMBOL_COUNT: usize = ELEMENT_SYMBOLS.len();

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const EXCLUDED_SINGLE_LETTERS: [&str; 14] = [
        "H", "B", "C", "N", "O", "F", "P", "S", "K", "V", "W", "Y", "I", "U",
    ];

    #[test]
    fn contains_exactly_104_symbols() {
        assert_eq!(ELEMENT_SYMBOLS.len(), 104);
        assert_eq!(SYMBOL_COUNT, 104);
    }

    #[test]
    fn has_no_duplicates() {
        let unique: HashSet<&str> = ELEMENT_SYMBOLS.into_iter().collect();
        assert_eq!(unique.len(), ELEMENT_SYMBOLS.len());
    }

    #[test]
    fn is_entirely_canonical_two_letter_casing() {
        for symbol in ELEMENT_SYMBOLS {
            let mut chars = symbol.chars();
            let first = chars.next().unwrap();
            let second = chars.next().unwrap();
            assert!(chars.next().is_none(), "{symbol} is not two characters");
            assert!(first.is_ascii_uppercase(), "{symbol} must start uppercase");
            assert!(second.is_ascii_lowercase(), "{symbol} must end lowercase");
        }
    }

    #[test]
    fn excludes_every_single_letter_element_symbol() {
        for single in EXCLUDED_SINGLE_LETTERS {
            assert!(!ELEMENT_SYMBOLS.contains(&single));
        }
    }
}
