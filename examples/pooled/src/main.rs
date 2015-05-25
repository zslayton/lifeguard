extern crate value_pool;
extern crate env_logger;
use value_pool::StringPool;

fn main() {
  let _ = env_logger::init();
  let mut pool = StringPool::with_size(5);
  for _ in 0..10_000 {
    let _string = pool.new_from("man");
    let _string = pool.new_from("dog");
    let _string = pool.new_from("cat");
    let _string = pool.new_from("mouse");
    let _string = pool.new_from("cheese");
  }
}

