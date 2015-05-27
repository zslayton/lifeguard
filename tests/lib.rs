extern crate lifeguard;

#[cfg(test)]
mod tests {
  use lifeguard::{Pool, Recycled};
  
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
  fn test_as_mut() {
      let mut str_pool : Pool<String> = Pool::with_size(1);
      let mut rstring = str_pool.new_from("cat");
      rstring.as_mut().push_str("s love eating mice");
      assert_eq!("cats love eating mice", *rstring);
  }

  #[test]
  fn test_as_ref() {
      let mut str_pool : Pool<String> = Pool::with_size(1);      
      let rstring = str_pool.new_from("cat");
      assert_eq!("cat", rstring.as_ref());
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
        let _string : String = str_pool.new().detach();
        assert_eq!(0, str_pool.size());
      }
      assert_eq!(0, str_pool.size());
  }

  #[test]
  fn test_attach() {
      let mut str_pool : Pool<String> = Pool::with_size(1);
      {
        assert_eq!(1, str_pool.size());
        let string: String = str_pool.new().detach();
        assert_eq!(0, str_pool.size());
        let _rstring: Recycled<String> = str_pool.attach(string);
      }
      assert_eq!(1, str_pool.size());
  }
}
