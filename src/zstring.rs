use crate::{
    parse::{take_until, Parse, ParseResult},
    writable::{Writable, WriteResult},
};
use bstr::BString;
use std::io::{Read, Seek, Write};

/// Simple (ascii-ish, but more a byte-string) Null-terminated string.
/// Is not meant to work on unicode.
/// Does not store null-terminator, but does write it out when requested.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ZString(pub BString);
impl ZString {
    pub const TERMINATOR: u8 = 0x00;

    /// Note: this is without null-terminator!
    /// This also does _not_ check if this contains nulls, which may confuse things if
    /// you aren't sure!
    pub fn new(data: Vec<u8>) -> Self {
        ZString(BString::from(data))
    }

    // TODO: provide various methods that bstring/vec<u8> might provide.

    /// Returns the number of elements.
    /// Note: Does not include null-terminator.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether there is any values.
    /// Note: does not include null-terminator, otherwise it would always be false.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Note: Does not include null-terminator.
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Note: Does not include null-terminator
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}
impl<F: Read + Seek> Parse<F> for ZString {
    fn parse(f: &mut F, _d: ()) -> ParseResult<Self> {
        let data = take_until(f, ZString::TERMINATOR, false)?;

        Ok(ZString::new(data))
    }
}
impl Writable<()> for ZString {
    fn write_to<W>(&self, w: &mut W, _d: ()) -> WriteResult
    where
        W: Write,
    {
        self.0.as_slice().write_to(w, ())?;
        // Write null-terminator due to it not being included in stored string
        ZString::TERMINATOR.write_to(w, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NSTR: &[u8] = b"HELLO\x00";

    #[test]
    fn test_write_back() {
        let mut cursor = std::io::Cursor::new(NSTR);
        let zstring = ZString::parse(&mut cursor, ()).unwrap();
        assert_eq!(zstring.as_slice(), b"HELLO");
        assert_eq!(zstring.len(), 5);
        assert_eq!(zstring.is_empty(), false);

        // Test writing back
        let mut output = [0u8; 6];
        let mut output_cursor = std::io::Cursor::new((&mut output) as &mut [u8]);
        zstring.write_to(&mut output_cursor, ()).unwrap();
        assert_eq!(&output, NSTR);
    }
}
