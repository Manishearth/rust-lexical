//! Defines rounding schemes for floating-point numbers.

use util::*;
use super::float::ExtendedFloat;
use super::mantissa::Mantissa;
use super::shift::{shl, shr};

// GENERIC
// -------

/// Parameters for general rounding operations.
#[derive(Debug)]
pub struct RoundingParameters<M: Mantissa> {
    /// Bits to truncate from the mantissa.
    pub mask: M,
    /// Midway point for truncated bits.
    pub mid: M,
    /// Number of bits to shift
    pub shift: i32,
}

// ROUND NEAREST TIE EVEN

/// Shift right N-bytes and round to the nearest.
///
/// Return if we are above halfway and if we are halfway.
#[inline]
pub(super) fn round_nearest<M>(fp: &mut ExtendedFloat<M>, params: &RoundingParameters<M>)
    -> (bool, bool)
    where M: Mantissa
{
    // Extract the truncated bits using mask.
    // Calculate if the value of the truncated bits are either above
    // the mid-way point, or equal to it.
    //
    // For example, for 4 truncated bytes, the mask would be b1111
    // and the midway point would be b1000.
    let truncated_bits = fp.frac & params.mask;
    let is_above = truncated_bits > params.mid;
    let is_halfway = truncated_bits == params.mid;

    // Bit shift so the leading bit is in the hidden bit.
    shr(fp, params.shift);

    (is_above, is_halfway)
}

/// Shift right N-bytes and round nearest, tie-to-even.
///
/// Floating-point arithmetic uses round to nearest, ties to even,
/// which rounds to the nearest value, if the value is halfway in between,
/// round to an even value.
#[inline]
pub(super) fn round_nearest_tie_even<M>(fp: &mut ExtendedFloat<M>, params: &RoundingParameters<M>)
    where M: Mantissa
{
    let (is_above, is_halfway) = round_nearest(fp, params);

    // Extract the last bit after shifting (and determine if it is odd).
    let is_odd = fp.frac & M::ONE == M::ONE;

    // Calculate if we need to roundup.
    // We need to roundup if we are above halfway, or if we are odd
    // and at half-way (need to tie-to-even).
    let is_roundup = is_above || (is_odd && is_halfway);

    // Roundup as needed.
    fp.frac += as_::<M, _>(is_roundup as u32);
}

/// Shift right N-bytes and round nearest, tie-to-even.
///
/// Floating-point arithmetic uses round to nearest, ties to even,
/// which rounds to the nearest value, if the value is halfway in between,
/// round to an even value.
#[inline]
#[allow(dead_code)]
pub(super) fn round_nearest_tie_away_zero<M>(fp: &mut ExtendedFloat<M>, params: &RoundingParameters<M>)
    where M: Mantissa
{
    let (is_above, is_halfway) = round_nearest(fp, params);

    // Calculate if we need to roundup.
    // We need to roundup if we are halfway or above halfway,
    // since the value is always positive and we need to round away
    // from zero.
    let is_roundup = is_above || is_halfway;

    // Roundup as needed.
    fp.frac += as_::<M, _>(is_roundup as u32);
}

// NATIVE FLOAT
// ------------

// FLOAT ROUNDING

/// Trait to round extended-precision floats to native representations.
pub trait FloatRounding<M: Mantissa>: Float {
    /// Default number of bits to shift (or 64 - mantissa size - 1).
    const DEFAULT_SHIFT: i32;
    /// Mask to determine if a full-carry occurred (1 in bit above hidden bit).
    const CARRY_MASK: M;
    /// Mask from the hidden bit to the right, to see if we can prevent overflow.]
    const OVERFLOW_MASK: &'static [M];
    /// Rounding parameters to convert to native float.
    const ROUNDING_PARAMS: &'static RoundingParameters<M> = &M::ROUNDING_PARAMETERS[Self::DEFAULT_SHIFT as usize];
}

