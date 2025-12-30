use crate::{LuaNumber, LuaTableEntry};
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "wasm32",
))]
use static_assertions::assert_eq_size;
use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    str::{from_utf8, Utf8Error},
};

/// Basic Lua 5.4 data types that are equivalent to those available in JSON, similar to
/// `serde_json::Value`.
///
/// This structure avoids owning data where possible.
///
/// Reference: <https://www.lua.org/manual/5.4/manual.html#2.1>
///
/// ## Serde
///
/// **Unlike** `serde_json::Value`, [`LuaValue`][] does not implement
/// [`Deserialize`][serde::Deserialize]. This is due to two problems:
///
/// * [`LuaValue`][] tries to avoid owning data, so any fields would need a lifetime of `'de`. This
///   doesn't work with the `derive` macros.
///
/// * Lua tables' [multiple key types][LuaTableEntry] are a hybrid of Serde's map and sequence
///   types, which are converted to one or the other depending on the destination field type. Serde
///   also does not allow us to make a distinction between [a regular key][LuaTableEntry::KeyValue]
///   and [an identifier key][LuaTableEntry::NameValue], so the table might change.
///
/// As a result, you can't use [`LuaValue`][] as a field type or deserialise to it.
///
/// If you want to deserialise Lua to a [`LuaValue`][], use one of
/// [the `peg` deserialisers][crate#peg-deserialiser].
#[derive(Clone, PartialEq)]
pub enum LuaValue<'a> {
    /// Nil value.
    ///
    /// Unlike Lua, `nil` can be used anywhere, including as a table key or value.
    Nil,

    /// Boolean, either `true` or `false`.
    Boolean(bool),

    /// Lua strings are equivalent to a `[u8]`.
    ///
    /// We don't attempt to turn this into a [`str`][], as it is possible for Lua strings to contain
    /// _optionally-escaped_ binary data, and non-UTF-8 encoded strings.
    ///
    /// This also requires the parser to work on `&[u8]`, rather than `&str`.
    ///
    /// [The parser][crate::lua_value] returns a borrowed byte slice for strings that do not contain
    /// escape sequences.
    ///
    /// ## Reference
    ///
    /// [Lua 5.4 Reference Manual, Section 2.1](https://www.lua.org/manual/5.4/manual.html#2.1):
    ///
    /// > The type *string* represents immutable sequences of bytes. Lua is
    /// > 8-bit clean: strings can contain any 8-bit value, including embedded
    /// > zeros (`\0`). Lua is also encoding-agnostic; it makes no assumptions
    /// > about the contents of a string.
    String(Cow<'a, [u8]>),

    /// Number type, which can be an [integer][LuaNumber::Integer] or
    /// [floating point][LuaNumber::Float].
    ///
    /// ## Compatibility
    ///
    /// * **Lua 5.2 and earlier, and Luau** always use `f64` for numbers.
    ///
    ///   **Lua 5.3 and later, and `serde_luaq`** store integers as `i64`.
    ///
    /// * **Lua 5.3** over/underflows decimal integers that didn't fit in a `i64`.
    ///
    ///   **Lua 5.4 and `serde_luaq`** coerce them to `f64`.
    ///
    ///   Hexadecimal integers over/underflow Lua 5.3 and later, and `serde_luaq`.
    ///
    /// * **Luau** adds binary integer literals, which aren't supported by Lua or `serde_luaq`.
    ///
    /// * **Luau** adds optional separators in all types integer literals, which aren't supported by
    ///   Lua or `serde_luaq`.
    Number(LuaNumber),

    /// Array / record / object type.
    ///
    /// ## References
    ///
    /// [Lua 5.4 Reference Manual, Section 2.1][lua2.1]:
    ///
    /// > The type _table_ implements associative arrays, that is, arrays that can be indexed not
    /// > only with numbers, but with any value (except `nil`).
    /// >
    /// > Tables are the sole data structuring mechanism in Lua; they can be used to represent
    /// > ordinary arrays, symbol tables, sets, records, graphs, trees, etc.
    ///
    /// and [Section 3.4.9][lua3.4.9]:
    ///
    /// > Each field of the form `[exp1] = exp2` adds to the new table an entry with key `exp1` and
    /// > value `exp2`. A field of the form `name = exp` is equivalent to `["name"] = exp`. Fields
    /// > of the form `exp` are equivalent to `[i] = exp`, where `i` are consecutive integers
    /// > starting with 1; fields in the other formats do not affect this counting.
    /// >
    /// > The order of the assignments in a constructor is undefined. (This order would be relevant
    /// > only when there are repeated keys.)
    ///
    /// ## Caveats
    ///
    /// `serde_luaq` implements tables slightly differently to Lua:
    ///
    /// * A [`LuaValue::Table`] is a sequence ([`Vec`]) of [entries][LuaTableEntry], rather than a
    ///   `Map`.
    ///
    ///   Entries appear in the order they were in the original file. Their position in the `Vec`
    ///   may not match their key, whether that be [implicit][LuaTableEntry::Value] or explicit.
    ///
    ///   This allows keys to be repeated, and include non-hashable types.
    ///
    /// * Table keys may be set to _any_ value, including `nil` and [`NaN`][f64::NAN].
    ///
    /// * Table values may be set to `nil`.
    ///
    /// When using `serde`, you can still use a table to populate a [`BTreeMap`] or [`Vec`].
    ///
    /// [`BTreeMap`]: std::collections::BTreeMap
    /// [lua2.1]: https://www.lua.org/manual/5.4/manual.html#2.1
    /// [lua3.4.9]: https://www.lua.org/manual/5.4/manual.html#3.4.9
    Table(Vec<LuaTableEntry<'a>>),
}

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "wasm32",
))]
assert_eq_size!((usize, usize, LuaNumber), LuaValue<'_>);

impl Debug for LuaValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Boolean(b) => f.debug_tuple("Boolean").field(b).finish(),
            Self::String(s) => f
                .debug_tuple("String")
                .field(&s.escape_ascii().to_string())
                .finish(),
            Self::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Self::Table(t) => f.debug_tuple("Table").field(t).finish(),
        }
    }
}

