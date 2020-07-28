use crate::{stream_len, stream_position, Endian};
use std::{
    fmt::Debug,
    io::{Read, Seek},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParseData<D>
where
    D: Debug + Clone + PartialEq,
{
    pub endian: Endian,
    pub data: D,
}
impl<D> ParseData<D>
where
    D: Debug + Clone + PartialEq,
{
    pub fn from_endian(endian: Endian, data: D) -> Self {
        Self { endian, data }
    }

    pub fn clear(&self) -> ParseData<()> {
        ParseData::from_empty(self.endian)
    }
}
impl ParseData<()> {
    pub const fn from_empty(endian: Endian) -> ParseData<()> {
        Self { endian, data: () }
    }
}

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
}
impl From<std::io::Error> for ParseError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

pub type ParseResult<R, E = ParseError> = Result<R, E>;

pub fn single<F>(f: &mut F, _d: &mut ParseData<()>) -> ParseResult<u8>
where
    F: Read + Seek,
{
    let mut output = [0u8];
    f.read_exact(&mut output)?;
    Ok(output[0])
}

// TODO: const generics version that takes in the size as a template param
//  and returns an array of that size
pub fn take<F>(f: &mut F, _d: &mut ParseData<()>, amount: usize) -> ParseResult<Vec<u8>>
where
    F: Read + Seek,
{
    let mut output = Vec::new();
    output.resize(amount, 0);

    f.read_exact(&mut output)?;

    Ok(output)
}

/// This loops until it has consumed everything, or reached an error
/// Note that this does not rollback when it encounters an error
/// and should be used when you know that what you're reading from is
/// parseable by repeated calls to [func]
pub fn many<F, C, R, E, D>(f: &mut F, d: &mut ParseData<D>, func: C) -> ParseResult<Vec<R>>
where
    F: Read + Seek,
    C: Fn(&mut F, &mut ParseData<D>) -> Result<R, E>,
    E: Into<ParseError>,
    D: Debug + Clone + PartialEq,
{
    let mut result: Vec<R> = Vec::new();
    let stream_len = stream_len(f)?;
    loop {
        if stream_position(f)? >= stream_len {
            break;
        }

        let value: R = match func(f, d) {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };
        result.push(value);
    }

    debug_assert_eq!(stream_position(f)?, stream_len);

    Ok(result)
}

pub fn many_parse<F, P, D>(f: &mut F, d: &mut ParseData<D>) -> ParseResult<Vec<P>>
where
    F: Read + Seek,
    P: Parse<D>,
    D: Debug + Clone + PartialEq,
{
    let mut result = Vec::new();
    let stream_len = stream_len(f)?;

    loop {
        if stream_position(f)? >= stream_len {
            break;
        }

        let value: P = P::parse(f, d)?;
        result.push(value);
    }

    debug_assert_eq!(stream_position(f)?, stream_len);

    Ok(result)
}

/// Expect certain bytes. Does not return them.
pub fn tag<F, X>(f: &mut F, d: &mut ParseData<()>, data: &[X]) -> ParseResult<()>
where
    F: Read + Seek,
    X: PartialEq<u8>,
{
    for x in data.iter() {
        let value = single(f, d)?;
        if x != &value {
            return Err(ParseError::InvalidByte);
        }
    }
    Ok(())
}

// Internal utilities, since const generics don't exist
fn take_2<F, D>(f: &mut F, _d: &mut ParseData<D>) -> ParseResult<[u8; 2]>
where
    F: Read + Seek,
    D: Debug + Clone + PartialEq,
{
    let mut output = [0u8; 2];
    f.read_exact(&mut output)?;
    Ok(output)
}

fn take_4<F, D>(f: &mut F, _d: &mut ParseData<D>) -> ParseResult<[u8; 4]>
where
    F: Read + Seek,
    D: Debug + Clone + PartialEq,
{
    let mut output = [0u8; 4];
    f.read_exact(&mut output)?;
    Ok(output)
}

fn take_8<F, D>(f: &mut F, _d: &mut ParseData<D>) -> ParseResult<[u8; 8]>
where
    F: Read + Seek,
    D: Debug + Clone + PartialEq,
{
    let mut output = [0u8; 8];
    f.read_exact(&mut output)?;
    Ok(output)
}

// TODO: should this take a template for what error it returns.. that would complicate things
// TODO: It would be nice to allow non-Seek types. At the very least, forward seek can be
//   ''implemented'' by reading data and throwing it away.
//     Most parsing probably doesn't need arbitrarty seeking.
pub trait Parse<D>: Sized
where
    D: Debug + Clone + PartialEq,
{
    fn parse<F>(f: &mut F, d: &mut ParseData<D>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek;
}

impl Parse<()> for u8 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = single(f, d)?;
        Ok(match d.endian {
            Endian::Big => u8::from_be_bytes([data]),
            Endian::Little => u8::from_le_bytes([data]),
        })
    }
}

