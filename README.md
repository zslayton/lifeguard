# lifeguard [![](https://api.travis-ci.org/zslayton/lifeguard.png?branch=master)](https://travis-ci.org/zslayton/lifeguard) [![](http://meritbadge.herokuapp.com/lifeguard)](https://crates.io/crates/stomp)
## Object Pool Manager

`lifeguard` issues owned values wrapped in smartpointers.

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
  let recyled_string : Recycled<String> = pool.new();
  let string : String = recycled_string.detach();
  // Alternatively:
  // let string : String = pool.detached();
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

### Highly Unscientific Benchmarks

Benchmark source can be found [here](https://github.com/zslayton/lifeguard/blob/master/benches/lib.rs). Tests were run on a VirtualBox VM with 3 CPUs @ 3Ghz and 4GB of RAM.

| Test Description                                           | Allocating Normally           | Using Object Pool | Improvement
| ---------------------------------------------------------- |:-----------------------------:| -----------------:|-----------|
| String Allocation<br/>(String::with_capacity vs Pool::new)     | 14379471 ns/iter<br/>(+/- 939144) | 8100463 ns/iter<br/>(+/- 208630) | ~43.67%
| String Duplication<br/>(String::to_owned vs Pool::new_from)     | 22243887 ns/iter<br/>(+/- 1251080) | 17502346 ns/iter<br/>(+/- 1086291) | ~21.32%
| Creating a &lt;Vec&lt;Vec&lt;String>>>     | 1277138 ns/iter<br/>(+/- 114681) | 727415 ns/iter<br/>(+/- 62881) | ~43.04%

Ideas and PRs welcome!

Inspired by frankmcsherry's [recycler](https://github.com/frankmcsherry/recycler).