// Literals don't work for generic types, we need to use this as a hack.
macro_rules! float_rounding_f32 {
    ($($t:tt)*) => ($(
        impl FloatRounding<$t> for f32 {
            const DEFAULT_SHIFT: i32    = $t::BITS - f32::MANTISSA_SIZE - 1;
            const CARRY_MASK: $t        = 0x1000000;
            const OVERFLOW_MASK: &'static [$t] = &[
                0x00800000, 0x00C00000, 0x00E00000, 0x00F00000, 0x00F80000, 0x00FC0000,
                0x00FE0000, 0x00FF0000, 0x00FF8000, 0x00FFC000, 0x00FFE000, 0x00FFF000,
                0x00FFF800, 0x00FFFC00, 0x00FFFE00, 0x00FFFF00, 0x00FFFF80, 0x00FFFFC0,
                0x00FFFFE0, 0x00FFFFF0, 0x00FFFFF8, 0x00FFFFFC, 0x00FFFFFE, 0x00FFFFFF
            ];
        }
    )*)
}

float_rounding_f32! { u64 u128 }

// Literals don't work for generic types, we need to use this as a hack.
macro_rules! float_rounding_f64 {
    ($($t:tt)*) => ($(
        impl FloatRounding<$t> for f64 {
            const DEFAULT_SHIFT: i32    = $t::BITS - f64::MANTISSA_SIZE - 1;
            const CARRY_MASK: $t        = 0x20000000000000;
            const OVERFLOW_MASK: &'static [$t] = &[
                0x0010000000000000, 0x0018000000000000, 0x001C000000000000,
                0x001E000000000000, 0x001F000000000000, 0x001F800000000000,
                0x001FC00000000000, 0x001FE00000000000, 0x001FF00000000000,
                0x001FF80000000000, 0x001FFC0000000000, 0x001FFE0000000000,
                0x001FFF0000000000, 0x001FFF8000000000, 0x001FFFC000000000,
                0x001FFFE000000000, 0x001FFFF000000000, 0x001FFFF800000000,
                0x001FFFFC00000000, 0x001FFFFE00000000, 0x001FFFFF00000000,
                0x001FFFFF80000000, 0x001FFFFFC0000000, 0x001FFFFFE0000000,
                0x001FFFFFF0000000, 0x001FFFFFF8000000, 0x001FFFFFFC000000,
                0x001FFFFFFE000000, 0x001FFFFFFF000000, 0x001FFFFFFF800000,
                0x001FFFFFFFC00000, 0x001FFFFFFFE00000, 0x001FFFFFFFF00000,
                0x001FFFFFFFF80000, 0x001FFFFFFFFC0000, 0x001FFFFFFFFE0000,
                0x001FFFFFFFFF0000, 0x001FFFFFFFFF8000, 0x001FFFFFFFFFC000,
                0x001FFFFFFFFFE000, 0x001FFFFFFFFFF000, 0x001FFFFFFFFFF800,
                0x001FFFFFFFFFFC00, 0x001FFFFFFFFFFE00, 0x001FFFFFFFFFFF00,
                0x001FFFFFFFFFFF80, 0x001FFFFFFFFFFFC0, 0x001FFFFFFFFFFFE0,
                0x001FFFFFFFFFFFF0, 0x001FFFFFFFFFFFF8, 0x001FFFFFFFFFFFFC,
                0x001FFFFFFFFFFFFE, 0x001FFFFFFFFFFFFF
            ];
        }
    )*)
}

float_rounding_f64! { u64 u128 }

// ROUND TO FLOAT

/// Shift the ExtendedFloat fraction to the fraction bits in a native float.
///
/// Floating-point arithmetic uses round to nearest, ties to even,
/// which rounds to the nearest value, if the value is halfway in between,
/// round to an even value.
#[inline]
pub(super) fn round_to_float<T, M>(fp: &mut ExtendedFloat<M>)
    where T: FloatRounding<M>,
          M: Mantissa
{
    // Calculate the difference to allow a single calculation
    // rather than a loop, to minimize the number of ops required.
    // This does underflow detection.
    let final_exp = fp.exp + T::DEFAULT_SHIFT;
    if final_exp < T::DENORMAL_EXPONENT {
        // We would end up with a denormal exponent, try to round to more
        // digits. Only shift right if we can avoid zeroing out the value,
        // which requires the exponent diff to be < M::BITS. The value
        // is already normalized, so we shouldn't have any issue zeroing
        // out the value.
        let diff = T::DENORMAL_EXPONENT - fp.exp;
        if diff < M::BITS {
            let params = unsafe { M::ROUNDING_PARAMETERS.get_unchecked(diff as usize) };
            round_nearest_tie_even(fp, params);
        } else {
            // Certain underflow, assign literal 0s.
            fp.frac = M::ZERO;
            fp.exp = 0;
        }
    } else {
        round_nearest_tie_even(fp, T::ROUNDING_PARAMS);
    }

    if fp.frac & T::CARRY_MASK == T::CARRY_MASK {
        // Roundup carried over to 1 past the hidden bit.
        shr(fp, 1);
    }
}

