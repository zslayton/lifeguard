#![feature(test)]
#![allow(dead_code)]
extern crate test;
#[macro_use]
extern crate log;
extern crate env_logger;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Drop, Deref, DerefMut};
use std::convert::AsRef;

pub trait Recycleable {
  fn new() -> Self;
  fn reset(&mut self);
}

pub trait InitializeWith<T> {
  fn initialize_with(&mut self, source: T);
}

impl Recycleable for String {
  #[inline] 
  fn new() -> String {
    String::new()
  }
  #[inline] 
  fn reset(&mut self) {
    self.clear();
  }
}

impl <T> Recycleable for Vec<T> {
  #[inline] 
  fn new() -> Vec<T> {
    Vec::new()
  }
  #[inline] 
  fn reset(&mut self) {
    self.clear();
  }
}

impl <A> InitializeWith<A> for String where A : AsRef<str> {
  #[inline] 
  fn initialize_with(&mut self, source: A) {
    let s : &str = source.as_ref();
    self.push_str(s);
  }
}

pub struct Recycled<T> where T : Recycleable {
  pub value: Option<T>,
  pool: Rc<RefCell<Vec<T>>>
}

impl <T> Drop for Recycled<T> where T : Recycleable {
  #[inline] 
  fn drop(&mut self) {
    if let Some(mut value) = self.value.take() {
      value.reset();
      self.pool.borrow_mut().push(value);
    }
  }
}

impl <T> fmt::Display for Recycled<T> where T : fmt::Display + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <T> Deref for Recycled<T> where T : Recycleable {
  type Target = T;
  #[inline] 
  fn deref<'a>(&'a self) -> &'a T {
    match self.value.as_ref() {
      Some(v) => v,
      None => panic!("Recycleable wrapper missing its value.")
    }
  }
}

impl <T> DerefMut for Recycled<T> where T : Recycleable {
  #[inline] 
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    match self.value.as_mut() {
      Some(v) => v,
      None => panic!("Recycleable wrapper missing its value.")
    }
  }
}

impl <T> Recycled<T> where T : Recycleable {
  #[inline] 
  pub fn new(pool: Rc<RefCell<Vec<T>>>, value: T) -> Recycled<T> {
    Recycled {
      value: Some(value),
      pool: pool
    }
  }
  
  #[inline] 
  pub fn new_from<A>(pool: Rc<RefCell<Vec<T>>>, mut value: T, source: A) -> Recycled<T> where T : InitializeWith<A> {
    value.initialize_with(source);
    Recycled {
      value: Some(value),
      pool: pool
    }
  }

  #[inline] 
  pub fn detach(mut self) -> T {
    let value = self.value.take().unwrap();
    drop(self);
    value
  }
}

pub struct Pool <T> where T : Recycleable {
  values: Rc<RefCell<Vec<T>>>,
}

impl <T> Pool <T> where T: Recycleable {
  #[inline]
  pub fn with_size(size: u32) -> Pool <T> {
    let values: Vec<T> = 
      (0..size)
      .map(|_| T::new() )
      .collect();
    Pool {
      values: Rc::new(RefCell::new(values)),
    }
  }

  #[inline] 
  pub fn attach(&mut self, mut value: T) {
    value.reset();
    self.values.borrow_mut().push(value);
  }

  #[inline] 
  pub fn detached(&mut self) -> T {
    match self.values.borrow_mut().pop() {
      Some(v) => v,
      None => T::new()
    }
  }

  #[inline] 
  pub fn new(&mut self) -> Recycled<T> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    Recycled::new(pool_reference, t)
  }
 
  #[inline(always)] 
  pub fn new_from<A>(&mut self, source: A) -> Recycled<T> where T: InitializeWith<A> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    Recycled::new_from(pool_reference, t, source)
  }

  #[inline] 
  pub fn size(&self) -> usize {
    self.values.borrow().len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;
  use test::black_box;
  use env_logger;

  const ITERATIONS : u32 = 10_000;

  // Calling String::new() is very close to a no-op; no actual allocation
  // is performed until bytes are pushed onto the end of the String. As such,
  // we need to explicitly ask for some space to be available to trigger allocation.
  const EMPTY_STRING_CAPACITY : usize = 4;

  #[bench]
  fn bench01_standard_allocation_speed(b: &mut Bencher) {
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
        let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
        let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
        let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
        let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
      }
    });
  }

  #[bench]
  fn bench02_pooled_allocation_speed(b: &mut Bencher) {
    let mut pool : Pool<String> = Pool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
      }
    });
  }

  #[bench]
  fn bench03_standard_initialized_allocation_speed(b: &mut Bencher) {
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
  fn bench04_pooled_initialized_allocation_speed(b: &mut Bencher) {
    let _ = env_logger::init();
    let mut pool : Pool<String> = Pool::with_size(5);
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
  #[bench]
  fn bench05_allocate_vec_vec_str(bencher: &mut Bencher) {
      bencher.iter(|| {
          let mut v1 = Vec::new();
          for _ in 0..100 {
              let mut v2 = Vec::new();
              for _ in 0..100 {
                  v2.push(("test!").to_owned());
              }
              v1.push(v2);
          }
          v1
      });
  }

  #[bench]
  fn bench06_pooled_vec_vec_str(bencher: &mut Bencher) {
      let mut vec_str_pool : Pool<Vec<Recycled<String>>> = Pool::with_size(100);
      let mut str_pool : Pool<String> = Pool::with_size(10000);
      bencher.iter(|| {
          let mut v1 = Vec::new();
          for _ in 0..100 {
              let mut v2 = vec_str_pool.new();
              for _ in 0..100 {
                  v2.push(str_pool.new_from("test!"));
              }
              v1.push(v2);
          }
          v1
      });
  }

  #[test]
  fn test_deref() {
      let mut str_pool : Pool<String> = Pool::with_size(1);      
      let rstring = str_pool.new_from("cat");
      assert_eq!("cat", *rstring);
  }

  #[test]
  fn test_deref_mut() {
      let mut str_pool : Pool<String> = Pool::with_size(1);
      let mut rstring = str_pool.new_from("cat");
      (*rstring).push_str("s love eating mice");
      println!("{}", *rstring);
      assert_eq!("cats love eating mice", *rstring);
  }

  #[test]
  fn test_recycle() {
      let mut str_pool : Pool<String> = Pool::with_size(1);
      {
        assert_eq!(1, str_pool.size());
        let _rstring = str_pool.new_from("cat");
        assert_eq!(0, str_pool.size());
      }
      assert_eq!(1, str_pool.size());
  }

  #[test]
  fn test_detached() {
      let mut str_pool : Pool<String> = Pool::with_size(1);
      {
        assert_eq!(1, str_pool.size());
        let _rstring = str_pool.detached();
        assert_eq!(0, str_pool.size());
      }
      assert_eq!(0, str_pool.size());
  }

}
