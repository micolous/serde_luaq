mod de;

use std::{fmt::Display, ops::Neg};

/// Maximum integer value that can be represented in an [`f64`] without loss of precision.
pub const MAX_F64_INTEGER: i64 = (1_i64 << f64::MANTISSA_DIGITS) - 1;

/// Minimum integer value that can be represented in an [`f64`] without loss of precision.
pub const MIN_F64_INTEGER: i64 = -((1_i64 << f64::MANTISSA_DIGITS) - 1);

/// Lua number types.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LuaNumber {
    /// *number* type, *integer* subtype.
    ///
    /// Lua uses 64-bit values by default, but can be configured to use 32-bit. `serde_luaq` only
    /// uses 64-bit values.
    Integer(i64),

    /// *number* type, *float* subtype.
    ///
    /// Lua uses 64-bit values by default, but can be configured to use 32-bit. `serde_luaq` only
    /// uses 64-bit values.
    Float(f64),
}

impl LuaNumber {
    /// Returns `true` if the value is NaN.
    #[inline]
    pub const fn is_nan(&self) -> bool {
        matches!(self, LuaNumber::Float(f) if f.is_nan())
    }

    /// Returns `true` if the value is neither infinite nor NaN.
    #[inline]
    pub const fn is_finite(&self) -> bool {
        match self {
            LuaNumber::Float(f) => f.is_finite(),
            LuaNumber::Integer(_) => true,
        }
    }

    /// Returns `true` if the value is positive or negative infinity.
    #[inline]
    pub const fn is_infinite(&self) -> bool {
        match self {
            LuaNumber::Float(f) => f.is_infinite(),
            LuaNumber::Integer(_) => false,
        }
    }

