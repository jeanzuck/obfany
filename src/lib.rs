/*!
Compiletime string constant obfuscation.
*/

#![cfg_attr(not(test), no_std)]

use core::ffi::CStr;
use core::str;

#[doc(hidden)]
pub mod wide;

#[doc(hidden)]
pub mod cfo;

mod murmur3;
pub use self::murmur3::murmur3;

mod pos;
pub use self::pos::position;

#[doc(hidden)]
pub mod xref;

//----------------------------------------------------------------

/// Compiletime random number generator.
///
/// Supported types are `u8`, `u16`, `u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`, `isize`, `bool`, `f32` and `f64`.
///
/// The integer types generate a random value in their respective range.  
/// The float types generate a random value in range of `[1.0, 2.0)`.
///
/// ```
/// const RND: i32 = obfany::random!(u8) as i32;
/// assert!(RND >= 0 && RND <= 255);
/// # const _: f32 = obfany::random!(f32);
/// # const _: f64 = obfany::random!(f64);
/// ```
///
/// The behavior of the macro inside other macros can be surprising:
///
/// ```
/// // When used as top-level input to macros, random works as expected
/// assert_ne!(obfany::random!(u64), obfany::random!(u64));
///
/// // When used inside the definition of a macro, random does not work as expected
/// macro_rules! inside {
/// 	() => {
/// 		assert_eq!(obfany::random!(u64), obfany::random!(u64));
/// 	};
/// }
/// inside!();
///
/// // When provided a unique seed, random works as expected
/// // Note that the seeds must evaluate to a literal!
/// macro_rules! seeded {
/// 	() => {
/// 		assert_ne!(obfany::random!(u64, "lhs"), obfany::random!(u64, "rhs"));
/// 	};
/// }
/// seeded!();
///
/// // Repeated usage in macros, random does not work as expected
/// macro_rules! repeated {
/// 	($($name:ident),*) => {
/// 		$(let $name = obfany::random!(u64, "seed");)*
/// 	};
/// }
/// repeated!(a, b);
/// assert_eq!(a, b);
///
/// // Provide additional unique seeds, random works as expected
/// macro_rules! repeated_seeded {
/// 	($($name:ident),*) => {
/// 		$(let $name = obfany::random!(u64, "seed", stringify!($name));)*
/// 	};
/// }
/// repeated_seeded!(c, d);
/// assert_ne!(c, d);
/// ```
#[macro_export]
macro_rules! random {
	($ty:ident $(, $seeds:expr)* $(,)?) => {{
		const _RANDOM: $ty = $crate::__random_cast!($ty,
			$crate::entropy(concat!(file!(), ":", line!(), ":", column!() $(, ":", $seeds)*)));
		_RANDOM
	}};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __random_cast {
    (u8, $seed:expr) => {
        $seed as u8
    };
    (u16, $seed:expr) => {
        $seed as u16
    };
    (u32, $seed:expr) => {
        $seed as u32
    };
    (u64, $seed:expr) => {
        $seed
    };
    (usize, $seed:expr) => {
        $seed as usize
    };
    (i8, $seed:expr) => {
        $seed as i8
    };
    (i16, $seed:expr) => {
        $seed as i16
    };
    (i32, $seed:expr) => {
        $seed as i32
    };
    (i64, $seed:expr) => {
        $seed as i64
    };
    (isize, $seed:expr) => {
        $seed as isize
    };
    (bool, $seed:expr) => {
        $seed as i64 >= 0
    };

    // f32::from_bits and f64::from_bits are stable as const fn since Rust 1.46
    (f32, $seed:expr) => {
        f32::from_bits(0b0_01111111 << (f32::MANTISSA_DIGITS - 1) | ($seed as u32 >> 9))
    };
    (f64, $seed:expr) => {
        f64::from_bits(0b0_01111111111 << (f64::MANTISSA_DIGITS - 1) | ($seed >> 12))
    };

    ($ty:ident, $seed:expr) => {
        compile_error!(concat!("unsupported type: ", stringify!($ty)))
    };
}

#[test]
fn test_random_f32() {
    fn t(v: f32) {
        assert!(v >= 1.0 && v < 2.0, "{}", v);
    }
    // Multiple calls with different seeds ensure coverage across the float range
    t(random!(f32, "a"));
    t(random!(f32, "b"));
    t(random!(f32, "c"));
    // Unseeded call uses file!(), line!(), column!()
    t(random!(f32));
}

#[test]
fn test_random_f64() {
    fn t(v: f64) {
        assert!(v >= 1.0 && v < 2.0, "{}", v);
    }
    t(random!(f64, "a"));
    t(random!(f64, "b"));
    t(random!(f64, "c"));
    t(random!(f64));
}

#[test]
fn test_random_int_types() {
    // Test that all supported integer types compile and produce in-range values
    let _: u8 = random!(u8, "u8");
    let _: u16 = random!(u16, "u16");
    let _: u32 = random!(u32, "u32");
    let _: u64 = random!(u64, "u64");
    let _: usize = random!(usize, "usize");
    let _: i8 = random!(i8, "i8");
    let _: i16 = random!(i16, "i16");
    let _: i32 = random!(i32, "i32");
    let _: i64 = random!(i64, "i64");
    let _: isize = random!(isize, "isize");
    let _: bool = random!(bool, "bool");
    // Different seeds produce different values (random! uses column!() so same-seed is impossible)
    assert_ne!(random!(u64, "x"), random!(u64, "y"));
}

/// Compiletime bitmixing.
///
/// Takes an intermediate hash that may not be thoroughly mixed and increase its entropy to obtain both better distribution.
/// See [Better Bit Mixing](https://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html) for reference.
#[inline(always)]
pub const fn splitmix(seed: u64) -> u64 {
    let next = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = next;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    return z ^ (z >> 31);
}

#[test]
fn test_splitmix() {
    // Non-zero output for zero input (avalanche)
    assert_ne!(splitmix(0), 0);
    // Deterministic: same input always produces same output
    assert_eq!(splitmix(42), splitmix(42));
    // Different seeds produce different outputs
    assert_ne!(splitmix(1), splitmix(2));
}

/// Compiletime string constant hash.
///
/// Implemented using the [DJB2 hash function](http://www.cse.yorku.ca/~oz/hash.html#djb2) xor variation.
#[inline(always)]
pub const fn hash(s: &str) -> u32 {
    let s = s.as_bytes();
    let mut result = 3581u32;
    let mut i = 0usize;
    while i < s.len() {
        result = result.wrapping_mul(33) ^ s[i] as u32;
        i += 1;
    }
    return result;
}

#[test]
fn test_hash() {
    // Known answer test (matches doctest)
    assert_eq!(hash!("Hello World"), 0x6E4A573D);
    // Empty string
    let _ = hash!("");
    // Unicode
    let _ = hash!("🦀");
    // Determinism
    assert_eq!(hash!("abc"), hash!("abc"));
    // Different inputs produce different hashes (probabilistic)
    assert_ne!(hash!("abc"), hash!("abd"));
}

/// Compiletime string constant hash.
///
/// Helper macro guarantees compiletime evaluation of the string constant hash.
///
/// ```
/// const STRING: &str = "Hello World";
/// assert_eq!(obfany::hash!(STRING), 0x6E4A573D);
/// ```
#[macro_export]
macro_rules! hash {
    ($s:expr) => {{
        const _DJB2_HASH: u32 = $crate::hash($s);
        _DJB2_HASH
    }};
}

/// Produces pseudorandom entropy from the given string.
#[doc(hidden)]
#[inline(always)]
pub const fn entropy(string: &str) -> u64 {
    splitmix(SEED ^ splitmix(hash(string) as u64))
}

/// Compiletime RNG seed.
///
/// This value is derived from the environment variable `OBFANY_SEED` and has a fixed value if absent.
/// If it changes all downstream dependents are recompiled automatically.
pub const SEED: u64 = splitmix(hash(match option_env!("OBFANY_SEED") {
    Some(seed) => seed,
    None => "FIXED",
}) as u64);

//----------------------------------------------------------------

#[doc(hidden)]
pub mod bytes;

#[doc(hidden)]
pub mod words;

#[doc(hidden)]
#[inline(always)]
pub const fn unsafe_as_str(bytes: &[u8]) -> &str {
    // When used correctly by this crate's macros this should be safe
    #[cfg(debug_assertions)]
    return match str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => panic!("invalid str"),
    };
    #[cfg(not(debug_assertions))]
    return unsafe { str::from_utf8_unchecked(bytes) };
}

#[doc(hidden)]
#[inline(always)]
pub const fn unsafe_as_cstr(bytes: &[u8]) -> &CStr {
    // When used correctly by this crate's macros this should be safe
    #[cfg(debug_assertions)]
    return match CStr::from_bytes_with_nul(bytes) {
        Ok(cstr) => cstr,
        Err(_) => panic!("invalid cstr"),
    };
    #[cfg(not(debug_assertions))]
    return unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };
}
