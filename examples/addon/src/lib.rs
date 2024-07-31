use std::process::Output;
use std::thread;
use std::time::Duration;

use async_std::channel;
use async_std::task;
use napi::*;
use napi_async_local::napi_async;
use napi_async_local::prelude::*;
use napi_derive::napi;

pub async fn example_f(env: Env) -> napi::Result<JsObject> {
  env.spawn_local_promise::<JsUndefined,_,_>(|env| async move {
    env.get_undefined()
  })
}


#[napi_async]
pub async fn example_e(env: Env) -> napi::Result<JsUndefined> {
  println!("hi");
  env.get_undefined()
}
