use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::io::{Read, Write};
use std::marker::PhantomData;

/// Implement this trait for data to be sorted
pub trait Sortable<W: Write, R: Read>: Ord + Sized {

  /// Errors that may occur during serialization and deserialization. Note that it needs to
  /// implement `From<std::io::Error>`.
  type Error;

  /// serialize itself and write into the writer.
  fn serialize(&self, w: &mut W) -> Result<(), Self::Error>;

  /// read data and deserialize.
  ///
  /// If no more items to be read, return `None`.
  fn deserialize(r: &mut R) -> Option<Result<Self, Self::Error>>;
}

/// The iterator type for producing results sorted by merge sort.
///
/// It produces `Result<T, T::Error>`s.
///
/// It can fail due to issues reading intermediate sorted chunks from disk, or due to
/// deserialization issues.
///
/// The writer `W` is not actually used by this struct.
pub struct ExtSortedIterator<T, R, W> {
  tips: BinaryHeap<Reverse<(T, usize)>>,
  readers: Vec<R>,
  failed: bool,
  phantom: PhantomData<W>,
}

impl<T, R, W> ExtSortedIterator<T, R, W>
  where T: Sortable<W, R>,
        W: Write, R: Read,
{
  /// do merge sort on `readers`.
  pub fn new(mut readers: Vec<R>) -> Result<Self, T::Error> {
    let mut tips = BinaryHeap::with_capacity(readers.len());
    for (idx, r) in readers.iter_mut().enumerate() {
      let item = T::deserialize(r).unwrap()?;
      tips.push(Reverse((item, idx)));
    }

    Ok(Self {
      tips,
      readers,
      failed: false,
      phantom: PhantomData,
    })
  }
}

impl<T, R, W> Iterator for ExtSortedIterator<T, R, W>
where
    T: Sortable<W, R>,
    R: Read,
    W: Write,
{
    type Item = Result<T, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }

        let Reverse((r, idx)) = self.tips.pop()?;
        match T::deserialize(&mut self.readers[idx]) {
          Some(Ok(n)) => {
            self.tips.push(Reverse((n, idx)));
          },
          Some(Err(e)) => {
            self.failed = true;
            return Some(Err(e));
          },
          None => { },
        };

        Some(Ok(r))
    }
}

