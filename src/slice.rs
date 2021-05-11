use crate::stream_position;
use std::{
    io::{self, Read, Seek, SeekFrom},
    ops::{Bound, RangeBounds, RangeInclusive},
};

/// This was created because Take doesn't support Seek
#[derive(Debug)]
pub struct InputSlice<F: Read> {
    input: F,
    range: RangeInclusive<u64>,
}
impl<F> InputSlice<F>
where
    F: Read,
{
    /// Creates `InputSlice` instance withprovided [range] and [range]
    /// Panics if the input is unsound, by getting current position
    /// and checking if it is within [range].
    /// Does not modify current position.
    #[inline]
    pub fn new<R>(mut input: F, range: R) -> std::io::Result<Self>
    where
        R: RangeBounds<u64>,
        F: Seek,
    {
        let position = input.seek(SeekFrom::Current(0))?;
        assert!(range.contains(&position));
        Ok(Self::new_unchecked(input, range))
    }

    /// Creates a new `InputSlice` instance with the provided range and file.
    /// Unlike `InputSlice::new` this does not check if the [input] is sound.
    /// # Soundness: Requires
    ///  `range.start() <= input.seek(SeekFrom::Current(0)) <= range.end()`
    #[inline]
    pub fn new_unchecked<R>(input: F, range: R) -> Self
    where
        R: RangeBounds<u64>,
    {
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x + 1,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => u64::MAX,
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x - 1,
        };
        let range = RangeInclusive::new(start, end);
        InputSlice { input, range }
    }

    /// Creates an InputSlice at current position, for [amount] bytes
    /// uses stream_len
    /// to get the current position.
    /// Note: `[current position] + amount` performs saturating addition
    #[inline]
    pub fn at(mut input: F, amount: u64) -> std::io::Result<Self>
    where
        F: Seek,
    {
        let start = input.seek(SeekFrom::Current(0))?;
        let end = start.saturating_add(amount);
        Ok(Self::new_unchecked(input, start..=end))
    }

    /// Returns inclusive start
    #[inline]
    pub fn start(&self) -> u64 {
        *self.range.start()
    }

    /// Returns inclusive end
    #[inline]
    pub fn last(&self) -> u64 {
        *self.range.end()
    }

    /// Returns exclusive end
    #[inline]
    pub fn end(&self) -> u64 {
        *self.range.end() + 1
    }

    #[inline]
    pub fn contains(&self, position: u64) -> bool {
        self.range.contains(&position)
    }

    #[inline]
    pub fn range(&self) -> &RangeInclusive<u64> {
        &self.range
    }

    #[inline]
    pub fn into_inner(self) -> F {
        self.input
    }

    #[inline]
    pub fn get_ref(&self) -> &F {
        &self.input
    }

    /// Note: one should be careful with this handle, as that might invalidate
    #[inline]
    pub fn get_mut(&mut self) -> &mut F {
        &mut self.input
    }

    // TODO: Once `Seek::stream_position` is stabilized, use that instead.
    // TODO: don't assume that we can subtract these two values safely
    /// Note: returns the position within this slice, rather than in the containing input as a whole
    #[inline]
    pub fn stream_position(&mut self) -> std::io::Result<u64>
    where
        F: Seek,
    {
        Ok(self.absolute_stream_position()? - self.start())
    }

    #[inline]
    pub fn absolute_stream_position(&mut self) -> std::io::Result<u64>
    where
        F: Seek,
    {
        // Have to use it on input otherwise we get infinite-recursion due to `seek` using
        // self.stream_position internally!
        stream_position(&mut self.input)
    }

    // TODO: use `crate::stream_len`
    // TODO: once `Seek::stream_len` is stabilized, use that instead.
    // We will still need to wrap around the stabilzied function
    #[inline]
    pub fn stream_len(&mut self) -> std::io::Result<u64>
    where
        F: Seek,
    {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::Start(self.end()))?;

        // Avoid seeking a third time if we're already at the end
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        // TODO: check this calculation
        let len = len.min(self.end() - self.start());

        Ok(len)
    }

    #[inline]
    pub fn get_distance_from_end(&self, position: u64) -> u64 {
        self.end() - position
    }

    #[inline]
    pub fn position_at_end(&mut self) -> std::io::Result<bool>
    where
        F: Seek,
    {
        let position = self.absolute_stream_position()?;
        Ok(position == self.end())
    }

    //    pub fn into_inner
    //
}
impl<F> Read for InputSlice<F>
where
    F: Read + Seek,
{
    /// Calls [input]'s Read::read method.
    /// If the
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let current_position = self.stream_position()?;
        if self.position_at_end()? {
            return Ok(0);
        }

        let max = std::cmp::min(
            buf.len() as u64,
            self.get_distance_from_end(current_position),
        ) as usize;
        let buf = &mut buf[..max];
        let amount_read = self.input.read(buf)?;
        debug_assert!(self.stream_position()? <= self.end());
        Ok(amount_read)
    }

    // TODO: it might be more efficient write wrappers around every <F as Read> method?
}
impl<F> Seek for InputSlice<F>
where
    F: Read + Seek,
{
    /// If you seek beyond the end, behavior is to constrain you to `*self.range.end()`
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        // Get position and offset
        let (base_pos, offset) = match pos {
            SeekFrom::Start(pos) => (pos, 0),
            SeekFrom::Current(off) => (self.stream_position()?, off),
            SeekFrom::End(off) => (self.end(), off),
        };

        // Add the position and offset, properly handling negatives
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u64)
        } else {
            base_pos.checked_sub(offset.wrapping_neg() as u64)
        }
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )
        })?;

        // Add the start position so we get the absolute position within the file.
        let new_pos = new_pos.checked_add(self.start()).ok_or_else(|| {
            std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "invald seek to overflowing position when added to base",
            )
        })?;

        // Clamp to new_pos
        let new_pos = if self.contains(new_pos) {
            new_pos
        } else {
            self.end()
        };

        // TODO: this shouldn't use a different type of SeekFrom as some
        //  Seek impls may not support it.

        self.input
            .seek(SeekFrom::Start(new_pos))
            // Subtract the start offset, so that the returned 'new position' is valid for our range
            // TODO: test this!
            // TODO: this should probably be a checked subtraction!
            .map(|x| x - self.start())
    }
}

