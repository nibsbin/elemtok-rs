# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-04

Initial Rust port of [`elemtok`](https://github.com/nibsbin/elemtok), tracking
the TypeScript package's 2.1.0 API and behavior.

### Added

- `generate(length: Option<usize>) -> Result<String, Error>` — generate a
  token from the 104-symbol element vocabulary using the OS CSPRNG
  (via [`getrandom`](https://docs.rs/getrandom)).
- `generate_from(next16: impl FnMut() -> u16, length: Option<usize>) -> Result<String, Error>` —
  the injectable-source seam `generate` is built on, for deterministic
  derivation from a caller-supplied 16-bit stream.
- `validate(token: &str) -> bool` — format-only validation against the closed
  vocabulary.
- `ELEMENT_SYMBOLS: [&str; 104]` and `SYMBOL_COUNT: usize` — the vocabulary and
  its size.
- `MAX_LENGTH: usize` (65536) — the denial-of-service guard on `length`.
- `Error` — the error type returned for an out-of-range `length` or an
  unavailable secure random source.

[0.1.0]: https://github.com/nibsbin/elemtok-rs/releases/tag/v0.1.0
