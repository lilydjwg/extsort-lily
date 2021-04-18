use std::error::Error;
use std::io::{Read, Write};

use extsort::{ExternalSorter, ExternallySortable};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Num {
  the_num: u8,
}

impl Num {
  fn new(num: u8) -> Num {
    Num { the_num: num }
  }
}

impl<W, R> ExternallySortable<W, R> for Num
  where W: Write,
        R: Read,
{
  fn serialize(&self, w: &mut W) -> Result<(), Box<dyn Error>> {
    match w.write(&[self.the_num]) {
      Ok(_) => Ok(()),
      Err(e) => Err(Box::new(e)),
    }
  }

  fn deserialize(r: &mut R) -> Option<Result<Self, Box<dyn Error>>> {
    let mut buf = [0];
    match r.read(&mut buf) {
      Ok(0) => None,
      Ok(1) => Some(Ok(Self::new(buf[0]))),
      Err(e) => Some(Err(Box::new(e))),
      _ => unreachable!(),
    }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let unsorted = vec![
    Num::new(5),
    Num::new(2),
    Num::new(1),
    Num::new(3),
    Num::new(4),
  ];
  let sorted = vec![
    Num::new(1),
    Num::new(2),
    Num::new(3),
    Num::new(4),
    Num::new(5),
  ];
  let iter = ExternalSorter::new(2)?
    .sort(unsorted.into_iter()).unwrap();
  for (idx, i) in iter.enumerate() {
    assert_eq!(i?.the_num, sorted[idx].the_num);
  }

  Ok(())
}
