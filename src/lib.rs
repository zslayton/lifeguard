#![allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Drop, Deref, DerefMut};
use std::convert::{AsRef, AsMut};

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

impl <T> AsRef<T> for Recycled<T> where T : Recycleable {
   fn as_ref(&self) -> &T {
    match self.value.as_ref() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <T> AsMut<T> for Recycled<T> where T : Recycleable {
   fn as_mut(&mut self) -> &mut T {
    match self.value.as_mut() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <T> fmt::Debug for Recycled<T> where T : fmt::Debug + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
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
    self.as_ref()
  }
}

impl <T> DerefMut for Recycled<T> where T : Recycleable {
  #[inline] 
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.as_mut()
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
  pub fn attach(&mut self, value: T) -> Recycled<T> {
    let pool_reference = self.values.clone();
    Recycled::new(pool_reference, value)
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
