//! Tokens built from chemical element symbols, designed for use in LLM
//! contexts.
//!
//! ```text
//! FeAuRnCuXeNdCsPbBiTh
//! ```
//!
//! Element symbols are a closed, formally defined set that appears
//! extensively in LLM training data - chemistry textbooks, papers, Wikipedia,
//! the periodic table itself. Models learn the full inventory, which means
//! they reproduce symbols accurately and can identify invalid ones without a
//! lookup. Each symbol contributes `log2(104) ≈ 6.7 bits` of CSPRNG entropy.
//!
//! ```
//! let token = elemtok::generate(None).unwrap();
//! assert!(elemtok::validate(&token));
//! assert_eq!(token.len(), 20); // 10 symbols * 2 chars
//! ```
//!
//! See the [README](https://github.com/nibsbin/elemtok-rs) for the full
//! design rationale, entropy table, and threat model.

mod error;
mod generate;
mod rng;
mod symbols;
mod validate;

pub use error::Error;
pub use generate::{generate, generate_from, MAX_LENGTH};
pub use symbols::{ELEMENT_SYMBOLS, SYMBOL_COUNT};
pub use validate::validate;
