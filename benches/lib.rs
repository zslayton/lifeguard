#![feature(test)]
extern crate test;
extern crate lifeguard;

#[cfg(test)]
mod tests {
  use test::Bencher;
  use test::black_box;
  use lifeguard::{Pool,Recycled};

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
}
