use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, self};
use std::marker::PhantomData;
use std::path::Path;
use std::io::{Read, Write};

use tempdir::TempDir;

mod iter;

pub trait ExternallySortable<W: Write, R: Read>: Ord + Sized {
  fn serialize(&self, w: &mut W) -> Result<(), Box<dyn Error>>;
  fn deserialize(r: &mut R) -> Option<Result<Self, Box<dyn Error>>>;
}

pub struct ExternalSorter<T> {
  buffer_n_items: usize,
  tmp_dir: TempDir,
  phantom: PhantomData<T>,
}

impl<T> ExternalSorter<T>
where
    T: ExternallySortable<BufWriter<File>, BufReader<File>>,
{
  pub fn new(buffer_n_items: usize) -> io::Result<Self> {
    Ok(ExternalSorter {
      buffer_n_items,
      tmp_dir: TempDir::new("extsort")?,
      phantom: PhantomData,
    })
  }

  pub fn new_in<P: AsRef<Path>>(
    buffer_n_items: usize, tmp_dir: P,
  ) -> io::Result<Self> {
    Ok(ExternalSorter {
      buffer_n_items,
      tmp_dir: TempDir::new_in(tmp_dir, "extsort")?,
      phantom: PhantomData,
    })
  }

  pub fn sort<I>(
    &self, unsorted: I,
  ) -> Result<iter::ExtSortedIterator<T, BufReader<File>, BufWriter<File>>, Box<dyn Error>>
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

    let mut readers = (0..chunk_count).map(|i| 
      File::open(self.tmp_dir.path().join(i.to_string())).map(BufReader::new)
    ).collect::<Result<Vec<_>, _>>()?;
    let mut tips = BinaryHeap::with_capacity(chunk_count);
    for (idx, r) in readers.iter_mut().enumerate() {
      let item = T::deserialize(r).unwrap()?;
      tips.push(Reverse((item, idx)));
    }

    Ok(iter::ExtSortedIterator::new(
      tips,
      readers,
    ))
  }

  fn write_chunk(&self, file: &Path, chunk: &mut Vec<T>) -> Result<(), Box<dyn Error>> {
    let new_file = OpenOptions::new().create(true).write(true).truncate(true).open(file)?;
    let mut w = BufWriter::new(new_file);
    for s in chunk {
      s.serialize(&mut w)?;
    }

    Ok(())
  }
}
