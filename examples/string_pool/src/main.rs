extern crate lifeguard;
use lifeguard::Pool;

fn main() {
  let mut pool : Pool<String> = Pool::with_size(5);
  for _ in 0..10_000 {
    let _ = pool.new_from("man");
    let _ = pool.new_from("dog");
    let _ = pool.new_from("cat");
    let _ = pool.new_from("mouse");
    let _ = pool.new_from("cheese");
  }
}

