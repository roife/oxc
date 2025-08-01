//! Parsing utilities for converting Javascript numbers to Rust f64.
//! Code copied originally from
//! [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src/numeric_value.rs)
//! but iterated on since.

use std::borrow::Cow;

use cow_utils::CowUtils;
use num_bigint::BigInt;
use num_traits::Num;

use oxc_allocator::Allocator;
use oxc_span::{Atom, format_atom};

use super::kind::Kind;


macro_rules! fallthrough {
    ($scrutinee:expr $(,)?) => {
        match $scrutinee {}
    };
    ($scrutinee:expr, $first_pat:pat => $first_body:expr $(, $label:lifetime $(: $pat:pat => $body:expr)?)* $(,)?) => {
        fallthrough_rec!{ (match $scrutinee {
            $first_pat => $first_body,
            $($($pat => break $label,)?)*
        }), $($label $(: ($body))?),* }
    };
}

macro_rules! fallthrough_rec {
    (($($acc:tt)*),) => {$($acc)*};
    (($($acc:tt)*), $label:lifetime) => {
        $label: {
            $($acc)*
        }
    };
    (($($acc:tt)*), $label:lifetime: ($($body:tt)*) $(,$($follow:tt)*)? ) => {

        fallthrough_rec!{($label: {
                $($acc)*
            }
            $($body)*
        ), $($($follow)*)?}
    };
}

pub fn parse_int(s: &str, kind: Kind, has_sep: bool) -> Result<f64, &'static str> {
    match kind {
        Kind::Decimal => {
            Ok(if has_sep { parse_decimal_with_underscores(s) } else { parse_decimal(s) })
        }
        Kind::Binary => {
            let s = &s[2..];
            Ok(if has_sep { parse_binary_with_underscores(s) } else { parse_binary(s) })
        }
        Kind::Octal => {
            // Octals always begin with `0`. Trim off leading `0`, `0o` or `0O`.
            let second_byte = s.as_bytes()[1];
            let s = if second_byte == b'o' || second_byte == b'O' {
                // SAFETY: We just checked that 2nd byte is ASCII, so slicing off 2 bytes
                // must be in bounds and on a UTF-8 character boundary.
                unsafe { s.get_unchecked(2..) }
            } else {
                &s[1..] // legacy octal
            };
            Ok(if has_sep { parse_octal_with_underscores(s) } else { parse_octal(s) })
        }
        Kind::Hex => {
            let s = &s[2..];
            Ok(if has_sep { parse_hex_with_underscores(s) } else { parse_hex(s) })
        }
        _ => unreachable!(),
    }
}

pub fn parse_float(s: &str, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { s.cow_replace('_', "") } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));
    s.parse::<f64>().map_err(|_| "invalid float")
}

// ==================================== DECIMAL ====================================

