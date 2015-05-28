#![allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Drop, Deref, DerefMut};
use std::convert::{AsRef, AsMut};
use std::borrow::Borrow;

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

pub struct RcRecycled<T> where T: Recycleable {
  value: RecycledInner<Rc<RefCell<Vec<T>>>, T>
}

pub struct Recycled<'a, T: 'a> where T: Recycleable {
  value: RecycledInner<&'a RefCell<Vec<T>>, T>
}

macro_rules! impl_recycled {
  ($name: ident, $typ: ty, $pool: ty) => {
  impl <'a, T> AsRef<T> for $typ where T : Recycleable {
     fn as_ref(&self) -> &T {
      self.value.as_ref()
    }
  }

  impl <'a, T> AsMut<T> for $typ where T : Recycleable {
     fn as_mut(&mut self) -> &mut T {
      self.value.as_mut()
    }
  }

  impl <'a, T> fmt::Debug for $typ where T : fmt::Debug + Recycleable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      self.value.fmt(f)
    }
  }

  impl <'a, T> fmt::Display for $typ where T : fmt::Display + Recycleable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      self.value.fmt(f)
    }
  }

  impl <'a, T> Deref for $typ where T : Recycleable {
    type Target = T;
    #[inline] 
    fn deref(&self) -> &T {
      self.as_ref()
    }
  }

  impl <'a, T> DerefMut for $typ where T : Recycleable {
    #[inline] 
    fn deref_mut(&mut self) -> &mut T {
      self.as_mut()
    }
  }

  impl <'a, T> $typ where T: Recycleable {
    pub fn new(pool: $pool, value: T) -> $typ {
      $name { value: RecycledInner::new(pool, value) }
    }
    
    #[inline] 
    pub fn new_from<A>(pool: $pool, value: T, source: A) -> $typ where T : InitializeWith<A> {
      $name { value: RecycledInner::new_from(pool, value, source) }
    }

    #[inline] 
    pub fn detach(self) -> T {
      self.value.detach()
    }
  }
}
}
impl_recycled!{ RcRecycled, RcRecycled<T>, Rc<RefCell<Vec<T>>> }
impl_recycled!{ Recycled, Recycled<'a, T>, &'a RefCell<Vec<T>> }

struct RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
  value: Option<T>,
  pool: P
}

impl <P, T> Drop for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
  #[inline] 
  fn drop(&mut self) {
    if let Some(mut value) = self.value.take() {
      value.reset();
      self.pool.borrow().borrow_mut().push(value);
    }
  }
}

impl <P, T> AsRef<T> for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
   fn as_ref(&self) -> &T {
    match self.value.as_ref() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <P, T> AsMut<T> for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
   fn as_mut(&mut self) -> &mut T {
    match self.value.as_mut() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <P, T> fmt::Debug for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : fmt::Debug + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <P, T> fmt::Display for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : fmt::Display + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <P, T> Deref for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
  type Target = T;
  #[inline] 
  fn deref<'a>(&'a self) -> &'a T {
    self.as_ref()
  }
}

impl <P, T> DerefMut for RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
  #[inline] 
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.as_mut()
  }
}

impl <P, T> RecycledInner<P, T> where P: Borrow<RefCell<Vec<T>>>, T : Recycleable {
  #[inline] 
  fn new(pool: P, value: T) -> RecycledInner<P, T> {
    RecycledInner {
      value: Some(value),
      pool: pool
    }
  }
  
  #[inline] 
  fn new_from<A>(pool: P, mut value: T, source: A) -> RecycledInner<P, T> where T : InitializeWith<A> {
    value.initialize_with(source);
    RecycledInner {
      value: Some(value),
      pool: pool
    }
  }

  #[inline] 
  fn detach(mut self) -> T {
    let value = self.value.take().unwrap();
    drop(self);
    value
  }
}

pub struct Pool <T> where T : Recycleable {
  values: Rc<RefCell<Vec<T>>>
}

impl <T> Pool <T>
  where T: Recycleable {

  #[inline]
  pub fn with_size(size: u32) -> Pool <T> {
    let values: Vec<T> = 
      (0..size)
      .map(|_| T::new() )
      .collect();
    Pool {
      values: Rc::new(RefCell::new(values))
    }
  }

  #[inline] 
  pub fn attach_rc(&self, value: T) -> RcRecycled<T> {
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new(pool_reference, value) }
  }

  #[inline] 
  pub fn new_rc(&self) -> RcRecycled<T> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new(pool_reference, t) }
  }
 
  #[inline(always)] 
  pub fn new_rc_from<A>(&self, source: A) -> RcRecycled<T> where T: InitializeWith<A> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new_from(pool_reference, t, source) }
  }

  #[inline] 
  pub fn attach(&self, value: T) -> Recycled<T> {
    Recycled { value: RecycledInner::new(&*self.values, value) }
  }

  #[inline] 
  pub fn new(&self) -> Recycled<T> {
    let t = self.detached();
    Recycled { value: RecycledInner::new(&*self.values, t) }
  }

  #[inline(always)] 
  pub fn new_from<A>(&self, source: A) -> Recycled<T> where T: InitializeWith<A> {
    let t = self.detached();
    Recycled { value: RecycledInner::new_from(&*self.values, t, source) }
  }

  #[inline] 
  pub fn detached(&self) -> T {
    match self.values.borrow_mut().pop() {
      Some(v) => v,
      None => T::new()
    }
  }

  #[inline] 
  pub fn size(&self) -> usize {
    (*self.values).borrow().len()
  }
}
  
