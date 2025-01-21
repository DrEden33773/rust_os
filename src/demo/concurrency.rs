use crate::{local_log_ln, print, print_with_color_ln, println};
use alloc::{boxed::Box, vec};
use async_recursion::async_recursion;
use core::num::NonZeroUsize;

cfg_if::cfg_if! {
  if #[cfg(feature = "use_SelfDefinedLRUCache")] {
    type LruCache<K, V> = crate::utils::collections::lru_cache::LruCache<K, V>;
  } else {
    type LruCache<K, V> = lru::LruCache<K, V>;
  }
}

pub async fn cached_show_fib(n: usize) {
  let mut cache = LruCache::new(NonZeroUsize::new(if n >= 2 { n / 2 } else { 1 }).unwrap());

  fn cached_fib(n: usize, cache: &mut LruCache<usize, usize>) -> usize {
    match n {
      0 | 1 => 1,
      _ => {
        if let Some(&result) = cache.get(&n) {
          result
        } else {
          let result = cached_fib(n - 1, cache) + cached_fib(n - 2, cache);
          cache.put(n, result);
          result
        }
      }
    }
  }

  let mut vec = vec![];
  for i in 0..=n {
    vec.push(cached_fib(i, &mut cache));
  }
  local_log_ln!("Cached @ Fibonacci from .0 to .{} ...", n);
  print!("Cached @ Fibonacci := {:?} ... ", vec);

  // assert
  for v in vec.windows(3) {
    if let &[a, b, c] = v {
      if c != a + b {
        print_with_color_ln!([Red] "ERR!\n");
        return;
      }
    }
  }
  print_with_color_ln!(<Green> "OK!\n");
}

pub async fn show_fib(n: usize) {
  #[async_recursion]
  async fn fib(n: usize) -> usize {
    match n {
      0 | 1 => 1,
      _ => fib(n - 1).await + fib(n - 2).await,
    }
  }
  let mut vec = vec![];
  for i in 0..=n {
    vec.push(fib(i).await);
  }
  local_log_ln!("Fibonacci from .0 to .{} ...", n);
  print!("Fibonacci := {:?} ... ", vec);

  // assert
  for v in vec.windows(3) {
    if let &[a, b, c] = v {
      if c != a + b {
        print_with_color_ln!([Red] "ERR!\n");
        return;
      }
    }
  }
  print_with_color_ln!(Green, "OK!\n");
}

pub async fn show_pi() {
  #[async_recursion]
  async fn pi_quarter(n: usize) -> f64 {
    match n {
      0 => 1.0,
      _ => {
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        sign / (2 * n + 1) as f64 + pi_quarter(n - 1).await
      }
    }
  }

  for steps in [500, 1000, 2000, 5000] {
    local_log_ln!("Calculating `PI` in {} steps ...", steps);
    let mut curr_pi = pi_quarter(steps).await;
    curr_pi *= 4.0;
    println!("`PI(.steps :: {steps})` = {}", curr_pi);
  }

  println!();
}
