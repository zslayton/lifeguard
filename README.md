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

### Benchmarks
[Current benchmarks](https://github.com/zslayton/lifeguard/blob/master/src/lib.rs#L187) show speedups between 20 and 40% depending on how much of the task at hand involves pure allocation; this should likely be more dramatic. Ideas and PRs welcome!

Inspired by frankmcsherry's [recycler](https://github.com/frankmcsherry/recycler).
