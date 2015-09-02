#![allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Drop, Deref, DerefMut};
use std::convert::{AsRef, AsMut};
use std::borrow::Borrow;
use std::collections::VecDeque;

/// In order to be managed by a `Pool`, values must be of a type that
/// implements the `Recycleable` trait. This allows the `Pool` to create
/// new instances as well as reset existing instances to a like-new state.
pub trait Recycleable {
  /// Allocates a new instance of the implementing type.
  fn new() -> Self;
  /// Sets the state of the modified instance to be that of a freshly
  /// allocated instance, thereby allowing it to be reused.
  fn reset(&mut self);
}

/// Informs how an already allocated value should be initialized 
/// when provided with a model value or other meaningful input.
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

impl <T> Recycleable for VecDeque<T> {
  #[inline] 
  fn new() -> VecDeque<T> {
    VecDeque::new()
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

/// A smartpointer which uses reference counting (`Rc`) to know
/// when to move its wrapped value back to the `Pool` that
/// issued it.
pub struct RcRecycled<T> where T: Recycleable {
  value: RecycledInner<Rc<RefCell<CappedCollection<T>>>, T>
}

/// A smartpointer which uses a shared reference (`&`) to know
/// when to move its wrapped value back to the `Pool` that
/// issued it.
pub struct Recycled<'a, T: 'a> where T: Recycleable {
  value: RecycledInner<&'a RefCell<CappedCollection<T>>, T>
}

macro_rules! impl_recycled {
  ($name: ident, $typ: ty, $pool: ty) => {
  impl <'a, T> AsRef<T> for $typ where T : Recycleable {
     /// Gets a shared reference to the value wrapped by the smartpointer.
     fn as_ref(&self) -> &T {
      self.value.as_ref()
    }
  }

  impl <'a, T> AsMut<T> for $typ where T : Recycleable {
     /// Gets a mutable reference to the value wrapped by the smartpointer.
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
    fn new(pool: $pool, value: T) -> $typ {
      $name { value: RecycledInner::new(pool, value) }
    }
    
    #[inline] 
    fn new_from<A>(pool: $pool, value: T, source: A) -> $typ where T : InitializeWith<A> {
      $name { value: RecycledInner::new_from(pool, value, source) }
    }

    #[inline] 
    /// Disassociates the value from the `Pool` that issued it. This
    /// destroys the smartpointer and returns the previously wrapped value.
    pub fn detach(self) -> T {
      self.value.detach()
    }
  }
}
}
impl_recycled!{ RcRecycled, RcRecycled<T>, Rc<RefCell<CappedCollection<T>>> }
impl_recycled!{ Recycled, Recycled<'a, T>, &'a RefCell<CappedCollection<T>> }

struct RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
  value: Option<T>,
  pool: P
}

impl <P, T> Drop for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
  #[inline] 
  fn drop(&mut self) {
    if let Some(value) = self.value.take() {
      self.pool.borrow().borrow_mut().insert_or_drop(value);
    }
  }
}

impl <P, T> AsRef<T> for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
   fn as_ref(&self) -> &T {
    match self.value.as_ref() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <P, T> AsMut<T> for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
   fn as_mut(&mut self) -> &mut T {
    match self.value.as_mut() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <P, T> fmt::Debug for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : fmt::Debug + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <P, T> fmt::Display for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : fmt::Display + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <P, T> Deref for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
  type Target = T;
  #[inline] 
  fn deref<'a>(&'a self) -> &'a T {
    self.as_ref()
  }
}

impl <P, T> DerefMut for RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
  #[inline] 
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.as_mut()
  }
}

impl <P, T> RecycledInner<P, T> where P: Borrow<RefCell<CappedCollection<T>>>, T : Recycleable {
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

struct CappedCollection <T> where T: Recycleable {
  values: Vec<T>,
  cap: usize
}

impl <T> CappedCollection <T> where T: Recycleable {
  #[inline]
  pub fn new(starting_size: usize, max_size: usize) -> CappedCollection<T> {
    use std::cmp;
    let starting_size = cmp::min(starting_size, max_size);
    let values: Vec<T> = 
      (0..starting_size)
      .map(|_| T::new() )
      .collect();
    CappedCollection {
      values: values,
      cap: max_size
    }
  }