impl<'a> LuaValue<'a> {
    /// Make a LuaValue from [`i64`].
    #[inline]
    pub const fn integer(v: i64) -> Self {
        Self::Number(LuaNumber::Integer(v))
    }

    /// Make a LuaValue from [`f64`].
    #[inline]
    pub const fn float(v: f64) -> Self {
        Self::Number(LuaNumber::Float(v))
    }

    /// Returns `true` for a [`LuaValue::Number`] that is not a number.
    ///
    /// Other types return `false`, even if they are not numbers.
    #[inline]
    pub const fn is_nan(&self) -> bool {
        matches!(self, LuaValue::Number(n) if n.is_nan())
    }

    /// Returns `true` for a [`LuaValue::Number`] that is finite.
    ///
    /// Other types return `false`.
    #[inline]
    pub const fn is_finite(&self) -> bool {
        matches!(self, LuaValue::Number(n) if n.is_finite())
    }

    /// Returns `true` for a [`LuaValue::Number`] that is infinite.
    ///
    /// Other types return `false`.
    #[inline]
    pub const fn is_infinite(&self) -> bool {
        matches!(self, LuaValue::Number(n) if n.is_infinite())
    }

    /// Returns `true` if the [`LuaValue`] is _entirely_ [borrowed][Cow::Borrowed].
    ///
    /// Returns `true` for `LuaValue::String(Cow::Borrowed(_))`, `false` otherwise.
    #[inline]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, LuaValue::String(Cow::Borrowed(_)))
    }

    /// Returns the value as a byte array, if it contains [a string][LuaValue::String].
    ///
    /// Lua strings may contain arbitrary binary data, with no defined encoding. This may not decode
    /// as UTF-8, or it otherwise may decode with _incorrect data_.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::String(b"hello world".into());
    /// assert_eq!(a.as_bytes().unwrap(), b"hello world".as_slice());
    ///
    /// let b = LuaValue::String(b"\0\xC0".into());
    /// assert_eq!(b.as_bytes().unwrap(), b"\0\xC0".as_slice());
    ///
    /// let c = LuaValue::Boolean(true);
    /// assert!(c.as_bytes().is_none());
    /// ```
    #[inline]
    pub fn as_bytes(&'a self) -> Option<Cow<'a, [u8]>> {
        match self {
            Self::String(s) => Some(Cow::Borrowed(s)),
            _ => None,
        }
    }

    /// Returns the value as a string, if it contains a UTF-8-encoded [string][LuaValue::String].
    ///
    /// Lua strings may contain arbitrary binary data, with no defined encoding. This may not decode
    /// as UTF-8 (so will return `None`), or it otherwise may decode with _incorrect data_.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::String(b"hello world".into());
    /// assert_eq!(a.as_str().unwrap(), "hello world");
    ///
    /// let b = LuaValue::String(b"\0\xC0".into());
    /// assert!(b.as_str().is_none());
    ///
    /// let c = LuaValue::Boolean(true);
    /// assert!(c.as_str().is_none());
    /// ```
    pub fn as_str(&'a self) -> Option<Cow<'a, str>> {
        match self {
            Self::String(s) => from_utf8_cow(Cow::Borrowed(s)).ok(),
            _ => None,
        }
    }

    /// Returns the value as a string, if it contains [a string][LuaValue::String].
    ///
    /// If the data cannot be decoded as UTF-8, it will be returned
    /// [lossily][String::from_utf8_lossy].
    ///
    /// Lua strings may contain arbitrary binary data, with no defined encoding. This may not decode
    /// as UTF-8, or it otherwise may decode with _incorrect data_.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::String(b"hello world".into());
    /// assert_eq!(a.as_str_lossy().unwrap(), "hello world");
    ///
    /// let b = LuaValue::String(b"\0\xC0".into());
    /// assert_eq!(b.as_str_lossy().unwrap(), "\0\u{FFFD}");
    ///
    /// let c = LuaValue::Boolean(true);
    /// assert!(c.as_str_lossy().is_none());
    /// ```
    pub fn as_str_lossy(&'a self) -> Option<Cow<'a, str>> {
        match self {
            Self::String(s) => Some(from_utf8_cow_lossy(Cow::Borrowed(s))),
            _ => None,
        }
    }

    /// Returns the value as a `bool`, if it contains [a boolean][LuaValue::Boolean].
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::Boolean(true);
    /// assert!(a.as_bool().unwrap());
    ///
    /// let b = LuaValue::String(b"hello world".into());
    /// assert!(b.as_bool().is_none());
    /// ```
    #[inline]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the value as an `f64`, if it contains [a number][LuaValue::Number].
    ///
    /// This will convert integer values to floating point if they can be represented without a loss
    /// of precision `[-(2**53)+1, (2**53)-1]`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::float(3.14);
    /// assert_eq!(a.as_f64().unwrap(), 3.14);
    ///
    /// let b = LuaValue::integer(123);
    /// assert_eq!(b.as_f64().unwrap(), 123.);
    ///
    /// let c = LuaValue::integer(i64::MAX);
    /// assert!(c.as_f64().is_none());
    ///
    /// let d = LuaValue::Boolean(true);
    /// assert!(d.as_f64().is_none());
    /// ```
    #[inline]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns the value as an `i64`, if it contains [an integer number][LuaNumber::Integer].
    ///
    /// ## Example
    ///
    /// ```rust
    /// use serde_luaq::LuaValue;
    ///
    /// let a = LuaValue::integer(123);
    /// assert_eq!(a.as_i64().unwrap(), 123);
    ///
    /// let b = LuaValue::integer(i64::MIN);
    /// assert_eq!(b.as_i64().unwrap(), i64::MIN);
    ///
    /// let c = LuaValue::float(3.);
    /// assert!(c.as_i64().is_none());
    ///
    /// let d = LuaValue::Boolean(true);
    /// assert!(d.as_f64().is_none());
    /// ```
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    // pub fn repr(&self, o: &mut Vec<u8>) {

    //     match self {
    //         Self::Nil => o.extend_from_slice(b"nil"),
    //         Self::Boolean(b) => o.extend_from_slice(b.to_string().as_bytes()),
    //         Self::Number(n) => o.extend_from_slice(n.to_string().as_bytes()),
    //         Self::Str(s) => {
    //             o.push(b'"');
    //             for b in s.iter() {
    //                 // escape things
    //                 match b {
    //                     b'\\' => o.extend_from_slice(b"\\\\"),
    //                     b'"' => o.extend_from_slice(b"\\\""),
    //                     b'\n' => o.extend_from_slice(b"\\\n"),
    //                     b'\r' => o.extend_from_slice(b"\\\r"),
    //                     b => o.push(*b),
    //                 }
    //             }
    //             o.push(b'"');
    //         },
    //         Self::Table(t) => {
    //             o.push(b'{');
    //             for e in t.iter() {
    //                 e.repr(o);
    //                 o.push(b',');
    //             }
    //             o.push(b'}');
    //         }
    //     }

    // }
}

impl<T> From<T> for LuaValue<'_>
where
    LuaNumber: From<T>,
{
    fn from(value: T) -> Self {
        Self::Number(LuaNumber::from(value))
    }
}

impl<'a> From<&'a [u8]> for LuaValue<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::String(value.into())
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for LuaValue<'a> {
    fn from(value: &'a [u8; N]) -> Self {
        Self::String(value.as_slice().into())
    }
}

impl<'a> From<&'a str> for LuaValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(Cow::Borrowed(value.as_bytes()))
    }
}

