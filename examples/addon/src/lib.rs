use std::thread;
use std::time::Duration;

use async_std::channel;
use async_std::task;
use bindgen_prelude::FromNapiValue;
use napi::*;
use napi_ext::*;
use napi_derive::napi;

#[napi]
pub fn example_a(
  env: Env,
  callback: JsRc<JsFunction>,
) -> napi::Result<()> {
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
) -> napi::Result<()> {
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

#[napi]
pub fn example_c(env: Env) -> napi::Result<JsObject> {
  env.spawn_local_promise(move |env| async move {
    task::sleep(Duration::from_millis(1000)).await;
    env.create_string("Hello World")
  })
}

#[napi_async]
pub async fn example_d(
  env: Env,
  value: JsRc<JsString>,
) -> napi::Result<JsUndefined> {
  let value = value.into_inner(&env)?;

  env.console_log(&[value])?;
  env.get_undefined()
}
