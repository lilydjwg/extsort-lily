use std::io::{Read, Write, Error};

use extsort_lily::{ExternalSorter, Sortable};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Num {
  the_num: u8,
}

impl Num {
  fn new(num: u8) -> Num {
    Num { the_num: num }
  }
}

impl<W, R> Sortable<W, R> for Num
  where W: Write,
        R: Read,
{
  type Error = Error;

  fn serialize(&self, w: &mut W) -> Result<(), Error> {
    w.write(&[self.the_num]).map(|_| ())
  }

  fn deserialize(r: &mut R) -> Option<Result<Self, Error>> {
    let mut buf = [0];
    match r.read(&mut buf) {
      Ok(0) => None,
      Ok(1) => Some(Ok(Self::new(buf[0]))),
      Err(e) => Some(Err(e)),
      _ => unreachable!(),
    }
  }
}

fn main() -> Result<(), Error> {
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
