use crate::{stream_len, stream_position, Endian, EnumConversionError};
use std::{
    convert::TryInto,
    error::Error,
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    /// How many bytes were expected
    ExpectedBytes(usize),
    /// We expected .0, but found .1
    ExpectedBytesFound(usize, usize),
    /// Unexpected End of File. Basically another expected bytes, but no expected amount.
    UnexpectedEOF,
    /// We expected there to be no more bytes!
    ExpectedEOF,
    /// There was an invalid enumeration somewhere.
    InvalidEnumerationValue,
    /// There was an invalid enumeration that had a name to give us
    InvalidEnumerationValueNamed(&'static str),
    /// It read a byte that was invalid.
    InvalidByte,
    Custom(Box<dyn Error>),
}
impl From<std::io::Error> for ParseError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}
impl<V> From<EnumConversionError<V>> for ParseError {
    fn from(e: EnumConversionError<V>) -> Self {
        match e {
            EnumConversionError::InvalidValue(_) => ParseError::InvalidEnumerationValue,
        }
    }
}

pub type ParseResult<R, E = ParseError> = Result<R, E>;

pub fn single<F>(f: &mut F) -> ParseResult<u8>
where
    F: Read,
{
    let mut output = [0u8];
    f.read_exact(&mut output)?;
    Ok(output[0])
}

// TODO: take_peek
// TODO: const generics version that takes in the size as a template param
//  and returns an array of that size
pub fn take<F>(f: &mut F, amount: usize) -> ParseResult<Vec<u8>>
where
    F: Read,
{
    let mut output = Vec::new();
    output.resize(amount, 0);

    f.read_exact(&mut output)?;

    Ok(output)
}

/// More efficient than parsing [u8; N]
pub fn take_n<F: Read, const N: usize>(f: &mut F) -> ParseResult<[u8; N]> {
    let mut output = [0_u8; N];
    f.read_exact(&mut output)?;
    Ok(output)
}

// TODO: take_until_peek
/// Takes all bytes until (and including the terminator).
/// If [include_terminator] is true, then the terminator is included in the output.
/// Otherwise, the terminator is not included. (but still consumed!)
pub fn take_until<F>(f: &mut F, terminator: u8, include_terminator: bool) -> ParseResult<Vec<u8>>
where
    F: Read,
{
    // FIXME: for some reason BString does not have a new function.
    let mut result = Vec::new();
    loop {
        let mut buffer = [0u8; 1];
        // If this errors before we find the null-terminator, then it's.. an error.
        // Ex: If it gets an EOF error, then it should have given us more bytes.
        f.read_exact(&mut buffer)?;
        let value = buffer[0];
        if value == terminator {
            if include_terminator {
                result.push(value);
            }
            break;
        }

        result.push(value);
    }

    Ok(result)
}

// TODO: many_peek
/// This loops until it has consumed everything, or reached an error
/// Note that this does not rollback when it encounters an error
/// and should be used when you know that what you're reading from is
/// parseable by repeated calls to [func]
pub fn many<F, C, R, E, D>(f: &mut F, d: D, func: C) -> ParseResult<Vec<R>>
where
    F: Read + Seek,
    C: Fn(&mut F, D) -> Result<R, E>,
    E: Into<ParseError>,
    D: Clone,
{
    let mut result: Vec<R> = Vec::new();
    let stream_len = stream_len(f)?;
    loop {
        if stream_position(f)? >= stream_len {
            break;
        }

        let value: R = match func(f, d.clone()) {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };
        result.push(value);
    }

    debug_assert_eq!(stream_position(f)?, stream_len);

    Ok(result)
}

// TODO: many_parse_peek
pub fn many_parse<'a, F, P, D>(f: &'a mut F, d: D) -> ParseResult<Vec<P>>
where
    F: Read + Seek,
    P: Parse<'a, F, D>,
    D: Clone,
{
    let mut result = Vec::new();
    let stream_len = stream_len(f)?;

    loop {
        if stream_position(f)? >= stream_len {
            break;
        }

        // TODO: if we're passed a reference, does this clone the reference or the type behind the reference?
        let value: P = P::parse(f, d.clone())?;
        result.push(value);
    }

    debug_assert_eq!(stream_position(f)?, stream_len);

    Ok(result)
}

// TODO: tag_peek
/// Expect certain bytes. Does not return them.
pub fn tag<F, X>(f: &mut F, data: &[X]) -> ParseResult<()>
where
    F: Read,
    X: PartialEq<u8>,
{
    for x in data.iter() {
        let value = single(f)?;
        if x != &value {
            return Err(ParseError::InvalidByte);
        }
    }
    Ok(())
}

