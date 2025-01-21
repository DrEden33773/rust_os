use alloc::sync::Arc;
use spin::mutex::Mutex;

pub async fn mutex() {
  const RES: usize = 3;

  let counter = Arc::new(Mutex::<usize>::new(0));
  for _ in 0..RES {
    let _ = counter.clone();
  }
}
