use crate::Endian;
use std::{fmt::Debug, io::Write};

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

// TODO: it'd be nice to support Little|Big endian as a more general crate for my parsing needs
/// NOTE: all Writables will write integers and floats in little endian.
pub trait Writable<D>: Sized
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: D) -> WriteResult
    where
        W: Write;
}

impl Writable<()> for u8 {
    fn write_to<W>(&self, w: &mut W, _d: ()) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&self.to_le_bytes())?;
        Ok(())
    }
}
impl Writable<()> for i8 {
    fn write_to<W>(&self, w: &mut W, _d: ()) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&self.to_le_bytes())?;
        Ok(())
    }
}
impl Writable<Endian> for u16 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for i16 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for u32 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for i32 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for u64 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for i64 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for f32 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl Writable<Endian> for f64 {
    fn write_to<W>(&self, w: &mut W, endian: Endian) -> WriteResult
    where
        W: Write,
    {
        w.write_all(&match endian {
            Endian::Big => self.to_be_bytes(),
            Endian::Little => self.to_le_bytes(),
        })?;
        Ok(())
    }
}
impl<D, T> Writable<D> for &[T]
where
    T: Writable<D>,
    D: Debug + Clone + PartialEq,
{
    /// Note: does not write length for you, that's up to you.
    fn write_to<W>(&self, w: &mut W, d: D) -> WriteResult
    where
        W: Write,
    {
        for entry in self.iter() {
            entry.write_to(w, d.clone())?;
        }
        Ok(())
    }
}
impl<D, T> Writable<D> for Vec<T>
where
    T: Writable<D>,
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: D) -> WriteResult
    where
        W: Write,
    {
        self.as_slice().write_to(w, d)
    }
}

// TODO: add tests
