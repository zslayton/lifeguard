# lifeguard [![](https://api.travis-ci.org/zslayton/lifeguard.png?branch=master)](https://travis-ci.org/zslayton/lifeguard) [![](http://meritbadge.herokuapp.com/lifeguard)](https://crates.io/crates/stomp)
## Object Pool Manager

`lifeguard` issues owned values wrapped in smartpointers.

```rust
extern crate lifeguard;
use lifeguard::Pool;
//...
let pool : Pool<String> = Pool::with_size(10);
{
   let string : Recycled<String> = Pool::new_from("Hello, World!"); // Pool size is now 9
} // Values that have gone out of scope are automatically moved back into the pool.
// Pool size is 10 again
```

Values taken from the pool can be dereferenced to access/mutate their contents.

```rust
extern crate lifeguard;
use lifeguard::Pool;
//...
let mut pool : Pool<String> = Pool::with_size(10);
let mut string = pool.new_from("cat");
(*string).push_str("s love eating mice"); //string.as_mut() also works
assert_eq!("cats love eating mice", *string);
```

Values can be unwrapped, detaching them from the pool.

```rust
let mut pool : Pool<String> = Pool::with_size(10);
{
  let string : String = pool.new().detach();
} // The String goes out of scope and is dropped; it is not returned to the pool
assert_eq!(9, pool.size());
```

Values can be manually entered into / returned to the pool.

```rust
let mut pool : Pool<String> = Pool::with_size(10);
{
  let string : String = pool.detached(); // An unwrapped String, detached from the Pool
  assert_eq!(9, pool.size());
  let rstring : Recycled<String> = pool.attach(string); // The String is attached to the pool again
  assert_eq!(9, pool.size());
} // rstring goes out of scope and is added back to the pool
assert_eq!(10, pool.size());
```

### Highly Unscientific Benchmarks

Benchmark source can be found [here](https://github.com/zslayton/lifeguard/blob/master/benches/lib.rs). Tests were run on a VirtualBox VM with 3 CPUs @ 3Ghz and 4GB of RAM.

| Test Description                                           | Allocating Normally           | Using Object Pool | Improvement
| ---------------------------------------------------------- |:-----------------------------:|:-----------------:|-----------|
| String Allocation<br/>(String::with_capacity vs Pool::new)     | 14379471 ns/iter<br/>(+/- 939144) | 8100463 ns/iter<br/>(+/- 208630) | ~43.67%
| String Duplication<br/>(String::to_owned vs Pool::new_from)     | 22243887 ns/iter<br/>(+/- 1251080) | 17502346 ns/iter<br/>(+/- 1086291) | ~21.32%
| Creating a &lt;Vec&lt;Vec&lt;String>>>     | 1277138 ns/iter<br/>(+/- 114681) | 727415 ns/iter<br/>(+/- 62881) | ~43.04%

Ideas and PRs welcome!

Inspired by frankmcsherry's [recycler](https://github.com/frankmcsherry/recycler).
