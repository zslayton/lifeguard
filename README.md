# lifeguard
## Object Pool Manager

Lifeguard issues owned values wrapped in smartpointers.

```rust
extern crate lifeguard;
use lifeguard::Pool;
//...
let pool : Pool<String> = Pool::with_size(10);
{
   assert_eq!(pool.size(), 10);
   let string : Recycled<String> = Pool::new_from("Hello, World!");
   assert_eq!(pool.size(), 9);
} // Values that have gone out of scope are automatically moved back into the pool.
assert_eq!(pool.size(), 10);
```

Values taken from the pool can be dereferenced to access/mutate their contents.

```rust
extern crate lifeguard;
use lifeguard::Pool;
//...
let mut pool : Pool<String> = Pool::with_size(1);
let mut string = pool.new_from("cat");
(*string).push_str("s love eating mice");
assert_eq!("cats love eating mice", *string);
```

Values can be unwrapped, detaching them from the pool.

```rust
let mut pool : Pool<String> = Pool::with_size(1);
{
  assert_eq!(1, pool.size());
  let string : String = pool.detached(); // An unwrapped String, detached from the Pool
  assert_eq!(0, pool.size());
} // The String goes out of scope and is dropped; it is not returned to the pool
assert_eq!(0, str_pool.size());
```

Values can be manually entered into / returned to the pool.

```rust
let mut pool : Pool<String> = Pool::with_size(1);
{
  assert_eq!(1, pool.size());
  let string : String = pool.detached(); // An unwrapped String, detached from the Pool
  assert_eq!(0, pool.size());
  pool.attach(string);
  assert_eq!(1, pool.size());
} // The String is owned by the pool now
assert_eq!(1, pool.size());
```

### String Recycling
```rust
extern crate lifeguard;
use lifeguard::Pool;
use test::Bencher;

const ITERATIONS : u32 = 10_000;

#[bench]
fn standard_initialized_allocation_speed(b: &mut Bencher) {
  // Allocating without a pool
  b.iter(|| {
    for _ in 0..ITERATIONS {
      let _ = "man".to_owned();
      let _ = "dog".to_owned();
      let _ = "cat".to_owned();
      let _ = "mouse".to_owned();
      let _ = "cheese".to_owned();
    }
  });
}

#[bench]
fn pooled_initialized_allocation_speed(b: &mut Bencher) {
  let mut pool : Pool<String> = Pool::with_size(5);
  b.iter(|| {
    for _ in 0..ITERATIONS {
      let _ = pool.new_from("man");
      let _ = pool.new_from("dog");
      let _ = pool.new_from("cat");
      let _ = pool.new_from("mouse");
      let _ = pool.new_from("cheese");
    } // All strings go out of scope here and are automatically returned to the pool
  });
}
}
```
On a VirtualBox VM, this simple test yielded:
```
test tests::standard_initialized_allocation_speed ... bench:   2194050 ns/iter (+/- 179969)
test tests::pooled_initialized_allocation_speed   ... bench:   1715122 ns/iter (+/- 165090)
```
An improvement of about 22%; this should likely be more dramatic. Ideas and PRs welcome!

Inspired by frankmcsherry's [recycler](https://github.com/frankmcsherry/recycler).