    /// Returns `true` if the number is represented using the `integer` subtype.
    pub const fn is_i64(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Returns `true` if the number is represented using the `float` subtype.
    pub const fn is_f64(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Attempt to convert a `LuaNumber::Integer` into an `i64`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaNumber;
    ///
    /// let a = LuaNumber::Integer(123);
    /// assert_eq!(a.as_i64().unwrap(), 123);
    ///
    /// let b = LuaNumber::Integer(i64::MIN);
    /// assert_eq!(b.as_i64().unwrap(), i64::MIN);
    ///
    /// let c = LuaNumber::Float(3.);
    /// assert!(c.as_i64().is_none());
    /// ```
    pub const fn as_i64(self) -> Option<i64> {
        match self {
            LuaNumber::Float(_) => None,
            LuaNumber::Integer(v) => Some(v),
        }
    }

    /// Attempt to convert a [`LuaNumber`] into an `f64`.
    ///
    /// This will convert integer values to floating point if they can be represented without a loss
    /// of precision `[-(2**53)+1, (2**53)-1]`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaNumber;
    ///
    /// let a = LuaNumber::Float(3.14);
    /// assert_eq!(a.as_f64().unwrap(), 3.14);
    ///
    /// let b = LuaNumber::Integer(123);
    /// assert_eq!(b.as_f64().unwrap(), 123.);
    ///
    /// let c = LuaNumber::Integer(i64::MAX);
    /// assert!(c.as_f64().is_none());
    /// ```
    pub const fn as_f64(self) -> Option<f64> {
        match self {
            LuaNumber::Float(v) => Some(v),
            LuaNumber::Integer(v) => {
                if v <= MAX_F64_INTEGER && v >= MIN_F64_INTEGER {
                    Some(v as f64)
                } else {
                    None
                }
            }
        }
    }
}

impl From<f64> for LuaNumber {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<f32> for LuaNumber {
    fn from(value: f32) -> Self {
        Self::Float(value.into())
    }
}

impl From<i64> for LuaNumber {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

macro_rules! lua_number_from_int {
    ($($ty:ty)*) => {$(
        impl From<$ty> for LuaNumber {
            fn from(value: $ty) -> Self {
                Self::Integer(value.into())
            }
        }
    )*};
}

lua_number_from_int! { i32 i16 i8 u32 u16 u8 }

macro_rules! lua_number_tryfrom_number {
    ($($ty:ty)*) => {$(
        impl TryFrom<$ty> for LuaNumber {
            type Error = <i64 as TryFrom<$ty>>::Error;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                i64::try_from(value).map(Self::Integer)
            }
        }

    )*};
}

lua_number_tryfrom_number! { u64 isize usize i128 u128 }

impl Neg for LuaNumber {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Float(v) => Self::Float(-v),
            Self::Integer(v) => Self::Integer(-v),
        }
    }
}

impl Display for LuaNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(v) => v.fmt(f),
            Self::Integer(v) => v.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn from_integer() {
        // i64
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0));
        assert_eq!(LuaNumber::Integer(i64::MIN), LuaNumber::from(i64::MIN));
        assert_eq!(LuaNumber::Integer(i64::MAX), LuaNumber::from(i64::MAX));

        // i32
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0i32));
        assert_eq!(
            LuaNumber::Integer(i32::MIN.into()),
            LuaNumber::from(i32::MIN)
        );
        assert_eq!(
            LuaNumber::Integer(i32::MAX.into()),
            LuaNumber::from(i32::MAX)
        );

        // i16
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0i16));
        assert_eq!(
            LuaNumber::Integer(i16::MIN.into()),
            LuaNumber::from(i16::MIN)
        );
        assert_eq!(
            LuaNumber::Integer(i16::MAX.into()),
            LuaNumber::from(i16::MAX)
        );

        // i8
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0i8));
        assert_eq!(LuaNumber::Integer(i8::MIN.into()), LuaNumber::from(i8::MIN));
        assert_eq!(LuaNumber::Integer(i8::MAX.into()), LuaNumber::from(i8::MAX));

        // u64
        assert_eq!(LuaNumber::Integer(0), LuaNumber::try_from(0u64).unwrap());
        LuaNumber::try_from(u64::MAX).unwrap_err();

        // u32
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0u32));
        assert_eq!(
            LuaNumber::Integer(u32::MAX.into()),
            LuaNumber::from(u32::MAX)
        );

        // u16
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0u16));
        assert_eq!(
            LuaNumber::Integer(u16::MAX.into()),
            LuaNumber::from(u16::MAX)
        );

        // u8
        assert_eq!(LuaNumber::Integer(0), LuaNumber::from(0u8));
        assert_eq!(LuaNumber::Integer(u8::MAX.into()), LuaNumber::from(u8::MAX));
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn to_integer() {
        assert_eq!(0_i64, LuaNumber::Integer(0).as_i64().unwrap());
        assert_eq!(i64::MIN, LuaNumber::Integer(i64::MIN).as_i64().unwrap());
        assert_eq!(i64::MAX, LuaNumber::Integer(i64::MAX).as_i64().unwrap());

        // Floats should always error, even if a whole number
        assert!(LuaNumber::Float(0.).as_i64().is_none());
        assert!(LuaNumber::Float(0.5).as_i64().is_none());
        assert!(LuaNumber::Float(i32::MAX as f64).as_i64().is_none());
        assert!(LuaNumber::Float(f64::MAX).as_i64().is_none());
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn from_float() {
        // f64
        assert_eq!(LuaNumber::Float(0.), LuaNumber::from(0.));
        assert_eq!(LuaNumber::Float(-0.), LuaNumber::from(-0.));
        assert_eq!(LuaNumber::Float(f64::MIN), LuaNumber::from(f64::MIN));
        assert_eq!(LuaNumber::Float(f64::MAX), LuaNumber::from(f64::MAX));
        assert_eq!(
            LuaNumber::Float(f64::INFINITY),
            LuaNumber::from(f64::INFINITY),
        );
        assert_eq!(
            LuaNumber::Float(f64::NEG_INFINITY),
            LuaNumber::from(f64::NEG_INFINITY),
        );

        let f = LuaNumber::from(f64::NAN);
        assert!(matches!(f, LuaNumber::Float(x) if x.is_nan()));

        // f32
        assert_eq!(LuaNumber::Float(0.), LuaNumber::from(0f32));
        assert_eq!(LuaNumber::Float(f32::MIN.into()), LuaNumber::from(f32::MIN));
        assert_eq!(LuaNumber::Float(f32::MAX.into()), LuaNumber::from(f32::MAX));
        assert_eq!(
            LuaNumber::Float(f64::INFINITY),
            LuaNumber::from(f32::INFINITY),
        );
        assert_eq!(
            LuaNumber::Float(f64::NEG_INFINITY),
            LuaNumber::from(f32::NEG_INFINITY),
        );

        let f = LuaNumber::from(f32::NAN);
        assert!(matches!(f, LuaNumber::Float(x) if x.is_nan()));
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn to_float() {
        assert_eq!(0_f64, LuaNumber::Float(0.).as_f64().unwrap());
        assert_eq!(-0_f64, LuaNumber::Float(-0.).as_f64().unwrap());
        assert_eq!(f64::MIN, LuaNumber::Float(f64::MIN).as_f64().unwrap());
        assert_eq!(f64::MAX, LuaNumber::Float(f64::MAX).as_f64().unwrap());

        assert_eq!(
            f64::INFINITY,
            LuaNumber::Float(f64::INFINITY).as_f64().unwrap(),
        );
        assert_eq!(
            f64::NEG_INFINITY,
            LuaNumber::Float(f64::NEG_INFINITY).as_f64().unwrap(),
        );
        assert!(LuaNumber::Float(f64::NAN).as_f64().unwrap().is_nan());

        // Integer to float conversion should sometimes work
        assert_eq!(0_f64, LuaNumber::Integer(0).as_f64().unwrap());
        assert_eq!(
            -2147483648_f64,
            LuaNumber::Integer(i32::MIN.into()).as_f64().unwrap(),
        );
        assert_eq!(
            2147483647_f64,
            LuaNumber::Integer(i32::MAX.into()).as_f64().unwrap(),
        );

        // Bounds of safe conversion
        assert_eq!(
            9007199254740990.0,
            LuaNumber::Integer(2_i64.pow(53) - 2).as_f64().unwrap(),
        );

        assert_eq!(
            9007199254740991.0,
            LuaNumber::Integer(2_i64.pow(53) - 1).as_f64().unwrap(),
        );

        assert_eq!(
            -9007199254740990.0,
            LuaNumber::Integer(-(2_i64.pow(53) - 2)).as_f64().unwrap(),
        );

        assert_eq!(
            -9007199254740991.0,
            LuaNumber::Integer(-(2_i64.pow(53) - 1)).as_f64().unwrap(),
        );

        // Out of bounds
        assert_eq!(None, LuaNumber::Integer(2_i64.pow(53)).as_f64());
        assert_eq!(None, LuaNumber::Integer(-(2_i64.pow(53))).as_f64());
        assert_eq!(None, LuaNumber::Integer(i64::MAX).as_f64());
    }
}