/// b'0' is 0x30 and b'9' is 0x39.
///
/// So we can convert from any decimal digit to its value with `b & 15`.
/// This is produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/WMarz15sq>
#[inline]
const fn decimal_byte_to_value(b: u8) -> u8 {
    debug_assert!(b >= b'0' && b <= b'9');
    b & 15
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_decimal(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    /// `u64::MAX + 1` in decimal is 18446744073709551616 (20 chars).
    const MAX_FAST_DECIMAL_LEN: usize = 19;

    debug_assert!(!s.is_empty());
    let len = s.len();
    if s.len() > MAX_FAST_DECIMAL_LEN {
        return parse_decimal_slow(s);
    }

    let mut result = 0_u64;
    let s = s.as_bytes();
    fallthrough! { len,
        19 => result = decimal_byte_to_value(unsafe { *s.get_unchecked(len - 19) }) as u64,
        'eighteen: 18 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 18) }) as u64;
        },
        'seventeen: 17 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 17) }) as u64;
        },
        'sixteen: 16 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 16) }) as u64;
        },
        'fifteen: 15 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 15) }) as u64;
        },
        'fourteen: 14 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 14) }) as u64;
        },
        'thirteen: 13 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 13) }) as u64;
        },
        'twelve: 12 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 12) }) as u64;
        },
        'eleven: 11 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 11) }) as u64;
        },
        'ten: 10 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 10) }) as u64;
        },
        'nine: 9 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 9) }) as u64;
        },
        'eight: 8 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 8) }) as u64;
        },
        'seven: 7 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 7) }) as u64;
        },
        'six: 6 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 6) }) as u64;
        },
        'five: 5 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 5) }) as u64;
        },
        'four: 4 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 4) }) as u64;
        },
        'three: 3 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 3) }) as u64;
        },
        'two: 2 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 2) }) as u64;
        },
        'one: 1 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 1) }) as u64;
        },
        'otherwise: _ => {},
    }

    result as f64
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_decimal_with_underscores(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    /// `u64::MAX + 1` in decimal is 18446744073709551616 (20 chars).
    const MAX_FAST_DECIMAL_LEN: usize = 19;

    debug_assert!(!s.is_empty());
    let len = s.len();
    if s.len() > MAX_FAST_DECIMAL_LEN {
        return parse_decimal_slow(&s.cow_replace('_', ""));
    }

    let mut result = 0_u64;
    let s = s.as_bytes();
    fallthrough! { len,
        19 => result = decimal_byte_to_value(unsafe { *s.get_unchecked(len - 19) }) as u64,
        'eighteen: 18 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 18) }) as u64;
        },
        'seventeen: 17 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 17) }) as u64;
        },
        'sixteen: 16 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 16) }) as u64;
        },
        'fifteen: 15 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 15) }) as u64;
        },
        'fourteen: 14 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 14) }) as u64;
        },
        'thirteen: 13 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 13) }) as u64;
        },
        'twelve: 12 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 12) }) as u64;
        },
        'eleven: 11 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 11) }) as u64;
        },
        'ten: 10 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 10) }) as u64;
        },
        'nine: 9 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 9) }) as u64;
        },
        'eight: 8 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 8) }) as u64;
        },
        'seven: 7 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 7) }) as u64;
        },
        'six: 6 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 6) }) as u64;
        },
        'five: 5 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 5) }) as u64;
        },
        'four: 4 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 4) }) as u64;
        },
        'three: 3 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 3) }) as u64;
        },
        'two: 2 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 2) }) as u64;
        },
        'one: 1 => {
            result *= 10;
            result += decimal_byte_to_value(unsafe { *s.get_unchecked(len - 1) }) as u64;
        },
        'otherwise: _ => {},
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_decimal_slow(s: &str) -> f64 {
    // NB: Cannot use the `mul_add` loop method that `parse_binary_slow` etc use here,
    // as it produces an imprecise result.
    // For the others it's fine, presumably because multiply by a power of 2
    // just increments f64's exponent. But multiplying by 10 is more complex.
    s.parse::<f64>().unwrap()
}

// ==================================== BINARY ====================================

/// b'0' is 0x30 and b'1' is 0x31.
///
/// So we can convert from binary digit to its value with `b & 1`.
/// This is produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/1vvrK78jf>
#[inline]
const fn binary_byte_to_value(b: u8) -> u8 {
    debug_assert!(b == b'0' || b == b'1');
    b & 1
}

