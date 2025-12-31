use crate::{valid_lua_identifier, Error, LuaNumber, LuaTableEntry, LuaValue, Result};
use hexfloat2::format as hexfloat_format;
use std::io;

#[derive(Clone, Debug, Default)]
pub struct CompactFormatter;
impl Formatter for CompactFormatter {}

/// Lua formatted code writer.
pub trait Formatter {
    fn write_nil<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"nil")
    }

    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(if value { b"true" } else { b"false" })
    }

    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if value == 0. {
            writer.write_all(b"0x0p0")
        } else if value.is_finite() {
            writer.write_all(hexfloat_format(value).as_bytes())
        } else {
            writer.write_all(if value.is_nan() {
                b"(0/0)"
            } else if value == f32::INFINITY {
                b"1e9999"
            } else if value == f32::NEG_INFINITY {
                b"-1e9999"
            } else {
                unreachable!()
            })
        }
    }

    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if value == 0. {
            writer.write_all(b"0x0p0")
        } else if value.is_finite() {
            writer.write_all(hexfloat_format(value).as_bytes())
        } else {
            writer.write_all(if value.is_nan() {
                b"(0/0)"
            } else if value == f64::INFINITY {
                b"1e9999"
            } else if value == f64::NEG_INFINITY {
                b"-1e9999"
            } else {
                unreachable!()
            })
        }
    }

    fn write_luanumber<W>(&mut self, writer: &mut W, value: LuaNumber) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        match value {
            LuaNumber::Float(f) => self.write_f64(writer, f),
            LuaNumber::Integer(i) => self.write_i64(writer, i),
        }
    }

    #[inline]
    fn write_empty_table<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{}")
    }

    fn begin_table<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{")
    }

    fn end_table<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"}")
    }

    fn begin_table_entry<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    fn end_table_entry<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    fn begin_table_value_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"[")
    }

    fn end_table_value_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"]")
    }

    fn begin_table_identifier_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    fn end_table_identifier_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    fn begin_table_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"=")
    }

    fn end_table_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_byte_escape<W>(&mut self, writer: &mut W, byte_escape: ByteEscape) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        use self::ByteEscape::*;

        let escape_char = match byte_escape {
            Quote => QU,
            ReverseSolidus => BS,
            Bell => AA,
            Backspace => BB,
            FormFeed => FF,
            LineFeed => NN,
            CarriageReturn => RR,
            HorizontalTab => TT,
            VerticalTab => VV,
            AsciiControl(_) => b'x',
        };

        match byte_escape {
            AsciiControl(byte) => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[
                    b'\\',
                    escape_char,
                    HEX_DIGITS[(byte >> 4) as usize],
                    HEX_DIGITS[(byte & 0xF) as usize],
                ];
                writer.write_all(bytes)
            }
            _ => writer.write_all(&[b'\\', escape_char]),
        }
    }

    /// Writes a bytes fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_bytes_fragment<W>(&mut self, writer: &mut W, fragment: &[u8]) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(fragment)
    }
}

