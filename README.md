# lifeguard [![](https://api.travis-ci.org/zslayton/lifeguard.png?branch=master)](https://travis-ci.org/zslayton/lifeguard) [![](http://meritbadge.herokuapp.com/lifeguard)](https://crates.io/crates/lifeguard)
## Object Pool Manager

`lifeguard` issues owned values wrapped in smartpointers.

```rust
extern crate lifeguard;
use lifeguard::{Pool, Recycled};

fn main() {
    let mut pool : Pool<String> = Pool::with_size(10);
    {
        let string = pool.new_from("Hello, World!"); // Pool size is now 9
    } // Values that have gone out of scope are automatically moved back into the pool.
    // Pool size is 10 again
}
```

Values taken from the pool can be dereferenced to access/mutate their contents.

```rust
extern crate lifeguard;
use lifeguard::Pool;

fn main() {
    let mut pool : Pool<String> = Pool::with_size(10);
    let mut string = pool.new_from("cat");
    string.push_str("s love eating mice"); //string.as_mut() also works
    assert_eq!("cats love eating mice", *string);
}
```

Values can be unwrapped, detaching them from the pool.

```rust
extern crate lifeguard;
use lifeguard::Pool;

fn main() {
    let mut pool : Pool<String> = Pool::with_size(10);
    {
        let string : String = pool.new().detach();
    } // The String goes out of scope and is dropped; it is not returned to the pool
    assert_eq!(9, pool.size());
}
```

Values can be manually entered into / returned to the pool.

```rust
extern crate lifeguard;
use lifeguard::{Pool, Recycled};

fn main() {
    let mut pool : Pool<String> = Pool::with_size(10);
    {
        let string : String = pool.detached(); // An unwrapped String, detached from the Pool
        assert_eq!(9, pool.size());
        let rstring : Recycled<String> = pool.attach(string); // The String is attached to the pool again
        assert_eq!(9, pool.size()); // but it is still checked out from the pool
    } // rstring goes out of scope and is added back to the pool
    assert_eq!(10, pool.size());
}
```

### Highly Unscientific Benchmarks

Benchmark source can be found [here](https://github.com/zslayton/lifeguard/blob/master/benches/lib.rs). Tests were run on a VirtualBox VM with 3 CPUs @ 3Ghz and 4GB of RAM.

#### Uninitialized Allocation

| `String::with_capacity`      | `Pool::new_rc`             | Improvement | `Pool::new`                | Improvement |
|:----------------------------:|:--------------------------:|:-----------:|:--------------------------:|:-----------:|
| 1421183 ns/iter (+/- 161572) | 841286 ns/iter (+/- 78602) |  ~40.80%    | 615875 ns/iter (+/- 53906) |   ~56.67%   |

#### Initialized Allocation

| `String::to_owned`           | `Pool::new_rc_from`         | Improvement | `Pool::new_from`             | Improvement |
|:----------------------------:|:---------------------------:|:-----------:|:----------------------------:|:-----------:|
| 2256492 ns/iter (+/- 184229) | 1652247 ns/iter (+/- 185096)|  ~26.78%    | 1430212 ns/iter (+/- 146108) |   ~36.62%   |

#### Vec<Vec<String>>> Allocation
Adapted from [this benchmark](https://github.com/frankmcsherry/recycler/blob/master/benches/benches.rs#L10).

| `Vec::new` + `String::to_owned` | `Pool::new_rc` + `Pool::new_rc_from` | Improvement | `Pool::new` + `Pool::new_from`| Improvement |
|:-------------------------------:|:------------------------------------:|:-----------:|:------------------------------:|:-----------:|
| 1303594 ns/iter (+/- 98974)     | 723338 ns/iter (+/- 82782)  |  ~44.51%    | 678324 ns/iter (+/- 88772)   |   ~47.97%   |

Ideas and PRs welcome!

Inspired by frankmcsherry's [recycler](https://github.com/frankmcsherry/recycler).