/// NOTE: bit shifting here is is safe and much faster than f64::mul_add.
/// It's safe because we're sure this number is an integer - if it wasn't, it
/// would be a [`Kind::Float`] instead. It's fast because shifting usually takes
/// 1 clock cycle on the ALU, while multiplication+addition uses the FPU and is
/// much slower. Additionally, this loop often gets unrolled by rustc since
/// these numbers are usually not long. On x84_64, FMUL has a latency of 4 clock
/// cycles, which doesn't include addition. Some platforms support mul + add in a
/// single instruction, but many others do not.
///
/// Unfortunately, this approach has the chance to overflow for excessively
/// large numbers. In such cases, we fall back to mul_add. Note that right now
/// we consider leading zeros as part of that length. Right now it doesn't seem
/// worth it performance-wise to check and strip them. Further experimentation
/// could be useful.
#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_binary(s: &str) -> f64 {
    /// binary literals longer than this many characters have the chance to
    /// overflow a u64, forcing us to take the slow path.
    const MAX_FAST_BINARY_LEN: usize = 64;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0b") && !s.starts_with("0B"));

    if s.len() > MAX_FAST_BINARY_LEN {
        return parse_binary_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        result <<= 1;
        result |= binary_byte_to_value(b) as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_binary_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        let value = f64::from(binary_byte_to_value(b));
        result = result.mul_add(2.0, value);
    }
    result
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_binary_with_underscores(s: &str) -> f64 {
    /// binary literals longer than this many characters have the chance to
    /// overflow a u64, forcing us to take the slow path.
    const MAX_FAST_BINARY_LEN: usize = 64;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0b") && !s.starts_with("0B"));

    if s.len() > MAX_FAST_BINARY_LEN {
        return parse_binary_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            result <<= 1;
            result |= binary_byte_to_value(b) as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_binary_with_underscores_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let value = f64::from(binary_byte_to_value(b));
            result = result.mul_add(2.0, value);
        }
    }
    result
}

// ==================================== OCTAL ====================================

/// b'0' is 0x30 and b'7' is 0x37.
///
/// So we can convert from any octal digit to its value with `b & 7`.
/// This is produces more compact assembly than `b - b'0'`.
///
/// <https://godbolt.org/z/9rYTsMoMM>
#[inline]
const fn octal_byte_to_value(b: u8) -> u8 {
    debug_assert!(b >= b'0' && b <= b'7');
    b & 7
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_octal(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    const MAX_FAST_OCTAL_LEN: usize = 21;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0o") && !s.starts_with("0O"));
    if s.len() > MAX_FAST_OCTAL_LEN {
        return parse_octal_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        let n = octal_byte_to_value(b);
        result <<= 3;
        result |= n as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_octal_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        let value = f64::from(octal_byte_to_value(b));
        result = result.mul_add(8.0, value);
    }
    result
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_octal_with_underscores(s: &str) -> f64 {
    /// Numeric strings longer than this have the chance to overflow u64.
    const MAX_FAST_OCTAL_LEN: usize = 21;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0o") && !s.starts_with("0O"));
    if s.len() > MAX_FAST_OCTAL_LEN {
        return parse_octal_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let n = octal_byte_to_value(b);
            result <<= 3;
            result |= n as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_octal_with_underscores_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let value = f64::from(octal_byte_to_value(b));
            result = result.mul_add(8.0, value);
        }
    }
    result
}

// ==================================== HEX ====================================

/// - b'0' is 0x30 and b'9' is 0x39.
/// - b'A' is 0x41 and b'F' is 0x46.
/// - b'a' is 0x61 and b'f' is 0x66.
///
/// So we can convert from a digit to its value with `b & 15`,
/// and from `A-F` or `a-f` to its value with `(b & 15) + 9`.
/// We could use `(b & 7) + 9` for `A-F`, but we use `(b & 15) + 9`
/// so that both branches share the same `b & 15` operation.
///
/// This is produces more slightly more assembly than explicitly matching all possibilities,
/// but only because compiler unrolls the loop.
///
/// <https://godbolt.org/z/5fsdv8rGo>
#[inline]
const fn hex_byte_to_value(b: u8) -> u8 {
    debug_assert!((b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'F') || (b >= b'a' && b <= b'f'));
    if b < b'A' {
        b & 15 // 0-9
    } else {
        (b & 15) + 9 // A-F or a-f
    }
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex(s: &str) -> f64 {
    /// Hex strings longer than this have the chance to overflow u64.
    const MAX_FAST_HEX_LEN: usize = 16;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0x"));

    if s.len() > MAX_FAST_HEX_LEN {
        return parse_hex_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        let n = hex_byte_to_value(b);
        result <<= 4;
        result |= n as u64;
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_hex_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        let value = f64::from(hex_byte_to_value(b));
        result = result.mul_add(16.0, value);
    }
    result
}

#[expect(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex_with_underscores(s: &str) -> f64 {
    /// Hex strings longer than this have the chance to overflow u64.
    const MAX_FAST_HEX_LEN: usize = 16;

    debug_assert!(!s.is_empty());
    debug_assert!(!s.starts_with("0x"));

    if s.len() > MAX_FAST_HEX_LEN {
        return parse_hex_with_underscores_slow(s);
    }

    let mut result = 0_u64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let n = hex_byte_to_value(b);
            result <<= 4;
            result |= n as u64;
        }
    }
    result as f64
}

