//! An efficient external sort implementation.
//!
//! You start by implementing the [`Sortable`] for your data, and provide your data via an
//! iterable. Then you create an [`ExtSorter`] to sort data.
//!
//! An example is provided in the `examples/` directory.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, self};
use std::marker::PhantomData;
use std::path::Path;

use tempfile::{Builder, TempDir};

mod iter;

pub use iter::{ExtSortedIterator, Sortable};

/// Sort the data
pub struct ExtSorter<T> {
  buffer_n_items: usize,
  tmp_dir: TempDir,
  phantom: PhantomData<T>,
}

impl<T> ExtSorter<T>
where
    T: Sortable<BufWriter<File>, BufReader<File>>,
    T::Error: From<io::Error>,
{
  /// Create an `ExtSorter` to sort your data.
  ///
  /// It will buffer `buffer_n_items` items in memory and sort them, and then write them serialized
  /// into temporary files.
  pub fn new(buffer_n_items: usize) -> io::Result<Self> {
    Ok(ExtSorter {
      buffer_n_items,
      tmp_dir: tempdir(None)?,
      phantom: PhantomData,
    })
  }

  /// Same as [`new`](fn@Self::new) but provide a directory to store temporary files instead of system default.
  pub fn new_in<P: AsRef<Path>>(
    buffer_n_items: usize, tmp_dir: P,
  ) -> io::Result<Self> {
    Ok(ExtSorter {
      buffer_n_items,
      tmp_dir: tempdir(Some(tmp_dir.as_ref()))?,
      phantom: PhantomData,
    })
  }

  /// Sort the data.
  ///
  /// It returns an iterator to produce sorted results. The sort is unstable.
  pub fn sort<I>(
    &self, unsorted: I,
  ) -> Result<iter::ExtSortedIterator<T, BufReader<File>, BufWriter<File>>, T::Error>
  where
      I: Iterator<Item = T>,
  {
    let mut chunk_count = 0;

    {
      let mut current_count = 0;
      let mut chunk = Vec::new();

      // make the initial chunks on disk
      for seq in unsorted {
        current_count += 1;
        chunk.push(seq);
        if current_count >= self.buffer_n_items {
          chunk.sort_unstable();
          self.write_chunk(
            &self.tmp_dir.path().join(chunk_count.to_string()),
            &mut chunk,
          )?;
          chunk.clear();
          current_count = 0;
          chunk_count += 1;
        }
      }
      // write the last chunk
      if !chunk.is_empty() {
        chunk.sort_unstable();
        self.write_chunk(
          &self.tmp_dir.path().join(chunk_count.to_string()),
          &mut chunk,
        )?;
        chunk_count += 1;
      }
    }

    let readers = (0..chunk_count).map(|i| 
      File::open(self.tmp_dir.path().join(i.to_string())).map(BufReader::new)
    ).collect::<Result<Vec<_>, _>>()?;
    iter::ExtSortedIterator::new(readers)
  }

  fn write_chunk(&self, file: &Path, chunk: &mut Vec<T>) -> Result<(), T::Error> {
    let new_file = OpenOptions::new().create(true).write(true).truncate(true).open(file)?;
    let mut w = BufWriter::new(new_file);
    for s in chunk {
      s.serialize(&mut w)?;
    }

    Ok(())
  }
}

fn tempdir(path: Option<&Path>) -> io::Result<TempDir> {
  let mut builder = Builder::new();
  builder.prefix("extsort_lily");

  if let Some(p) = path {
    builder.tempdir_in(p)
  } else {
    builder.tempdir()
  }
}