#[cfg(test)]
mod tests {
    use super::InputSlice;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    #[test]
    fn test_general() {
        let input = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
        ];
        let input_length = input.len() as u64;
        println!("Data length: {}", input_length);
        let cursor = Cursor::new(input);
        // This should be 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 ,16
        // so [0, 16], or as written: [0, 17)
        let mut slice = InputSlice::new(cursor, 0..input_length).unwrap();
        println!("Slice range: [{}, {}]", slice.start(), slice.end());
        assert_eq!(slice.stream_position().unwrap(), 0);
        assert_eq!(slice.stream_len().unwrap(), 17);

        let mut data = [0u8; 4];
        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [0, 1, 2, 3]);
        assert_eq!(slice.stream_position().unwrap(), 4);

        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [4, 5, 6, 7]);
        assert_eq!(slice.stream_position().unwrap(), 8);

        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [8, 9, 0xa, 0xb]);
        assert_eq!(slice.stream_position().unwrap(), 12);

        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [0xc, 0xd, 0xe, 0xf]);
        assert_eq!(slice.stream_position().unwrap(), 16);

        slice
            .read_exact(&mut data)
            .expect_err("Expected error when reading last byte!");

        assert_eq!(slice.seek(SeekFrom::Start(13)).unwrap(), 13);
        assert_eq!(slice.stream_position().unwrap(), 13);
        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [0xd, 0xe, 0xf, 0x10]);
        assert_eq!(slice.stream_position().unwrap(), input_length);

        let data = slice.into_inner().into_inner();
        let mut cursor = Cursor::new(data);

        // TODO: it would good to do this test
        // InputSlice::new(cursor, 3..length)
        //     .expect_err("Expected error to okay since not within slice range.");

        assert_eq!(cursor.seek(SeekFrom::Start(3)).unwrap(), 3);
        let mut slice = InputSlice::new(cursor, 3..input_length).unwrap();
        println!("Slice range: [{}, {}]", slice.start(), slice.end());
        assert_eq!(slice.stream_position().unwrap(), 0);
        assert_eq!(slice.absolute_stream_position().unwrap(), 3);
        assert_eq!(slice.stream_len().unwrap(), 14);

        let mut data = [0u8; 5];
        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [3, 4, 5, 6, 7]);
        assert_eq!(slice.stream_position().unwrap(), 5);

        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [8, 9, 0xa, 0xb, 0xc]);
        assert_eq!(slice.stream_position().unwrap(), 10);

        slice
            .read_exact(&mut data)
            .expect_err("Expected to fail reading bytes since there was not five more");

        // because it doesn't assure our position after error
        assert_eq!(slice.seek(SeekFrom::Start(10)).unwrap(), 10);
        let mut data = [0u8; 4];
        slice.read_exact(&mut data).unwrap();
        assert_eq!(data, [0xd, 0xe, 0xf, 0x10]);
        assert_eq!(slice.stream_position().unwrap(), 14);

        assert_eq!(slice.stream_len().unwrap(), 14);

        // TODO: this needs more tests. Like I was able to read multiple bytes past the end
        //  when I only accidently had 1 extra byte..
    }
}
