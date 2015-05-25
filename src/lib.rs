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
  fn drop(&mut self) {
    let string = self.string.take().unwrap();
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

  pub fn shared_reference(&self) -> Rc<RefCell<Vec<String>>> {
    self.strings.clone()
  }
 
  pub fn new(&mut self) -> String {//RecycledString {
    self.new_from("")
  }
  
  pub fn new_from(&mut self, source: &str) -> String {
    let string = match self.strings.borrow_mut().pop() {
      Some(mut s) => {
        debug!("Pulling from pool, size now: {}", self.size());
        s.push_str(source);
        //RecycledString::new_from(&self.sender, s)
        s
      },
      None => {
        debug!("Pool empty, creating a new string.");
        source.to_owned()
        //RecycledString::new_from(&self.sender, source.to_owned())
      }
    };
    string
  }

  pub fn size(&self) -> usize {
    self.strings.borrow().len()
  }
}

macro_rules! pooled {
  ($pool:ident, $value:expr) => {
    {
      let string = $pool.new_from($value);
      let pool_reference = $pool.shared_reference();
      RecycledString::new_from(pool_reference, string)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;
  use env_logger;

  #[bench]
  fn normal_allocation_speed(b: &mut Bencher) {
    b.iter(|| {
      for _ in 0..10_000 {
        let _string = "man".to_string();
        let _string = "dog".to_string();
        let _string = "cat".to_string();
        let _string = "mouse".to_string();
        let _string = "cheese".to_string();
      }
    });
  }

  #[bench]
  fn recycle_allocation_speed(b: &mut Bencher) {
    let _ = env_logger::init();
    let mut pool = StringPool::with_size(5);
    b.iter(|| {
      for _ in 0..10_000 {
        let _string = pooled!(pool, "man");
        let _string = pooled!(pool, "dog");
        let _string = pooled!(pool, "cat");
        let _string = pooled!(pool, "mouse");
        let _string = pooled!(pool, "cheese");
      }
    });
  }
}