/// Represents a byte escape code in a type-safe manner.
pub enum ByteEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    /// An escaped bell (usually escaped as `\a`)
    Bell,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped horizontal tab character (usually escaped as `\t`)
    HorizontalTab,
    /// An escaped vertical tab character (usually escaped as `\v`)
    VerticalTab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\xXX` where `XX` are two hex characters)
    AsciiControl(u8),
}

fn format_escaped_bytes_contents<W, F>(
    writer: &mut W,
    formatter: &mut F,
    mut bytes: &[u8],
) -> io::Result<()>
where
    W: ?Sized + io::Write,
    F: ?Sized + Formatter,
{
    let mut i = 0;
    while i < bytes.len() {
        let (string_run, rest) = bytes.split_at(i);
        let (&byte, rest) = rest.split_first().unwrap();

        let escape = ESCAPE[byte as usize];

        i += 1;
        if escape == 0 {
            continue;
        }

        bytes = rest;
        i = 0;

        if !string_run.is_empty() {
            formatter.write_bytes_fragment(writer, string_run)?;
        }

        // Safety: string_run is a valid utf8 string, since we only split on ascii sequences
        // let string_run = unsafe { str::from_utf8_unchecked(string_run) };
        // if !string_run.is_empty() {
        //     tri!(formatter.write_string_fragment(writer, string_run));
        // }

        let char_escape = match escape {
            self::AA => ByteEscape::Bell,
            self::BB => ByteEscape::Backspace,
            self::TT => ByteEscape::HorizontalTab,
            self::NN => ByteEscape::LineFeed,
            self::VV => ByteEscape::VerticalTab,
            self::FF => ByteEscape::FormFeed,
            self::RR => ByteEscape::CarriageReturn,
            self::QU => ByteEscape::Quote,
            self::BS => ByteEscape::ReverseSolidus,
            self::XX => ByteEscape::AsciiControl(byte),
            _ => unreachable!(),
        };

        formatter.write_byte_escape(writer, char_escape)?;
    }

    // Safety: bytes is a valid utf8 string, since we only split on ascii sequences
    if bytes.is_empty() {
        return Ok(());
    }

    formatter.write_bytes_fragment(writer, bytes)
}

const AA: u8 = b'a'; // \x07
const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const VV: u8 = b'v'; // \x0B
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const XX: u8 = b'x'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in Lua. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    XX, XX, XX, XX, XX, XX, XX, AA, BB, TT, NN, VV, FF, RR, XX, XX, // 0
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

impl LuaNumber {
    pub fn to_writer<W, F>(&self, writer: &mut W, formatter: &mut F) -> Result<()>
    where
        F: Formatter,
        W: ?Sized + io::Write,
    {
        Ok(formatter.write_luanumber(writer, *self)?)
    }
}

impl LuaTableEntry<'_> {
    pub fn to_writer<W, F>(&self, writer: &mut W, formatter: &mut F, first: bool) -> Result<()>
    where
        F: Formatter,
        W: ?Sized + io::Write,
    {
        formatter.begin_table_entry(writer, first)?;
        match self {
            Self::NilValue => {
                formatter.write_nil(writer)?;
            }
            Self::BooleanValue(value) => formatter.write_bool(writer, *value)?,
            Self::NumberValue(value) => value.to_writer(writer, formatter)?,

            Self::Value(value) => value.to_writer(writer, formatter)?,

            Self::NameValue(b) => {
                if !valid_lua_identifier(b.0.as_bytes()) {
                    return Err(Error::InvalidLuaIdentifier(b.0.to_string()));
                }

                formatter.begin_table_identifier_key(writer)?;
                formatter.write_bytes_fragment(writer, b.0.as_bytes())?;
                formatter.end_table_identifier_key(writer)?;

                formatter.begin_table_value(writer)?;
                b.1.to_writer(writer, formatter)?;
                formatter.end_table_value(writer)?;
            }

            Self::KeyValue(b) => {
                formatter.begin_table_value_key(writer)?;
                b.0.to_writer(writer, formatter)?;
                formatter.end_table_value_key(writer)?;

                formatter.begin_table_value(writer)?;
                b.1.to_writer(writer, formatter)?;
                formatter.end_table_value(writer)?;
            }
        }

        formatter.end_table_entry(writer)?;
        Ok(())
    }
}

impl LuaValue<'_> {
    pub fn to_writer<W, F>(&self, writer: &mut W, formatter: &mut F) -> Result<()>
    where
        F: Formatter,
        W: ?Sized + io::Write,
    {
        match self {
            Self::Nil => formatter.write_nil(writer)?,
            Self::Boolean(value) => formatter.write_bool(writer, *value)?,
            Self::Number(value) => value.to_writer(writer, formatter)?,
            Self::String(value) => {
                format_escaped_bytes_contents(writer, formatter, value)?;
            }
            Self::Table(table) => {
                formatter.begin_table(writer)?;
                let mut first = true;
                for entry in table {
                    entry.to_writer(writer, formatter, first)?;
                    first = false;
                }
                formatter.end_table(writer)?;
            }
        }
        Ok(())
    }
}