// AVOID OVERFLOW/UNDERFLOW

/// Avoid overflow for large values, shift left as needed.
///
/// Shift until a 1-bit is in the hidden bit, if the mantissa is not 0.
#[inline]
pub(super) fn avoid_overflow<T, M>(fp: &mut ExtendedFloat<M>)
    where T: FloatRounding<M>,
          M: Mantissa
{
    // Calculate the difference to allow a single calculation
    // rather than a loop, using a precalculated bitmask table,
    // minimizing the number of ops required.
    if fp.exp >= T::MAX_EXPONENT {
        let diff = fp.exp - T::MAX_EXPONENT;
        let idx = diff as usize;
        if let Some(mask) = T::OVERFLOW_MASK.get(idx) {
            if (fp.frac & *mask).is_zero() {
                // If we have no 1-bit in the hidden-bit position,
                // which is index 0, we need to shift 1.
                let shift = diff + 1;
                shl(fp, shift);
            }
        }
    }
}

// ROUND TO NATIVE

/// Round an extended-precision float to a native float representation.
#[inline]
pub(super) fn round_to_native<T, M>(fp: &mut ExtendedFloat<M>)
    where T: FloatRounding<M>,
          M: Mantissa
{
    // Shift all the way left, to ensure a consistent representation.
    // The following right-shifts do not work for a non-normalized number.
    fp.normalize();

    // Round so the fraction is in a native mantissa representation,
    // and avoid overflow/underflow.
    round_to_float::<T, M>(fp);
    avoid_overflow::<T, M>(fp)
}


// TESTS
// -----

#[cfg(test)]
mod tests {
    use float::ExtendedFloat80;
    use super::*;

    #[test]
    fn round_nearest_test() {
        let round = &u64::ROUNDING_PARAMETERS[6];

        // Check exactly halfway (b'1100000')
        let mut fp = ExtendedFloat80 { frac: 0x60, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, round);
        assert!(!above);
        assert!(halfway);
        assert_eq!(fp.frac, 1);

        // Check above halfway (b'1100001')
        let mut fp = ExtendedFloat80 { frac: 0x61, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, round);
        assert!(above);
        assert!(!halfway);
        assert_eq!(fp.frac, 1);

        // Check below halfway (b'1011111')
        let mut fp = ExtendedFloat80 { frac: 0x5F, exp: 0 };
        let (above, halfway) = round_nearest(&mut fp, round);
        assert!(!above);
        assert!(!halfway);
        assert_eq!(fp.frac, 1);
    }

    #[test]
    fn round_nearest_tie_even_test() {
        let round = &u64::ROUNDING_PARAMETERS[6];

        // Check round-up, halfway
        let mut fp = ExtendedFloat80 { frac: 0x60, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 2);

        // Check round-down, halfway
        let mut fp = ExtendedFloat80 { frac: 0x20, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 0);

        // Check round-up, above halfway
        let mut fp = ExtendedFloat80 { frac: 0x61, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 2);

        let mut fp = ExtendedFloat80 { frac: 0x21, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 1);

        // Check round-down, below halfway
        let mut fp = ExtendedFloat80 { frac: 0x5F, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 1);

        let mut fp = ExtendedFloat80 { frac: 0x1F, exp: 0 };
        round_nearest_tie_even(&mut fp, round);
        assert_eq!(fp.frac, 0);
    }

