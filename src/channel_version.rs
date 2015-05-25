#![feature(test)]
#![allow(dead_code)]
extern crate test;
#[macro_use]
extern crate log;
extern crate env_logger;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::fmt;
use std::ops::Drop;

pub struct RecycledString<'pool> {
  pub string: Option<String>,
  pool_sender: &'pool Sender<String>
}

impl <'pool> Drop for RecycledString<'pool> {
  fn drop(&mut self) {
    let string = self.string.take().unwrap();
    let _ = self.pool_sender.send(string); // TODO: Handle errors
  }
}

impl <'pool> fmt::Display for RecycledString<'pool> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.string {
      Some(ref s) => write!(f, "RecycledString('{}')", s),
      None => write!(f, "RecycledString('<data missing>')")
    }
  }
}

impl <'pool> RecycledString <'pool> {
  pub fn new(pool_sender: &'pool Sender<String>) -> RecycledString<'pool> {
    RecycledString {
      string: Some(String::new()),
      pool_sender: pool_sender
    }
  }
  
  pub fn new_from(pool_sender: &'pool Sender<String>, source_string: String) -> RecycledString<'pool> {
    RecycledString {
      string: Some(source_string),
      pool_sender: pool_sender
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
  strings: Vec<String>,
  reclaimed_strings: Receiver<String>,
  pub sender: Sender<String>
}

impl StringPool {

  pub fn sender<'a> (&'a self) -> &'a Sender<String>{
    &self.sender
  }

  pub fn with_size(size: u32) -> StringPool {
    let (sender, receiver) = channel();
    let strings: Vec<String> = 
      (0..size)
      .map(|_| String::new() )
      .collect();
    StringPool {
      strings: strings,
      reclaimed_strings: receiver,
      sender: sender
    }
  }
  
  fn reclaim_strings(&mut self) {
    debug!("Attempting to reclaim strings");
    loop {
      match self.reclaimed_strings.try_recv() {
        Ok(mut string) => {
          debug!("Reclaiming '{}'", string);
          string.clear();
          self.strings.push(string);
        },
        Err(_) => {
          debug!("No strings left to reclaim.");
          break;
        }
      }
    }
    debug!("Done reclaiming strings.");
  }

  pub fn new(&mut self) -> String {//RecycledString {
    self.new_from("")
  }
  
  pub fn new_from(&mut self, source: &str) -> String {//RecycledString<'pool> {
    self.reclaim_strings();
    let string = match self.strings.pop() {
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
    self.strings.len()
  }
}

macro_rules! pooled {
  ($pool:ident, $value:expr) => {
    {
      let string = $pool.new_from($value);
      let sender = $pool.sender();
      RecycledString::new_from(sender, string)
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
    debug!("Calling recycle!");
    let _ = env_logger::init();
    let mut pool = StringPool::with_size(10);
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