#[cold]
#[inline(never)]
fn parse_hex_with_underscores_slow(s: &str) -> f64 {
    let mut result = 0_f64;
    for &b in s.as_bytes() {
        if b != b'_' {
            let value = f64::from(hex_byte_to_value(b));
            result = result.mul_add(16.0, value);
        }
    }
    result
}

// ==================================== BIGINT ====================================

pub fn parse_big_int<'a>(
    s: &'a str,
    kind: Kind,
    has_sep: bool,
    allocator: &'a Allocator,
) -> Atom<'a> {
    let s = if has_sep { s.cow_replace('_', "") } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    let radix = match kind {
        // Skip parsing with `BigInt` - it's already in decimal form, and underscores are removed
        Kind::Decimal => return Atom::from_cow_in(&s, allocator),
        Kind::Binary => 2,
        Kind::Octal => 8,
        Kind::Hex => 16,
        _ => unreachable!(),
    };

    let s = &s[2..];

    // NOTE: BigInt::from_bytes does a UTF8 check, then uses from_str_radix under the hood.
    // We already have a string, so we can just use that directly.
    // Lexer already checked `s` represents a valid BigInt, so `unwrap` cannot fail.
    let bigint = BigInt::from_str_radix(s, radix).unwrap();
    format_atom!(allocator, "{bigint}")
}

#[cfg(test)]
#[expect(clippy::unreadable_literal, clippy::mixed_case_hex_literals)]
mod test {
    use super::*;