    #[test]
    fn round_nearest_tie_away_zero_test() {
        let round = &u64::ROUNDING_PARAMETERS[6];

        // Check round-up, halfway
        let mut fp = ExtendedFloat80 { frac: 0x60, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 2);

        let mut fp = ExtendedFloat80 { frac: 0x20, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 1);

        // Check round-up, above halfway
        let mut fp = ExtendedFloat80 { frac: 0x61, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 2);

        let mut fp = ExtendedFloat80 { frac: 0x21, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 1);

        // Check round-down, below halfway
        let mut fp = ExtendedFloat80 { frac: 0x5F, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 1);

        let mut fp = ExtendedFloat80 { frac: 0x1F, exp: 0 };
        round_nearest_tie_away_zero(&mut fp, round);
        assert_eq!(fp.frac, 0);
    }

    #[test]
    fn round_to_float_test() {
        // Denormal
        let mut fp = ExtendedFloat80 { frac: 1<<63, exp: f64::DENORMAL_EXPONENT - 15 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<48);
        assert_eq!(fp.exp, f64::DENORMAL_EXPONENT);

        // Halfway, round-down (b'1000000000000000000000000000000000000000000000000000010000000000')
        let mut fp = ExtendedFloat80 { frac: 0x8000000000000400, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<52);
        assert_eq!(fp.exp, -52);

        // Halfway, round-up (b'1000000000000000000000000000000000000000000000000000110000000000')
        let mut fp = ExtendedFloat80 { frac: 0x8000000000000C00, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Above halfway
        let mut fp = ExtendedFloat80 { frac: 0x8000000000000401, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52)+1);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat80 { frac: 0x8000000000000C01, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Below halfway
        let mut fp = ExtendedFloat80 { frac: 0x80000000000003FF, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<52);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat80 { frac: 0x8000000000000BFF, exp: -63 };
        round_to_float::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 1);
        assert_eq!(fp.exp, -52);
    }

    #[test]
    fn avoid_overflow_test() {
        // Avoid overflow, fails by 1
        let mut fp = ExtendedFloat80 { frac: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 5 };
        avoid_overflow::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 0xFFFFFFFFFFFF);
        assert_eq!(fp.exp, f64::MAX_EXPONENT+5);

        // Avoid overflow, succeeds
        let mut fp = ExtendedFloat80 { frac: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 4 };
        avoid_overflow::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 0x1FFFFFFFFFFFE0);
        assert_eq!(fp.exp, f64::MAX_EXPONENT-1);
    }

    #[test]
    fn round_to_native_test() {
        // Overflow
        let mut fp = ExtendedFloat80 { frac: 0xFFFFFFFFFFFF, exp: f64::MAX_EXPONENT + 4 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 0x1FFFFFFFFFFFE0);
        assert_eq!(fp.exp, f64::MAX_EXPONENT-1);

        // Need denormal
        let mut fp = ExtendedFloat80 { frac: 1, exp: f64::DENORMAL_EXPONENT +48 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<48);
        assert_eq!(fp.exp, f64::DENORMAL_EXPONENT);

        // Halfway, round-down (b'10000000000000000000000000000000000000000000000000000100000')
        let mut fp = ExtendedFloat80 { frac: 0x400000000000020, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<52);
        assert_eq!(fp.exp, -52);

        // Halfway, round-up (b'10000000000000000000000000000000000000000000000000001100000')
        let mut fp = ExtendedFloat80 { frac: 0x400000000000060, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Above halfway
        let mut fp = ExtendedFloat80 { frac: 0x400000000000021, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52)+1);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat80 { frac: 0x400000000000061, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 2);
        assert_eq!(fp.exp, -52);

        // Below halfway
        let mut fp = ExtendedFloat80 { frac: 0x40000000000001F, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, 1<<52);
        assert_eq!(fp.exp, -52);

        let mut fp = ExtendedFloat80 { frac: 0x40000000000005F, exp: -58 };
        round_to_native::<f64, _>(&mut fp);
        assert_eq!(fp.frac, (1<<52) + 1);
        assert_eq!(fp.exp, -52);
    }
}
