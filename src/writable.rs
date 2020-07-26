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

#[derive(Debug, Clone, PartialEq)]
pub struct WriteData<D>
where
    D: Debug + Clone + PartialEq,
{
    pub endian: Endian,
    pub data: D,
}
impl<D> WriteData<D>
where
    D: Debug + Clone + PartialEq,
{
    pub fn from_endian(endian: Endian, data: D) -> Self {
        Self { endian, data }
    }
}
impl WriteData<()> {
    pub fn clear(endian: Endian) -> Self {
        Self { endian, data: () }
    }
}

// TODO: it'd be nice to support Little|Big endian as a more general crate for my parsing needs
/// NOTE: all Writables will write integers and floats in little endian.
pub trait Writable<D>: Sized
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
    where
        W: Write;
}

impl<D> Writable<D> for u8
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for i8
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for u16
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for i16
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for u32
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for i32
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for u64
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for i64
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for f32
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D> Writable<D> for f64
where
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
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
impl<D, T> Writable<D> for &[T]
where
    T: Writable<D>,
    D: Debug + Clone + PartialEq,
{
    /// Note: does not write length for you, that's up to you.
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
    where
        W: Write,
    {
        for entry in self.iter() {
            entry.write_to(w, d)?;
        }
        Ok(())
    }
}
impl<D, T> Writable<D> for Vec<T>
where
    T: Writable<D>,
    D: Debug + Clone + PartialEq,
{
    fn write_to<W>(&self, w: &mut W, d: &mut WriteData<D>) -> WriteResult
    where
        W: Write,
    {
        self.as_slice().write_to(w, d)
    }
}

// TODO: add tests
