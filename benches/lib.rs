#[macro_use]
extern crate criterion;
extern crate lifeguard;

use criterion::{black_box, Criterion};

use lifeguard::{Pool,Recycled,RcRecycled};

const ITERATIONS : u32 = 10_000;

// Calling String::new() is very close to a no-op; no actual allocation
// is performed until bytes are pushed onto the end of the String. As such,
// we need to explicitly ask for some space to be available to trigger allocation.
const EMPTY_STRING_CAPACITY : usize = 4;

fn allocation(c: &mut Criterion) {
  c.bench_function("allocation standard", |b| b.iter(|| {
    for _ in 0..ITERATIONS {
      let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
      let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
      let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
      let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
      let _string = black_box(String::with_capacity(EMPTY_STRING_CAPACITY));
    }
  }));

  c.bench_function("allocation pooled", |b| {
    let pool : Pool<String> = Pool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
        let _string = pool.new();
      }
    });
  });

  c.bench_function("allocation pooled rc", |b| {
    let pool : Pool<String> = Pool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_rc();
        let _string = pool.new_rc();
        let _string = pool.new_rc();
        let _string = pool.new_rc();
        let _string = pool.new_rc();
      }
    });
  });
}

fn initialized_allocation(c: &mut Criterion) {
  c.bench_function("initialized allocation standard", |b| {
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = "man".to_owned();
        let _string = "dog".to_owned();
        let _string = "cat".to_owned();
        let _string = "mouse".to_owned();
        let _string = "cheese".to_owned();
      }
    });
  });

  c.bench_function("initialized allocation pooled", |b| {
    let pool : Pool<String> = Pool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_from("man");
        let _string = pool.new_from("dog");
        let _string = pool.new_from("cat");
        let _string = pool.new_from("mouse");
        let _string = pool.new_from("cheese");
      }
    });
  });

  c.bench_function("initialized allocation pooled rc", |b| {
    let pool : Pool<String> = Pool::with_size(5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_rc_from("man");
        let _string = pool.new_rc_from("dog");
        let _string = pool.new_rc_from("cat");
        let _string = pool.new_rc_from("mouse");
        let _string = pool.new_rc_from("cheese");
      }
    });
  });

  c.bench_function("initialized allocation pooled with cap empty", |b| {
    let pool : Pool<String> = Pool::with_size_and_max(0, 5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_from("man");
        let _string = pool.new_from("dog");
        let _string = pool.new_from("cat");
        let _string = pool.new_from("mouse");
        let _string = pool.new_from("cheese");
      }
    });
  });

  c.bench_function("initialized allocation pooled with cap full", |b| {
    let pool : Pool<String> = Pool::with_size_and_max(5, 5);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_from("man");
        let _string = pool.new_from("dog");
        let _string = pool.new_from("cat");
        let _string = pool.new_from("mouse");
        let _string = pool.new_from("cheese");
      }
    });
  });

  c.bench_function("initialized allocation pooled with low cap", |b| {
    let pool : Pool<String> = Pool::with_size_and_max(0, 2);
    b.iter(|| {
      for _ in 0..ITERATIONS {
        let _string = pool.new_from("man");
        let _string = pool.new_from("dog");
        let _string = pool.new_from("cat");
        let _string = pool.new_from("mouse");
        let _string = pool.new_from("cheese");
      }
    });
  });
}

fn vec_vec_str(c: &mut Criterion) {
  c.bench_function("vec vec str standard", |b| {
    b.iter(|| {
      let mut v1 = Vec::new();
      for _ in 0..100 {
          let mut v2 = Vec::new();
          for _ in 0..100 {
              v2.push(("test!").to_owned());
          }
          v1.push(v2);
      }
      v1
    })
  });

  c.bench_function("vec vec str pooled", |b| {
    // Note that because we're using scoped values (not Rc'ed values)
    // and we're storing items from one pool in the other,
    // the order that our pools are declared matters. 
    // Reversing them results in a compile error regarding lifetimes.
    let str_pool : Pool<String> = Pool::with_size(10000);
    let vec_str_pool : Pool<Vec<Recycled<String>>> = Pool::with_size(100);
    b.iter(|| {
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
  });

  c.bench_function("vec vec str pooled rc", |b| {
    let vec_str_pool : Pool<Vec<RcRecycled<String>>> = Pool::with_size(100);
    let str_pool : Pool<String> = Pool::with_size(10000);
    b.iter(|| {
        let mut v1 = Vec::new();
        for _ in 0..100 {
            let mut v2 = vec_str_pool.new_rc();
            for _ in 0..100 {
                v2.push(str_pool.new_rc_from("test!"));
            }
            v1.push(v2);
        }
        v1
    });
  });
}

criterion_group!(allocation_benches, allocation);
criterion_group!(initialized_allocation_benches, initialized_allocation);
criterion_group!(vec_vec_str_benches, vec_vec_str);
criterion_main!(allocation_benches, initialized_allocation_benches, vec_vec_str_benches);