impl From<String> for LuaValue<'_> {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value.into_bytes()))
    }
}

impl From<bool> for LuaValue<'_> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl<'a> FromIterator<(&'a [u8], LuaValue<'a>)> for LuaValue<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a [u8], LuaValue<'a>)>>(iter: T) -> Self {
        LuaValue::Table(iter.into_iter().map(From::from).collect())
    }
}

impl<'a> FromIterator<(&'a str, LuaValue<'a>)> for LuaValue<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a str, LuaValue<'a>)>>(iter: T) -> Self {
        LuaValue::Table(iter.into_iter().map(From::from).collect())
    }
}

impl<'a> From<Vec<LuaTableEntry<'a>>> for LuaValue<'a> {
    fn from(value: Vec<LuaTableEntry<'a>>) -> Self {
        LuaValue::Table(value)
    }
}

impl<'a> FromIterator<LuaTableEntry<'a>> for LuaValue<'a> {
    fn from_iter<T: IntoIterator<Item = LuaTableEntry<'a>>>(iter: T) -> Self {
        LuaValue::Table(iter.into_iter().collect())
    }
}

// We can't implement TryFrom for From types
macro_rules! lua_value_tryfrom_number {
    ($($ty:ty)*) => {$(
        impl TryFrom<$ty> for LuaValue<'_> {
            type Error = <LuaNumber as TryFrom<$ty>>::Error;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                LuaNumber::try_from(value).map(Self::Number)
            }
        }
    )*};
}

