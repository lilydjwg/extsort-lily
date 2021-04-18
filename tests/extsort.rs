use std::io::{Read, Write, Error};

use rand;

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

#[test]
fn sort() {
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
  let iter = ExternalSorter::new(2).unwrap()
    .sort(unsorted.into_iter())
    .unwrap();
    for (idx, i) in iter.enumerate() {
      assert_eq!(i.unwrap().the_num, sorted[idx].the_num);
    }
}

#[test]
fn zero_buff() {
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
  let iter = ExternalSorter::new(0).unwrap()
    .sort(unsorted.into_iter())
    .unwrap();
    for (idx, i) in iter.enumerate() {
      assert_eq!(i.unwrap().the_num, sorted[idx].the_num);
    }
}

#[test]
fn large_buff() {
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
  let iter = ExternalSorter::new(999999999).unwrap()
    .sort(unsorted.into_iter())
    .unwrap();
    for (idx, i) in iter.enumerate() {
      assert_eq!(i.unwrap().the_num, sorted[idx].the_num);
    }
}

#[test]
fn reuse() {
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
  let iter = ExternalSorter::new(2).unwrap()
    .sort(unsorted.clone().into_iter())
    .unwrap();
    for (idx, i) in iter.enumerate() {
      assert_eq!(i.unwrap().the_num, sorted[idx].the_num);
    }
    let iter2 = ExternalSorter::new(2).unwrap()
      .sort(unsorted.into_iter())
      .unwrap();
    for (idx, i) in iter2.enumerate() {
      assert_eq!(i.unwrap().the_num, sorted[idx].the_num);
    }
}

#[test]
fn large() {
  let mut unsorted = Vec::new();
  for _ in 0..10_000 {
    unsorted.push(Num::new(rand::random()));
  }
  let iter = ExternalSorter::new(100).unwrap()
    .sort(unsorted.into_iter())
    .unwrap();
    let mut last = 0;
    for i in iter {
      let n = i.unwrap().the_num;
      assert!(n >= last);
      last = n;
    }
}