  #[inline]
  pub fn insert_or_drop(&mut self, mut value: T) {
    match self.is_full() {
      true => drop(value),
      false => {
        value.reset();
        self.values.push(value)
      }
    }
  }

  #[inline]
  pub fn remove(&mut self) -> Option<T> {
    self.values.pop()
  }

  #[inline]
  pub fn is_full(&self) -> bool {
    self.values.len() >= self.cap
  }
  
  #[inline]
  pub fn len(&self) -> usize {
    self.values.len()
  }
  
  #[inline]
  pub fn cap(&self) -> usize {
    self.cap
  }
}

/// A collection of values that can be reused without requiring new allocations.
/// 
/// `Pool` issues each value wrapped in a smartpointer. When the smartpointer goes out of
/// scope, the wrapped value is automatically returned to the pool.
pub struct Pool <T> where T : Recycleable {
  values: Rc<RefCell<CappedCollection<T>>>
}

impl <T> Pool <T> where T: Recycleable {

  /// Creates a pool with `size` elements of type `T` allocated.
  #[inline]
  pub fn with_size(size: usize) -> Pool <T> {
    use std::usize;
    Pool::with_size_and_max(size, usize::max_value())
  }

  /// Creates a pool with `size` elements of type `T` allocated
  /// and sets a maximum pool size of `max_size`. Values being
  /// added to the pool via `Pool::attach` or being returned to
  /// the pool upon dropping will instead be discarded if the pool
  /// is full.
  #[inline]
  pub fn with_size_and_max(starting_size: usize, max_size: usize) -> Pool <T> {
    let values: CappedCollection<T> = CappedCollection::new(starting_size, max_size);
    Pool {
      values: Rc::new(RefCell::new(values))
    }
  }

  /// Returns the number of values remaining in the pool.
  #[inline] 
  pub fn size(&self) -> usize {
    (*self.values).borrow().len()
  }

  /// Removes a value from the pool and returns it wrapped in
  /// a `Recycled smartpointer. If the pool is empty when the
  /// method is called, a new value will be allocated.
  #[inline] 
  pub fn new(&self) -> Recycled<T> {
    let t = self.detached();
    Recycled { value: RecycledInner::new(&*self.values, t) }
  }

  /// Removes a value from the pool, initializes it using the provided
  /// source value, and returns it wrapped in a `Recycled` smartpointer.
  /// If the pool is empty when the method is called, a new value will be
  /// allocated.
  #[inline(always)] 
  pub fn new_from<A>(&self, source: A) -> Recycled<T> where T: InitializeWith<A> {
    let t = self.detached();
    Recycled { value: RecycledInner::new_from(&*self.values, t, source) }
  }

  /// Associates the provided value with the pool by wrapping it in a
  /// `Recycled` smartpointer.
  #[inline] 
  pub fn attach(&self, value: T) -> Recycled<T> {
    Recycled { value: RecycledInner::new(&*self.values, value) }
  }

  /// Removes a value from the pool and returns it without wrapping it in
  /// a smartpointer. When the value goes out of scope it will not be
  /// returned to the pool.
  #[inline] 
  pub fn detached(&self) -> T {
    match self.values.borrow_mut().remove() {
      Some(v) => v,
      None => T::new()
    }
  }

  /// Removes a value from the pool and returns it wrapped in
  /// an `RcRecycled` smartpointer. If the pool is empty when the
  /// method is called, a new value will be allocated.
  #[inline] 
  pub fn new_rc(&self) -> RcRecycled<T> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new(pool_reference, t) }
  }
 
  /// Removes a value from the pool, initializes it using the provided
  /// source value, and returns it wrapped in an `RcRecycled` smartpointer.
  /// If the pool is empty when the method is called, a new value will be
  /// allocated.
  #[inline(always)] 
  pub fn new_rc_from<A>(&self, source: A) -> RcRecycled<T> where T: InitializeWith<A> {
    let t = self.detached();
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new_from(pool_reference, t, source) }
  }

  /// Associates the provided value with the pool by wrapping it in an
  /// `RcRecycled` smartpointer.
  #[inline] 
  pub fn attach_rc(&self, value: T) -> RcRecycled<T> {
    let pool_reference = self.values.clone();
    RcRecycled { value: RecycledInner::new(pool_reference, value) }
  }
}
