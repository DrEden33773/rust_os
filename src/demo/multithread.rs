use alloc::{borrow::ToOwned, sync::Arc};
use spin::mutex::Mutex;

pub async fn mutex() {
  const RES: usize = 3;

  let counter = Arc::new(Mutex::<usize>::new(0));
  for i in 0..RES {
    // let _ = counter.clone();
    let mut v = Arc::new(Mutex::<usize>::new(i));
    counter.clone_into(&mut v);
  }
}
