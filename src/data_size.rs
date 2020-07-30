use std::fmt::Debug;

pub trait DataSize<D>
where
    D: Debug + Clone + PartialEq,
{
    /// Returns data size in bytes
    fn data_size(&self, d: D) -> u64;
}

// TODO: I would really like to have a StaticDataSize trait for types which don't need
//  to do any calculation of their size, and can be known at compile time
//  but I was unable to make it
// Issue:
// `impl<D, T> DataSize<D> for T where D: Debug + Clone + PartialEq, T: StaticDataSize<D>`
// would work fine
// but then when you wanted to do
// `impl<D, T> DataSize<D> for Vec<T> where D: Debug + Clone + PartialEq, T: StaticDataSize`
// which would cause errors due to conflicting implementation...
// it works fine if D is not generic, for some unholy reason.
// I even tried the very unstable 'specialization' feature and could not get it to work.
// Sigh.

// TODO: we could have a version which takes in data as a given variable.
/// usage:
/// `impl_data_size!(u32, 4);`
#[macro_export]
macro_rules! impl_data_size {
    ($typ:ty, $value:expr) => {
        impl $crate::data_size::DataSize<()> for $typ {
            fn data_size(&self, _d: ()) -> u64 {
                $value
            }
        }
    };
}

impl_data_size!((), 1);
impl_data_size!(u8, 1);
impl_data_size!(i8, 1);
impl_data_size!(u16, 2);
impl_data_size!(i16, 2);
impl_data_size!(u32, 4);
impl_data_size!(i32, 4);
impl_data_size!(u64, 8);
impl_data_size!(i64, 8);
impl_data_size!(u128, 16);
impl_data_size!(i128, 16);
impl_data_size!(f32, 4);
impl_data_size!(f64, 8);
impl<D, T> DataSize<D> for &[T]
where
    D: Debug + Clone + PartialEq,
    T: DataSize<D>,
{
    fn data_size(&self, d: D) -> u64 {
        self.iter()
            .fold(0u64, |acc, x| acc + x.data_size(d.clone()))
    }
}
impl<D, T> DataSize<D> for Vec<T>
where
    D: Debug + Clone + PartialEq,
    T: DataSize<D>,
{
    fn data_size(&self, d: D) -> u64 {
        self.as_slice().data_size(d)
    }
}