    #[expect(clippy::cast_precision_loss)]
    fn assert_all_ints_eq<I>(test_cases: I, kind: Kind, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, i64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_int(s, kind, has_sep);
            assert_eq!(
                parsed,
                Ok(expected as f64),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
    fn assert_all_floats_eq<I>(test_cases: I, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, f64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_float(s, has_sep);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }

    const _: () = {
        // decimal
        assert!(decimal_byte_to_value(b'0') == 0);
        assert!(decimal_byte_to_value(b'9') == 9);

        // binary
        assert!(binary_byte_to_value(b'0') == 0);
        assert!(binary_byte_to_value(b'1') == 1);

        // octal
        assert!(octal_byte_to_value(b'0') == 0);
        assert!(octal_byte_to_value(b'7') == 7);

        // hex
        assert!(hex_byte_to_value(b'0') == 0);
        assert!(hex_byte_to_value(b'9') == 9);
        assert!(hex_byte_to_value(b'A') == 10);
        assert!(hex_byte_to_value(b'F') == 15);
        assert!(hex_byte_to_value(b'a') == 10);
        assert!(hex_byte_to_value(b'f') == 15);
    };

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn test_int_precision() {
        assert_eq!(
            // 18446744073709551616 = 1 << 64
            parse_int("18446744073709551616", Kind::Decimal, false),
            Ok(18446744073709551616_i128 as f64)
        );
        // This tests for imprecision which results from using `mul_add` loop
        assert_eq!(
            parse_int("12300000000000000000000000", Kind::Decimal, false),
            Ok(12300000000000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0x10000000000000000 = 1 << 64
            parse_int("0x10000000000000000", Kind::Hex, false),
            Ok(0x10000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0o2000000000000000000000 = 1 << 64
            parse_int("0o2000000000000000000000", Kind::Octal, false),
            Ok(0o2000000000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0b10000000000000000000000000000000000000000000000000000000000000000 = 1 << 64
            parse_int(
                "0b10000000000000000000000000000000000000000000000000000000000000000",
                Kind::Binary,
                false
            ),
            Ok(0b10000000000000000000000000000000000000000000000000000000000000000_i128 as f64)
        );
    }

    #[test]
    fn test_large_number_of_leading_zeros() {
        assert_all_ints_eq(
            vec![("000000000000000000000000000000000000000000000000000000001", 1)],
            Kind::Decimal,
            false,
        );
    }

    #[test]
    #[expect(clippy::excessive_precision)]
    fn test_float_precision() {
        let cases = vec![
            ("1.7976931348623157e+308", 1.7976931348623157e+308),
            ("0.000000001", 0.000_000_001),
        ];
        assert_all_floats_eq(cases, false);
    }

    #[test]
    fn test_parse_int_no_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // normal
            ("0", 0),
            ("1", 1),
            ("000000000000", 0),
            ("9007199254740991", 9007199254740991), // max safe integer, 2^53 - 1
        ];
        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
        ];
        let octal = vec![("0o0", 0o0), ("0o1", 0o1), ("0o10", 0o10), ("0o777", 0o777)];
        let hex: Vec<(&str, i64)> = vec![
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xc", 0xc), // :)
            ("0xdeadbeef", 0xdeadbeef),
            ("0xFfEeDdCcBbAa", 0xFfEeDdCcBbAa),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, false);
        assert_all_ints_eq(binary, Kind::Binary, false);
        assert_all_ints_eq(octal, Kind::Octal, false);
        assert_all_ints_eq(hex, Kind::Hex, false);
    }

    #[test]
    fn test_parse_int_with_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0", 0),
            ("1", 1),
            ("1_000_000", 1_000_000),
            ("000000000000", 0),
            ("9_007_199_254_740_991", 9_007_199_254_740_991), // max safe integer, 2^53 - 1
            // still works for illegal tokens
            ("1___000_000", 1_000_000),
            ("1_", 1),
            ("_1", 1),
        ];

        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
            ("0b1_1000_1001_0001_0000", 0b1_1000_1001_0001_0000),
            // still works for illegal tokens
            ("0b1_0000__0000", 0b1_0000_0000),
            ("0b1_", 0b1),
            ("0b_0", 0b0),
        ];

        let octal = vec![
            ("0o0", 0o0),
            ("0o1", 0o1),
            ("0o10", 0o10),
            ("0o777", 0o777),
            ("0o7_7_7", 0o777),
            ("0o77_73_72", 0o77_73_72),
            // still works for illegal tokens
            ("0o1_0000__0000", 0o100_000_000),
            ("0o1_", 0o1),
            ("0o_0", 0o0),
        ];

        let hex: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xFF_AA_11", 0xFFAA11),
            ("0xdead_beef", 0xdead_beef),
            ("0xFf_Ee_Dd_Cc_Bb_Aa", 0xFfEe_DdCc_BbAa),
            ("0xFfEe_DdCc_BbAa", 0xFfEe_DdCc_BbAa),
            // still works for illegal tokens
            ("0x1_0000__0000", 0x100_000_000),
            ("0x1_", 0x1),
            ("0x_0", 0x0),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, true);
        assert_all_ints_eq(binary, Kind::Binary, true);
        assert_all_ints_eq(octal, Kind::Octal, true);
        assert_all_ints_eq(hex, Kind::Hex, true);
    }

    #[test]
    fn test_decimal() {
        let no_sep: Vec<(&'static str, f64)> =
            vec![("0", 0.0), ("1.0", 1.0), ("1.1", 1.1), ("25.125", 25.125)];

        let sep: Vec<(&'static str, f64)> = vec![
            ("1_000.0", 1000.0),
            ("1.5_000", 1.5),
            // works on invalid tokens
            ("_0._5", 0.5),
            ("0._5", 0.5),
            ("0.5_", 0.5),
        ];

        assert_all_floats_eq(no_sep.clone(), false);
        assert_all_floats_eq(sep.clone(), true);
        for (s, expected) in no_sep {
            let parsed = parse_float(s, false);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
        for (s, expected) in sep {
            let parsed = parse_float(s, true);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
}
