use crate::Endian;
use std::io::Write;

pub type WriteResult = Result<(), WriteError>;

#[derive(Debug)]
pub enum WriteError {
    Io(std::io::Error),
}
impl From<std::io::Error> for WriteError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WriteData {
    pub endian: Endian,
}
impl WriteData {
    pub fn from_endian(endian: Endian) -> Self {
        Self { endian }
    }
}

// TODO: it'd be nice to support Little|Big endian as a more general crate for my parsing needs
/// NOTE: all Writables will write integers and floats in little endian.
pub trait Writable: Sized {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write;
}

impl Writable for u8 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for i8 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for u16 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for i16 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for u32 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for i32 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for u64 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for i64 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for f32 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable for f64 {
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match d.endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl<T> Writable for &[T]
where
    T: Writable,
{
    /// Note: does not write length for you, that's up to you.
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        for entry in self.iter() {
            entry.write_to(w, d)?;
        }
        Ok(())
    }
}
impl<T> Writable for Vec<T>
where
    T: Writable,
{
    fn write_to<W>(&self, w: &mut W, d: WriteData) -> WriteResult
    where
        W: Write,
    {
        self.as_slice().write_to(w, d)
    }
}

// TODO: add tests
