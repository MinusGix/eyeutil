use std::io::{Seek, SeekFrom};

pub mod data_size;
pub mod parse;
pub mod slice;
pub mod writable;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EnumConversionError<V> {
    /// Invalid value.
    InvalidEnumerationValue(V),
}

// TODO: once const generics come around, we can use this as a template parameter instead?
// Similar to byteorder
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

pub fn stream_position<F>(f: &mut F) -> std::io::Result<u64>
where
    F: Seek,
{
    f.seek(SeekFrom::Current(0))
}

// TODO: once `Seek::stream_len` is stabilized, replace this with it.
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
}