// Internal utilities, since const generics don't exist
fn take_2<F>(f: &mut F) -> ParseResult<[u8; 2]>
where
    F: Read,
{
    let mut output = [0u8; 2];
    f.read_exact(&mut output)?;
    Ok(output)
}

fn take_4<F>(f: &mut F) -> ParseResult<[u8; 4]>
where
    F: Read,
{
    let mut output = [0u8; 4];
    f.read_exact(&mut output)?;
    Ok(output)
}

fn take_8<F>(f: &mut F) -> ParseResult<[u8; 8]>
where
    F: Read,
{
    let mut output = [0u8; 8];
    f.read_exact(&mut output)?;
    Ok(output)
}

// TODO: should this take a template for what error it returns.. that would complicate things
// TODO: It would be nice to allow non-Seek types. At the very least, forward seek can be
//   ''implemented'' by reading data and throwing it away.
//     Most parsing probably doesn't need arbitrarty seeking.
pub trait Parse<'a, F, D = ()>: Sized
where
    F: Read,
{
    fn parse<'b>(f: &'b mut F, d: D) -> ParseResult<Self>;

    // TODO: would it be nice to diffrentiate between seek errors and parsing errors?
    // It's not *too* much of a gain, but it is more correct.
    /// Parse the data, then move back to the initial position. Useful for peeking ahead.
    /// Note: this function only works if [stream_position] works upon the type.
    /// If the state is modified by seeking it back and forth, then this has side effects.
    /// This works fine much of the time.
    /// If there is an error originating from the seeks, then the file position is undefined.
    /// If there is an error from parsing the data, it should be back to where it was previously.
    fn parse_peek(f: &'a mut F, d: D) -> ParseResult<Self>
    where
        F: Seek,
    {
        // Store the starting position
        let initial_position = stream_position(f)?;
        let data = Self::parse(f, d);
        // First we seek back to our starting position, before dealing with the data
        f.seek(SeekFrom::Start(initial_position))?;
        data
    }
}

impl<F: Read> Parse<'_, F> for u8 {
    fn parse(f: &mut F, _d: ()) -> ParseResult<Self> {
        Ok(u8::from_le_bytes([single(f)?]))
    }
}

