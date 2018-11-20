//! High-level traits to translate the low-level API to idiomatic Rust.

use atof::*;
use atoi::*;
use error::Error;
use ftoa::*;
use itoa::*;
use lib;

// FROM BYTES

/// Trait for types that are deserializable from strings or bytes.
pub trait FromBytes: Sized {
    /// Deserialize from byte slice.
    fn from_bytes(bytes: &[u8], base: u8) -> Self;

    /// Error-checking deserialize from byte slice.
    fn try_from_bytes(bytes: &[u8], base: u8) -> Result<Self, Error>;
}

macro_rules! from_bytes {
    ($t:ty, $bytes_cb:ident, $try_bytes_cb:ident) => (
        impl FromBytes for $t {
            #[inline(always)]
            fn from_bytes(bytes: &[u8], base: u8) -> $t
            {
                // We reverse the argument order, since the low-level API
                // always uses (base: u8, first: *const u8, last: *const u8)
                $bytes_cb(base, bytes)
            }

            #[inline(always)]
            fn try_from_bytes(bytes: &[u8], base: u8) -> Result<$t, Error>
            {
                // We reverse the argument order, since the low-level API
                // always uses (base: u8, first: *const u8, last: *const u8)
                $try_bytes_cb(base, bytes)
            }
        }
    )
}

from_bytes!(u8, atou8_bytes, try_atou8_bytes);
from_bytes!(u16, atou16_bytes, try_atou16_bytes);
from_bytes!(u32, atou32_bytes, try_atou32_bytes);
from_bytes!(u64, atou64_bytes, try_atou64_bytes);
from_bytes!(usize, atousize_bytes, try_atousize_bytes);
from_bytes!(i8, atoi8_bytes, try_atoi8_bytes);
from_bytes!(i16, atoi16_bytes, try_atoi16_bytes);
from_bytes!(i32, atoi32_bytes, try_atoi32_bytes);
from_bytes!(i64, atoi64_bytes, try_atoi64_bytes);
from_bytes!(isize, atoisize_bytes, try_atoisize_bytes);
from_bytes!(f32, atof32_bytes, try_atof32_bytes);
from_bytes!(f64, atof64_bytes, try_atof64_bytes);

// FROM BYTES LOSSY

pub trait FromBytesLossy: FromBytes {
    /// Deserialize from byte slice.
    fn from_bytes_lossy(bytes: &[u8], base: u8) -> Self;

    /// Error-checking deserialize from byte slice.
    fn try_from_bytes_lossy(bytes: &[u8], base: u8) -> Result<Self, Error>;
}

macro_rules! from_bytes_lossy {
    ($t:ty, $bytes_cb:ident, $try_bytes_cb:ident) => (
        impl FromBytesLossy for $t {
            #[inline(always)]
            fn from_bytes_lossy(bytes: &[u8], base: u8) -> $t
            {
                // We reverse the argument order, since the low-level API
                // always uses (base: u8, first: *const u8, last: *const u8)
                $bytes_cb(base, bytes)
            }

            #[inline(always)]
            fn try_from_bytes_lossy(bytes: &[u8], base: u8) -> Result<$t, Error>
            {
                // We reverse the argument order, since the low-level API
                // always uses (base: u8, first: *const u8, last: *const u8)
                $try_bytes_cb(base, bytes)
            }
        }
    )
}

from_bytes_lossy!(f32, atof32_lossy_bytes, try_atof32_lossy_bytes);
from_bytes_lossy!(f64, atof64_lossy_bytes, try_atof64_lossy_bytes);

// TO BYTES

/// Trait for types that are serializable to string or bytes.
pub trait ToBytes: Sized {
    /// Serialize to string.
    fn to_bytes(&self, base: u8) -> lib::Vec<u8>;
}

macro_rules! to_bytes {
    ($t:ty, $string_cb:ident) => (
        impl ToBytes for $t {
            #[inline(always)]
            fn to_bytes(&self, base: u8) -> lib::Vec<u8>
            {
                $string_cb(*self, base)
            }
        }
    )
}

to_bytes!(u8, u8toa_bytes);
to_bytes!(u16, u16toa_bytes);
to_bytes!(u32, u32toa_bytes);
to_bytes!(u64, u64toa_bytes);
to_bytes!(usize, usizetoa_bytes);
to_bytes!(i8, i8toa_bytes);
to_bytes!(i16, i16toa_bytes);
to_bytes!(i32, i32toa_bytes);
to_bytes!(i64, i64toa_bytes);
to_bytes!(isize, isizetoa_bytes);
to_bytes!(f32, f32toa_bytes);
to_bytes!(f64, f64toa_bytes);

// TESTS
// -----

#[cfg(test)]
mod tests {
    use error::invalid_digit;
    use super::*;

    macro_rules! deserialize_int {
        ($($t:tt)*) => ($({
            assert_eq!($t::from_bytes(b"0", 10), 0);
            assert_eq!($t::try_from_bytes(b"0", 10), Ok(0));
            assert_eq!($t::try_from_bytes(b"", 10), Err(invalid_digit(0)));
            assert_eq!($t::try_from_bytes(b"1a", 10), Err(invalid_digit(1)));
        })*)
    }

    macro_rules! deserialize_float {
        ($($t:tt)*) => ($({
            assert_eq!($t::from_bytes(b"0.0", 10), 0.0);
            assert_eq!($t::from_bytes_lossy(b"0.0", 10), 0.0);
            assert_eq!($t::try_from_bytes(b"0.0", 10), Ok(0.0));
            assert_eq!($t::try_from_bytes(b"0.0a", 10), Err(invalid_digit(3)));
            assert_eq!($t::try_from_bytes_lossy(b"0.0", 10), Ok(0.0));
        })*)
    }

    #[test]
    fn from_bytes_test() {
        deserialize_int! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }
        deserialize_float! { f32 f64 }
    }

    macro_rules! serialize_int {
        ($($t:tt)*) => ($({
            let x: $t = 0;
            assert_eq!(x.to_bytes(10), b"0".to_vec());
        })*)
    }

    macro_rules! serialize_float {
        ($($t:tt)*) => ($({
            let x: $t = 0.0;
            assert_eq!(x.to_bytes(10), b"0.0".to_vec());
        })*)
    }

    #[test]
    fn to_bytes_test() {
        serialize_int! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }
        serialize_float! { f32 f64 }
    }
}
