use std::{
    fmt::Result,
    io::{ErrorKind, Read, Seek, SeekFrom},
};

pub mod data_size;
pub mod parse;
pub mod slice;
pub mod writable;
pub mod zstring;
pub use bstr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EnumConversionError<V> {
    /// Invalid value.
    InvalidValue(V),
}

// TODO: once const generics come around, we can use this as a template parameter instead?
// Similar to byteorder
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

#[inline]
pub fn stream_position<F>(f: &mut F) -> std::io::Result<u64>
where
    F: Seek,
{
    f.seek(SeekFrom::Current(0))
}

// TODO: once `Seek::stream_len` is stabilized, replace this with it.
#[inline]
pub fn stream_len<F>(f: &mut F) -> std::io::Result<u64>
where
    F: Seek,
{
    let old_pos = stream_position(f)?;
    let len = f.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time if we're already at the end
    if old_pos != len {
        f.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

// TODO: should this have a sub-module?
/// implements function that returns if that bit is set:
/// `impl_flags!(LinkFlags, flags, [thing1 : 0b1, thing2: 0b10]);`
/// implements function that shift-lefts (1 << n) then returns if that bit set:
/// `impl_flags(shl, LinkFlags, flags, [thing1: 0, thing2: 1]);`
#[macro_export]
macro_rules! impl_flags {
    ($strct:ty, $field:ident, [$($(#[$outer:meta])* $name:ident : $bits:expr),*]) => {
        impl $strct {
            $(
                $(#[$outer])*
                pub fn $name(&self) -> bool {
                    (self.$field & $bits) != 0
                }
            )*
        }
    };

    (shl $strct:ty, $field:ident, [$($(#[$outer:meta])* $name:ident : $l:expr),*]) => {
        impl $strct {
            $(
                $(#[$outer])*
                pub fn $name(&self) -> bool {
                    (self.$field & (1 << $l)) != 0
                }
            )*
        }
    };
}

/// Skip `amount` bytes. This is for when you don't implement seek.
/// TODO: This is really shoddy, and specialization would make this way better.
#[inline]
pub fn skip<F: Read, const CHUNK: usize>(mut f: F, mut amount: usize) -> std::io::Result<()> {
    let mut buf = [0_u8; CHUNK];

    loop {
        if amount == 0 {
            break;
        }

        let end = CHUNK.min(amount);
        let buf_slice: &mut [u8] = &mut buf[..end];
        f.read_exact(buf_slice)?;

        amount = amount.saturating_sub(CHUNK);
    }

    Ok(())
}

/// Reads to fill the buffer if it can.
/// Values up to the returned Ok(usize) are valid
/// If there was an error then no assurances are made.
///
/// Currently, Rust has `read` and `read_exact`, which work pretty well in many cases
/// But if you want to only read some bit of data that may not fit your buffer, then
/// you'd have to use `read`.
/// The problem with read is that it doesn't have to read all the data it can on the first go
/// So this function tries to finish this trifecta by allowing the reading of 'as much as we can' or
/// until we hit the end of the buffer.
#[inline]
pub fn read_if_possible<F: Read>(mut f: F, mut buf: &mut [u8]) -> std::io::Result<usize> {
    let mut amount_read: usize = 0;
    while !buf.is_empty() {
        match f.read(buf) {
            // We got no data, so we assume that we're done here.
            Ok(0) => break,
            // TODO: Is there a better way of handling this than a saturating add?
            Ok(c) => {
                amount_read = amount_read.saturating_add(c);
                // Reassign the buffer
                buf = &mut buf[c..];
            }
            // We were interrupted in reading, so we ignore it and retry.
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        };
    }

    // We don't bother to check if it wasn't empty.
    Ok(amount_read)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    const DATA: [u8; 16] = [
        0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
    ];

    // TODO: these are very specific tests just for cursor, rather than testing various types

    #[test]
    pub fn test_stream_position() {
        let mut cursor = std::io::Cursor::new(&DATA);
        assert_eq!(stream_position(&mut cursor).unwrap(), 0);
        let mut out = [0u8; 4];
        cursor.read_exact(&mut out).unwrap();
        assert_eq!(stream_position(&mut cursor).unwrap(), 4);
        assert_eq!(cursor.seek(SeekFrom::End(0)).unwrap(), DATA.len() as u64);
        assert_eq!(stream_position(&mut cursor).unwrap(), DATA.len() as u64);
    }

    #[test]
    pub fn test_stream_len() {
        let mut cursor = std::io::Cursor::new(&DATA);
        assert_eq!(stream_len(&mut cursor).unwrap(), DATA.len() as u64);
        let mut out = [0u8; 4];
        cursor.read_exact(&mut out).unwrap();
        assert_eq!(stream_len(&mut cursor).unwrap(), DATA.len() as u64);
    }

    #[test]
    pub fn test_skip() {
        let mut cursor = std::io::Cursor::new(&DATA as &[u8]);
        skip::<_, 16>(&mut cursor, 1).unwrap();
        assert_eq!(cursor.position(), 1);
        skip::<_, 16>(&mut cursor, 1).unwrap();
        assert_eq!(cursor.position(), 2);
        skip::<_, 16>(&mut cursor, 4).unwrap();
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    pub fn test_read_if_possible() {
        let mut cursor = std::io::Cursor::new(&DATA);
        let mut buf = [0; 5];
        assert_eq!(read_if_possible(&mut cursor, &mut buf).unwrap(), 5);
        assert_eq!(buf, [0x1, 0x2, 0x3, 0x4, 0x5]);

        assert_eq!(read_if_possible(&mut cursor, &mut buf).unwrap(), 5);
        assert_eq!(buf, [0x6, 0x7, 0x8, 0x9, 0xa]);

        assert_eq!(read_if_possible(&mut cursor, &mut buf).unwrap(), 5);
        assert_eq!(buf, [0xb, 0xc, 0xd, 0xe, 0xf]);

        assert_eq!(read_if_possible(&mut cursor, &mut buf).unwrap(), 1);
        assert_eq!(buf[0], 0x10);
    }
}