lua_value_tryfrom_number! { u64 isize usize i128 u128 }

impl<'a, T> From<Option<T>> for LuaValue<'a>
where
    LuaValue<'a>: From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map(LuaValue::from).unwrap_or(LuaValue::Nil)
    }
}

/// Attempts to convert a `Cow<'a, [u8]>` into a `Cow<'a, str>` while avoiding
/// copying.
pub(crate) fn from_utf8_cow(v: Cow<'_, [u8]>) -> Result<Cow<'_, str>, (Utf8Error, Cow<'_, [u8]>)> {
    match v {
        Cow::Borrowed(v) => from_utf8(v)
            .map(Cow::Borrowed)
            .map_err(|e| (e, Cow::Borrowed(v))),
        Cow::Owned(v) => String::from_utf8(v)
            .map(Cow::Owned)
            .map_err(|e| (e.utf8_error(), Cow::Owned(e.into_bytes()))),
    }
}

/// Attempts to lossily convert a `Cow<'a, [u8]>` into a `Cow<'a, str>` while
/// avoiding copying.
pub(crate) fn from_utf8_cow_lossy(v: Cow<'_, [u8]>) -> Cow<'_, str> {
    match v {
        Cow::Borrowed(v) => String::from_utf8_lossy(v),
        // TODO: replace with from_utf8_lossy_owned: https://github.com/rust-lang/rust/issues/129436
        Cow::Owned(v) => Cow::Owned(String::from_utf8_lossy(&v).into_owned()),
    }
}

/// Converts a `Cow<'a, str>` into a `Cow<'a, [u8]>` while avoiding copying.
pub(crate) fn to_utf8_cow(v: Cow<'_, str>) -> Cow<'_, [u8]> {
    match v {
        Cow::Borrowed(v) => Cow::Borrowed(v.as_bytes()),
        Cow::Owned(v) => Cow::Owned(v.into_bytes()),
    }
}

impl PartialEq<LuaNumber> for LuaValue<'_> {
    fn eq(&self, other: &LuaNumber) -> bool {
        match self {
            LuaValue::Number(n) => n == other,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{cmp::PartialEq, fmt::Debug};

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    wasm_bindgen_test_configure!(run_in_browser);

    fn assert_cow_eq<'a, T, U>(expected: U, is_borrowed: bool, actual: Cow<'a, T>)
    where
        T: ToOwned + Debug + ?Sized,
        <T as ToOwned>::Owned: Debug,
        T: PartialEq,
        U: AsRef<T> + Debug + PartialEq<Cow<'a, T>>,
    {
        assert_eq!(is_borrowed, matches!(actual, Cow::Borrowed(_)));
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn cow() {
        assert_cow_eq("foo", true, from_utf8_cow(b"foo".into()).unwrap());
        assert_cow_eq("foo", true, from_utf8_cow_lossy(b"foo".into()));

        assert_cow_eq("foo", false, from_utf8_cow(b"foo".to_vec().into()).unwrap());
        assert_cow_eq("foo", false, from_utf8_cow_lossy(b"foo".to_vec().into()));

        // Errors should return the borrowed value.
        let (_, cow) = from_utf8_cow(b"\xFEfoo".into()).unwrap_err();
        assert_cow_eq(Cow::Borrowed(b"\xFEfoo".as_slice()), true, cow);

        // Lossy conversions may copy
        let cow = from_utf8_cow_lossy(b"\xFEfoo".into());
        assert_cow_eq("\u{FFFD}foo", false, cow);
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn from_bool_option() {
        // bool
        assert_eq!(LuaValue::Boolean(true), LuaValue::from(true));
        assert_eq!(LuaValue::Boolean(false), LuaValue::from(false));

        // Option<T>
        assert_eq!(LuaValue::Nil, LuaValue::from(None::<bool>));
        assert_eq!(LuaValue::Boolean(true), LuaValue::from(Some(true)));
        assert_eq!(LuaValue::Boolean(false), LuaValue::from(Some(false)));
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn from_integer() {
        // i64
        for x in [0, i64::MIN, i64::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x), v);
            assert_eq!(LuaValue::integer(x), LuaNumber::Integer(x));
            assert_eq!(LuaNumber::Integer(x), LuaValue::integer(x));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // i32
        for x in [0, i32::MIN, i32::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // i16
        for x in [0, i16::MIN, i16::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // i8
        for x in [0, i8::MIN, i8::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // u64
        assert_eq!(LuaValue::integer(0), LuaValue::try_from(0u64).unwrap());
        LuaValue::try_from(u64::MAX).unwrap_err();

        // u32
        for x in [0, u32::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // u16
        for x in [0, u16::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }

        // u8
        for x in [0, u8::MAX] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::integer(x.into()), v);
            assert_eq!(LuaValue::integer(x.into()), LuaNumber::Integer(x.into()));
            assert_eq!(LuaNumber::Integer(x.into()), LuaValue::integer(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::integer(1), v);
        }
    }

    #[test]
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    fn from_float() {
        // f64
        for x in [0., f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::float(x), v);
            assert_eq!(LuaValue::float(x), LuaNumber::Float(x));
            assert_eq!(LuaNumber::Float(x), LuaValue::float(x));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::float(1.), v);
        }

        let f = LuaValue::from(f64::NAN);
        assert!(matches!(f, LuaValue::Number(LuaNumber::Float(x)) if x.is_nan()));

        // f32
        for x in [0., f32::MIN, f32::MAX, f32::INFINITY, f32::NEG_INFINITY] {
            let v = LuaValue::from(x);
            assert_eq!(LuaValue::float(x.into()), v);
            assert_eq!(LuaValue::float(x.into()), LuaNumber::Float(x.into()));
            assert_eq!(LuaNumber::Float(x.into()), LuaValue::float(x.into()));
            assert_ne!(LuaValue::Boolean(false), v);
            assert_ne!(LuaValue::float(1.), v);
        }

        let f = LuaValue::from(f32::NAN);
        assert!(matches!(f, LuaValue::Number(LuaNumber::Float(x)) if x.is_nan()));
    }
}
