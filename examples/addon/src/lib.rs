use std::thread;
use std::time::Duration;

use async_std::channel;
use async_std::task;
use napi::*;
use napi_async_local::JsRc;
use napi_async_local::SpawnLocalExt;
use napi_derive::napi;

#[napi]
pub fn example_a(
  env: Env,
  callback: JsRc<JsFunction>,
) -> napi::Result<JsObject> {
  env.spawn_local(move |env| async move {
    task::sleep(Duration::from_millis(1000)).await;
    callback.inner(&env)?.call_without_args(None)?;
    Ok(())
  })
}

#[napi]
pub fn example_b(
  env: Env,
  callback: JsRc<JsFunction>,
) -> napi::Result<JsObject> {
  let (tx, rx) = channel::unbounded();

  thread::spawn(move || {
    for i in 0..10 {
      tx.send_blocking(i).unwrap();
      thread::sleep(Duration::from_millis(500));
    }
  });

  env.spawn_local(move |env| async move {
    while let Ok(value) = rx.recv().await {
      println!("RS: {}", value);
      callback
        .inner(&env)?
        .call(None, &[env.create_int32(value)?])?;
    }

    Ok(())
  })
}
