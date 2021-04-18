use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::io::{Read, Write};
use std::marker::PhantomData;

pub struct ExtSortedIterator<T, R, W> {
  tips: BinaryHeap<Reverse<(T, usize)>>,
  readers: Vec<R>,
  failed: bool,
  phantom: PhantomData<W>,
}

impl<T, R, W> ExtSortedIterator<T, R, W> {
  pub(crate) fn new(
    tips: BinaryHeap<Reverse<(T, usize)>>,
    readers: Vec<R>,
  ) -> Self {
    Self {
      tips,
      readers,
      failed: false,
      phantom: PhantomData,
    }
  }
}

impl<T, R, W> Iterator for ExtSortedIterator<T, R, W>
where
    T: super::ExternallySortable<W, R>,
    R: Read,
    W: Write,
{
    type Item = Result<T, T::Error>;

    ///
    /// # Errors
    ///
    /// This method can fail due to issues reading intermediate sorted chunks
    /// from disk, or due to deserialization issues
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

