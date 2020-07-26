use std::io::{Seek, SeekFrom};

pub mod slice;
pub mod writable;

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
