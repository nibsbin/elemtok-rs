//! Cryptographically secure, unbiased index sampling.
//!
//! Built on the [`getrandom`] crate, which wraps the operating system's CSPRNG
//! (`getrandom(2)`/`BCryptGenRandom`/`arc4random_buf`/...) on every supported
//! platform. `rand::random` / a non-cryptographic PRNG is never used.

use crate::error::Error;

/// Size of the sampling window: a 16-bit value drawn from two random bytes.
const DRAW_SPACE: u32 = 1 << 16; // 65536

/// Read one cryptographically secure 16-bit value in `[0, 65536)`.
///
/// Both bytes come from a single [`getrandom::getrandom`] call, so each draw
/// costs one CSPRNG invocation regardless of the rejection-sampling retry rate
/// (which is vanishingly small at this width - see [`random_index_from`]).
///
/// Returns [`Error::NoSecureRandom`] if the platform has no secure source
/// rather than silently degrading.
pub(crate) fn secure_uint16() -> Result<u16, Error> {
    let mut buffer = [0u8; 2];
    getrandom::getrandom(&mut buffer).map_err(|_| Error::NoSecureRandom)?;
    Ok(u16::from_be_bytes(buffer))
}

/// Uniformly sample an integer in `[0, n)` for `1 <= n <= 65536` by rejection
/// sampling over `draw`, a fallible 16-bit source. Both public entry points
/// funnel through here so the range check, reject window, and loop live once;
/// `draw` returns `Result` so a fallible CSPRNG source can surface its error
/// instead of panicking inside the closure.
///
/// Why rejection sampling: 65536 is not a multiple of most `n`, so a naive
/// `value % n` over-represents the low residues (modulo bias). We accept only
/// values in `[0, max)`, where `max` is the largest multiple of `n` that fits
/// the 16-bit window; within that window every residue class is equally
/// populated.
///
/// Why a 16-bit window: the rejection zone is `65536 - max`, always `< n` out
/// of 65536, so the reject rate is at most ~0.4% for any `n` in range - and
/// just `16 / 65536 ≈ 0.024%` for the 104-symbol vocabulary (`max = 65520`).
/// Sampling a single byte instead would reject up to ~50% (`48 / 256 ≈
/// 18.75%` at `n = 104`), forcing far more resampling for the same result.
fn rejection_sample<F>(n: u32, mut draw: F) -> Result<u32, Error>
where
    F: FnMut() -> Result<u32, Error>,
{
    if !(1..=DRAW_SPACE).contains(&n) {
        return Err(Error::InvalidLength {
            got: n as usize,
            max: DRAW_SPACE as usize,
        });
    }

    let max = (DRAW_SPACE / n) * n;
    loop {
        let v = draw()?;
        if v < max {
            return Ok(v % n);
        }
        // otherwise reject and resample
    }
}

/// Uniformly sample an integer in `[0, n)` from an injectable 16-bit source.
/// The source is injectable so tests can drive it with a deterministic stream
/// and prove the no-bias property exactly.
pub(crate) fn random_index_from<F>(n: u32, mut next16: F) -> Result<u32, Error>
where
    F: FnMut() -> u16,
{
    rejection_sample(n, || Ok(next16() as u32))
}

/// Uniformly sample an integer in `[0, n)` from the platform CSPRNG, free of
/// modulo bias. A `NoSecureRandom` error from the source propagates to the
/// caller.
pub(crate) fn random_index(n: u32) -> Result<u32, Error> {
    rejection_sample(n, || Ok(secure_uint16()? as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 16-bit source that yields the given values in order, then
    /// panics if drawn from again (mirrors the TS test suite's "exhausted"
    /// source, which throws).
    fn words_from(values: &[u16]) -> impl FnMut() -> u16 + '_ {
        let mut i = 0;
        move || {
            let v = values[i];
            i += 1;
            v
        }
    }

    #[test]
    fn random_index_always_in_range() {
        for n in [2u32, 3, 16, 100, 104, 128, 200, 256] {
            for _ in 0..2000 {
                let v = random_index(n).unwrap();
                assert!(v < n);
            }
        }
    }

    #[test]
    fn random_index_rejects_out_of_range_n() {
        assert!(matches!(
            random_index(0),
            Err(Error::InvalidLength { got: 0, .. })
        ));
        assert!(matches!(
            random_index(65537),
            Err(Error::InvalidLength { got: 65537, .. })
        ));
    }

    #[test]
    fn random_index_from_skips_reject_zone() {
        // For n=104, max=65520: values >= 65520 are rejected.
        let mut src = words_from(&[65530, 5]); // 65530 rejected, 5 accepted -> 5 % 104
        assert_eq!(random_index_from(104, &mut src).unwrap(), 5);
    }

    #[test]
    fn random_index_from_maps_accepted_value_via_modulo() {
        // 150 < 65520, so 150 % 104 = 46.
        let mut src = words_from(&[150]);
        assert_eq!(random_index_from(104, &mut src).unwrap(), 46);
    }

    #[test]
    #[should_panic]
    fn random_index_from_never_accepts_the_reject_zone() {
        // Feeding only a rejected value must exhaust the source (panic),
        // proving it is never accepted.
        let mut src = words_from(&[65530]);
        let _ = random_index_from(104, &mut src);
    }

    #[test]
    fn random_index_from_is_provably_unbiased() {
        // Enumerating all 65536 values hits each of the 104 indices exactly
        // 630 times, with the top 16 values (65520..65536) rejected.
        let mut counts = [0u32; 104];
        let mut rejected = 0;
        for v in 0u32..65536 {
            if v >= 65520 {
                let v16 = v as u16;
                let mut src = words_from(std::slice::from_ref(&v16));
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    random_index_from(104, &mut src)
                }));
                assert!(result.is_err(), "value {v} should be in the reject zone");
                rejected += 1;
                continue;
            }
            let v16 = v as u16;
            let mut src = words_from(std::slice::from_ref(&v16));
            counts[random_index_from(104, &mut src).unwrap() as usize] += 1;
        }
        assert_eq!(rejected, 16); // 65536 - 65520
        for c in counts {
            assert_eq!(c, 630); // 65520 / 104
        }
    }

    /// Integration-level no-bias check: a chi-square goodness-of-fit test on
    /// a large sample of real CSPRNG draws. The exact, non-flaky proof of
    /// uniformity lives in `random_index_from_is_provably_unbiased` (the
    /// 65536-value enumeration); this complements it end-to-end.
    ///
    /// df = 103. Critical value at alpha = 0.001 is ~149.4, chosen loose so a
    /// correct implementation effectively never flakes while a biased one
    /// fails reliably.
    #[test]
    fn random_index_distribution_passes_chi_square() {
        const N: u32 = 104;
        const SAMPLES: u32 = N * 5000; // ~520k draws
        let mut counts = [0u32; N as usize];
        for _ in 0..SAMPLES {
            counts[random_index(N).unwrap() as usize] += 1;
        }

        let expected = SAMPLES as f64 / N as f64;
        let chi_square: f64 = counts
            .iter()
            .map(|&observed| {
                let diff = observed as f64 - expected;
                (diff * diff) / expected
            })
            .sum();

        assert!(chi_square < 149.4);
    }
}
