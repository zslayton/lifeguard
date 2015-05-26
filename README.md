# lifeguard
## Object Pool Manager

### String Recycling
```rust
extern crate lifeguard;
use lifeguard::Pool;

fn main() {
  let mut pool = Pool::with_size(5);
  for _ in 0..10_000 {
    let _ = pool.new_from("man");
    let _ = pool.new_from("dog");
    let _ = pool.new_from("cat");
    let _ = pool.new_from("mouse");
    let _ = pool.new_from("cheese");
  } // All values are added back to the pool once they have gone out of scope
}
```
