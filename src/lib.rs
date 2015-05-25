#![feature(test)]
#![allow(dead_code)]
extern crate test;
#[macro_use]
extern crate log;
extern crate env_logger;
//use std::sync::mpsc::{channel, Sender, Receiver};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::ops::Drop;

pub struct RecycledString {
  pub string: Option<String>,
  pool: Rc<RefCell<Vec<String>>>
}

impl Drop for RecycledString {

  #[inline(always)] 
  fn drop(&mut self) {
    let mut string = self.string.take().unwrap();
    string.clear();
    let _ = self.pool.borrow_mut().push(string);
  }
}

impl fmt::Display for RecycledString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.string {
      Some(ref s) => write!(f, "RecycledString('{}')", s),
      None => write!(f, "RecycledString('<data missing>')")
    }
  }
}

impl RecycledString {
  pub fn new(pool: Rc<RefCell<Vec<String>>>) -> RecycledString {
    RecycledString {
      string: Some(String::new()),
      pool: pool
    }
  }
  
  #[inline(always)] 
  pub fn new_from(pool: Rc<RefCell<Vec<String>>>, source_string: String) -> RecycledString {
    RecycledString {
      string: Some(source_string),
      pool: pool
    }
  }

  pub fn clear(&mut self) {
    self.string.as_mut().map(|s| s.clear());
  }

  pub fn len(&self) -> usize {
    self.string.as_ref().map_or(0, |s| s.len())
  }
  
  pub fn push_str(&mut self, source: &str) {
    self.string.as_mut().map(|s| s.push_str(source));
  }
}

pub struct StringPool {
  strings: Rc<RefCell<Vec<String>>>,
}

impl StringPool {

  pub fn with_size(size: u32) -> StringPool {
    let strings: Vec<String> = 
      (0..size)
      .map(|_| String::new() )
      .collect();
    StringPool {
      strings: Rc::new(RefCell::new(strings)),
    }
  }

  pub fn new(&mut self) -> RecycledString {//RecycledString {
    self.new_from("")
  }
 
  #[inline(always)] 
  pub fn new_from(&mut self, source: &str) -> RecycledString {
    let new_reference = self.strings.clone();
    let string = match self.strings.borrow_mut().pop() {
      Some(mut s) => {
        //debug!("Pulling from pool, size now: {}", self.size());
        s.push_str(source);
        RecycledString::new_from(new_reference, s)
      },
      None => {
        //debug!("Pool empty, creating a new string.");
        RecycledString::new_from(new_reference, source.to_owned())
      }
    };
    string
  }

  pub fn size(&self) -> usize {
    self.strings.borrow().len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;
  use env_logger;

  const ITERATIONS : u32 = 1_000_000;

  #[bench]
  fn normal_allocation_speed(b: &mut Bencher) {
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = "man".to_owned();
        let _string = "dog".to_owned();
        let _string = "cat".to_owned();
        let _string = "mouse".to_owned();
        let _string = "cheese".to_owned();
      }
    });
  }

  #[bench]
  fn recycle_allocation_speed(b: &mut Bencher) {
    let _ = env_logger::init();
    let mut pool = StringPool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_from("man");
        let _string = pool.new_from("dog");
        let _string = pool.new_from("cat");
        let _string = pool.new_from("mouse");
        let _string = pool.new_from("cheese");
      }
    });
  }
}
