# elemtok

Tokens built from chemical element symbols, designed for use in LLM contexts.

```
FeAuRnCuXeNdCsPbBiTh
```

Element symbols are a closed, formally defined set that appears extensively in
LLM training data — chemistry textbooks, papers, Wikipedia, the periodic table
itself. Models learn the full inventory, which means they reproduce symbols
accurately and can identify invalid ones without a lookup. Each symbol
contributes `log2(104) ≈ 6.7 bits` of CSPRNG entropy.

This is a Rust port of the [TypeScript `elemtok`](https://github.com/nibsbin/elemtok)
package; the API and behavior are deliberately identical.

## Design

**Vocabulary:** 104 two-letter IUPAC element symbols (elements 1–118, canonical
casing). The 14 single-letter symbols (H, B, C, N, O, F, P, S, K, V, W, Y, I,
U) are excluded. Mixed-width identifiers break uniform positional structure and
introduce boundary ambiguity; a lone `C` adjacent to `Fe` is where LLM
transcription errors occur.

**Entropy:** Each symbol is drawn uniformly via rejection sampling over a 16-bit
window, eliminating modulo bias across 104 choices. Per-symbol entropy is the
full `log2(104) ≈ 6.7 bits` with no skew.

**Validation:** The vocabulary is closed. Any two-character string either is or
is not an element symbol — no ambiguity, no variants. `validate` checks only
public structure; it is a format gate, not an authentication check.

## Install

```bash
cargo add elemtok
```

Entropy is sourced via the [`getrandom`](https://docs.rs/getrandom) crate,
which wraps the operating system's CSPRNG on every supported platform.

## Quick start

```rust
use elemtok::{generate, validate};

let token = generate(None)?;        // "FeAuRnCuXeNdCsPbBiTh"
assert!(validate(&token));          // true
assert!(!validate("feaurncuxe"));   // false  (case-sensitive)
assert!(!validate("XxAuRnCuXe"));   // false  (Xx is not an element)

generate(Some(8))?;                 // 8 symbols ≈ 53.6 bits
# Ok::<(), elemtok::Error>(())
```

## API

### `generate(length: Option<usize>) -> Result<String, Error>`

Draws each symbol uniformly from the 104-symbol vocabulary using a CSPRNG with
rejection sampling.

| Argument | Type            | Default | Description                                 |
| -------- | --------------- | ------- | -------------------------------------------- |
| `length` | `Option<usize>` | `10`    | Number of symbols. Must be in `[1, 65536]`. |

```rust
elemtok::generate(None)?;       // "FeAuRnCuXeNdCsPbBiTh"
elemtok::generate(Some(8))?;    // 8 symbols ≈ 53.6 bits
# Ok::<(), elemtok::Error>(())
```

Returns `Err(Error::InvalidLength { .. })` if `length` is outside `[1, 65536]`.
The upper bound prevents an attacker-controlled length from causing an
unbounded allocation. Tokens are a bare concatenation; split if needed with
`token.as_bytes().chunks(2)`.

### `generate_from(next16: impl FnMut() -> u16, length: Option<usize>) -> Result<String, Error>`

The seam behind `generate` (`generate(length)` is `generate_from(secure_uint16,
length)`), exposed for **deterministic derivation**: supply your own 16-bit
source (`FnMut() -> u16` returning a value in `[0, 65536)`) to derive a stable
token from a seed — a hash digest, a UUID, a row id. Same rejection sampling,
so no modulo bias for any source; elemtok takes no opinion on the hash or what
the seed means. Two caveats: `next16` must be inexhaustible (a fixed digest
slice can run dry mid-token, since rejection sampling resamples), and the
token inherits the entropy of `next16`, **not** `104^length` — a guessable seed
yields a guessable token. If you don't need determinism, use `generate`.

### `validate(token: &str) -> bool`

Returns `true` only if the input is a non-empty concatenation of known
two-letter symbols, case-sensitive, with no delimiters or trailing characters.
Any unknown two-character chunk, odd-length string, or empty input returns
`false`.

`validate` checks format only — a `true` result does not mean the token is
authorized. Authorization requires a constant-time lookup against a stored
secret; `validate` leaks nothing about secrets because it examines only the
public symbol vocabulary.

```rust
use elemtok::validate;

assert!(validate("FeAuRnCuXe"));
assert!(!validate("feaurncuxe")); // false  (case-sensitive)
assert!(!validate("XxFe"));       // false  (Xx is not an element)
assert!(!validate("Fe-Au"));      // false  (no delimiters)
```

`ELEMENT_SYMBOLS` (`[&str; 104]`) and `SYMBOL_COUNT` (`104`) are also exported.

## Entropy

One symbol = `log2(104) ≈ 6.7 bits`.

| Length | Entropy                 |
| ------ | ------------------------ |
| 4      | ≈ 26.8 bits              |
| 6      | ≈ 40.2 bits              |
| 8      | ≈ 53.6 bits              |
| **10** | **≈ 67 bits** (default)  |
| 12     | ≈ 80.4 bits              |
| 20     | ≈ 134 bits               |

Target entropy ÷ 6.7 = required length (e.g. 128 bits → 20 symbols).

Compared to BIP39: BIP39 yields 11 bits per word from a 2048-word vocabulary
and includes a checksum. elemtok yields 6.7 bits per two-character atom with
no checksum.

## Threat model

Rate-limited, short-lived identifiers. Not intended for offline-attack scenarios.

- Randomness from the OS CSPRNG via `getrandom`; no non-cryptographic PRNG is
  ever used.
- Rejection sampling over a 16-bit window; no modulo bias; all 104 symbols
  equiprobable. Search space is `104^length`.
- No checksum. The database lookup is the validity check.
- Default (length 10, ≈ 67 bits) is sized for rate-limited tokens. For
  offline-attack resistance use length ≥ 12 (≈ 80 bits) or ≥ 20 (≈ 128 bits).

## License

MIT © Nibs