impl Parse<()> for i8 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = single(f, d)?;
        Ok(match d.endian {
            Endian::Big => i8::from_be_bytes([data]),
            Endian::Little => i8::from_le_bytes([data]),
        })
    }
}
impl Parse<()> for u16 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_2(f, d)?;
        Ok(match d.endian {
            Endian::Big => u16::from_be_bytes(data),
            Endian::Little => u16::from_le_bytes(data),
        })
    }
}
impl Parse<()> for i16 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_2(f, d)?;
        Ok(match d.endian {
            Endian::Big => i16::from_be_bytes(data),
            Endian::Little => i16::from_le_bytes(data),
        })
    }
}
impl Parse<()> for u32 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_4(f, d)?;
        Ok(match d.endian {
            Endian::Big => u32::from_be_bytes(data),
            Endian::Little => u32::from_le_bytes(data),
        })
    }
}
impl Parse<()> for i32 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_4(f, d)?;
        Ok(match d.endian {
            Endian::Big => i32::from_be_bytes(data),
            Endian::Little => i32::from_le_bytes(data),
        })
    }
}
impl Parse<()> for u64 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_8(f, d)?;
        Ok(match d.endian {
            Endian::Big => u64::from_be_bytes(data),
            Endian::Little => u64::from_le_bytes(data),
        })
    }
}
impl Parse<()> for i64 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_8(f, d)?;
        Ok(match d.endian {
            Endian::Big => i64::from_be_bytes(data),
            Endian::Little => i64::from_le_bytes(data),
        })
    }
}
impl Parse<()> for f32 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_4(f, d)?;
        Ok(match d.endian {
            Endian::Big => f32::from_be_bytes(data),
            Endian::Little => f32::from_le_bytes(data),
        })
    }
}
impl Parse<()> for f64 {
    fn parse<F>(f: &mut F, d: &mut ParseData<()>) -> ParseResult<Self>
    where
        F: std::io::Read + std::io::Seek,
    {
        let data = take_8(f, d)?;
        Ok(match d.endian {
            Endian::Big => f64::from_be_bytes(data),
            Endian::Little => f64::from_le_bytes(data),
        })
    }
}

#[macro_export]
macro_rules! impl_parse_field {
    ($name:ident : l : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse(
            $input,
            &mut $crate::parse::ParseData::clear($crate::Endian::Little),
        )?;
    };
    ($name:ident : b : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse(
            $input,
            &mut $crate::parse::ParseData::clear($crate::Endian::Big),
        )?;
    };
    ($name:ident : u : $typ:ty; $input:expr) => {
        let $name = <$typ>::parse(
            $input,
            &mut $crate::parse::ParseData::clear($crate::Endian::Big),
        )?;
    };
}

#[macro_export]
macro_rules! impl_parse {
    ($on:ty, [$($name:ident : $e:ident : $typ:ty),*]) => {
        impl $crate::parse::Parse<()> for $on {
            fn parse<F>(f: &mut F, _d: &mut $crate::parse::ParseData<()>) -> $crate::parse::ParseResult<Self>
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
        impl $crate::parse::Parse<()> for $on {
            fn parse<F>(f: &mut F, _d: &mut $crate::parse::ParseData<()>) -> $crate::parse::ParseResult<Self>
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
    const PDATA: ParseData<()> = ParseData::from_empty(Endian::Little);

    #[test]
    fn test_single() {
        let mut cursor = Cursor::new(&DATA);
        assert_eq!(single(&mut cursor, &mut PDATA).unwrap(), 0x1);
        assert_eq!(single(&mut cursor, &mut PDATA).unwrap(), 0x2);
        assert_eq!(single(&mut cursor, &mut PDATA).unwrap(), 0x3);
        assert_eq!(single(&mut cursor, &mut PDATA).unwrap(), 0x4);
    }

    #[test]
    fn test_take() {
        let mut cursor = Cursor::new(&DATA);
        assert_eq!(
            take(&mut cursor, &mut PDATA, 4).unwrap().as_slice(),
            &[0x1, 0x2, 0x3, 0x4]
        );
        assert_eq!(
            take(&mut cursor, &mut PDATA, 4).unwrap().as_slice(),
            &[0x5, 0x6, 0x7, 0x8]
        );
    }

    #[test]
    fn test_tag() {
        let mut cursor = Cursor::new(&DATA);
        tag(&mut cursor, &mut PDATA, &[0x1, 0x2, 0x3, 0x4]).unwrap();
        tag(&mut cursor, &mut PDATA, &[0x5, 0x6, 0x7, 0x8]).unwrap();

        tag(&mut cursor, &mut PDATA, &[0x20, 0x52])
            .expect_err("Expected error since invalid bytes!");
    }

    #[test]
    fn test_many() {
        let mut cursor = Cursor::new(&DATA);
        let result = many(&mut cursor, &mut PDATA, |f, d| -> ParseResult<[u8; 4]> {
            let data = take(f, d, 4)?;
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
        many(&mut cursor, &mut PDATA, |f, d| -> ParseResult<[u8; 3]> {
            let data = take(f, d, 3)?;
            Ok([data[0], data[1], data[2]])
        })
        .expect_err("Expected failure in dividing DATA into 3-byte chunks");
    }
}