impl<F: Read> Parse<'_, F> for i8 {
    fn parse(f: &mut F, _d: ()) -> ParseResult<Self> {
        Ok(i8::from_le_bytes([single(f)?]))
    }
}
impl<F: Read> Parse<'_, F, Endian> for u16 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_2(f)?;
        Ok(match endian {
            Endian::Big => u16::from_be_bytes(data),
            Endian::Little => u16::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for i16 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_2(f)?;
        Ok(match endian {
            Endian::Big => i16::from_be_bytes(data),
            Endian::Little => i16::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for u32 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_4(f)?;
        Ok(match endian {
            Endian::Big => u32::from_be_bytes(data),
            Endian::Little => u32::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for i32 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_4(f)?;
        Ok(match endian {
            Endian::Big => i32::from_be_bytes(data),
            Endian::Little => i32::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for u64 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_8(f)?;
        Ok(match endian {
            Endian::Big => u64::from_be_bytes(data),
            Endian::Little => u64::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for i64 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_8(f)?;
        Ok(match endian {
            Endian::Big => i64::from_be_bytes(data),
            Endian::Little => i64::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for f32 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_4(f)?;
        Ok(match endian {
            Endian::Big => f32::from_be_bytes(data),
            Endian::Little => f32::from_le_bytes(data),
        })
    }
}
impl<F: Read> Parse<'_, F, Endian> for f64 {
    fn parse(f: &mut F, endian: Endian) -> ParseResult<Self> {
        let data = take_8(f)?;
        Ok(match endian {
            Endian::Big => f64::from_be_bytes(data),
            Endian::Little => f64::from_le_bytes(data),
        })
    }
}

impl<'a, D: Clone, F: Read, T: Parse<'a, F, D>, const N: usize> Parse<'a, F, D> for [T; N] {
    fn parse<'b>(f: &'b mut F, d: D) -> ParseResult<Self> {
        // TODO: This might be able to be optimized out, but I don't want to rely on that.
        let mut data: Vec<T> = Vec::with_capacity(N);
        for _ in 0..N {
            data.push(T::parse(f, d.clone())?);
        }

        Ok(data.try_into().unwrap_or_else(|_| unreachable!()))
    }
}

// TODO: add tests for impl_parse_field and impl_parse
#[macro_export]
macro_rules! impl_parse_field {
    ($name:ident : l : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse($input, Endian::Little)?;
    };
    ($name:ident : b : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse($input, Endian::Big)?;
    };
    // No data
    ($name:ident : u : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse($input, ())?;
    };
}

#[macro_export]
macro_rules! impl_parse {
    ($on:ty, [$($name:ident : $e:ident : $typ:ty),*]) => {
        impl $crate::parse::Parse<'_, ()> for $on {
            fn parse(f: &mut F, _d: ()) -> $crate::parse::ParseResult<Self>
            where
                F: std::io::Seek + std::io::Read {
                $(
                    $crate::impl_parse_field!($name : $e : $typ; f);
                )*
                Ok(Self {
                    $($name),*
                })
            }
        }
    };
    (newtype $on:ty, $name:ident : $e:ident : $typ:ty) => {
        impl $crate::parse::Parse<'_, ()> for $on {
            fn parse(f: &mut F, _d: ()) -> $crate::parse::ParseResult<Self>
            where
                F: std::io::Seek + std::io::Read {
                $crate::impl_parse_field!($name: $e : $typ; f);
                Ok(Self($name))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const DATA: [u8; 20] = [
        0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10, 0x11,
        0x12, 0x13, 0x14,
    ];

    #[test]
    fn test_single() {
        let mut cursor = Cursor::new(&DATA);
        assert_eq!(single(&mut cursor).unwrap(), 0x1);
        assert_eq!(single(&mut cursor).unwrap(), 0x2);
        assert_eq!(single(&mut cursor).unwrap(), 0x3);
        assert_eq!(single(&mut cursor).unwrap(), 0x4);
    }

    #[test]
    fn test_take() {
        let mut cursor = Cursor::new(&DATA);
        assert_eq!(
            take(&mut cursor, 4).unwrap().as_slice(),
            &[0x1, 0x2, 0x3, 0x4]
        );
        assert_eq!(
            take(&mut cursor, 4).unwrap().as_slice(),
            &[0x5, 0x6, 0x7, 0x8]
        );
    }

    #[test]
    fn test_tag() {
        let mut cursor = Cursor::new(&DATA);
        tag(&mut cursor, &[0x1, 0x2, 0x3, 0x4]).unwrap();
        tag(&mut cursor, &[0x5, 0x6, 0x7, 0x8]).unwrap();

        tag(&mut cursor, &[0x20, 0x52]).expect_err("Expected error since invalid bytes!");
    }

    #[test]
    fn test_many() {
        let mut cursor = Cursor::new(&DATA);
        let result = many(&mut cursor, (), |f, _d| -> ParseResult<[u8; 4]> {
            let data = take(f, 4)?;
            Ok([data[0], data[1], data[2], data[3]])
        })
        .unwrap();
        assert_eq!(
            result.as_slice(),
            &[
                [0x1, 0x2, 0x3, 0x4],
                [0x5, 0x6, 0x7, 0x8],
                [0x9, 0xa, 0xb, 0xc],
                [0xd, 0xe, 0xf, 0x10],
                [0x11, 0x12, 0x13, 0x14]
            ]
        );

        let mut cursor = Cursor::new(&DATA);
        many(&mut cursor, (), |f, _d| -> ParseResult<[u8; 3]> {
            let data = take(f, 3)?;
            Ok([data[0], data[1], data[2]])
        })
        .expect_err("Expected failure in dividing DATA into 3-byte chunks");
    }

    #[test]
    fn test_take_until() {
        let mut cursor = Cursor::new(&DATA);
        let result = take_until(&mut cursor, 0x7, false).unwrap();
        assert_eq!(result.as_slice(), &[0x1, 0x2, 0x3, 0x4, 0x5, 0x6]);
        assert_eq!(stream_position(&mut cursor).unwrap(), 7);
        let result = take_until(&mut cursor, 0xa, true).unwrap();
        // Note: cursor has been moved by previous.
        assert_eq!(result.as_slice(), &[0x8, 0x9, 0xa]);
        assert_eq!(stream_position(&mut cursor).unwrap(), 10);
    }

    #[test]
    fn test_parse_peek() {
        let mut cursor = Cursor::new(&DATA);
        let value = u16::parse_peek(&mut cursor, Endian::Big).unwrap();
        assert_eq!(value, 0x0102);
        assert_eq!(stream_position(&mut cursor).unwrap(), 0);
    }
}
