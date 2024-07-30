use futures::Future;
use napi::Env;
use napi::JsObject;
use napi::NapiValue;

use crate::runtime;
use crate::utils::UtilsExt;

pub fn spawn_local<F, Fut>(
  env: &Env,
  callback: F,
) -> napi::Result<()>
where
  F: FnOnce(Env) -> Fut + 'static,
  Fut: Future<Output = napi::Result<()>> + 'static,
{
  let future = callback(env.to_owned());

  runtime::spawn_async_local(&env, async move {
    if let Err(error) = future.await {
      eprintln!("Uncaught Napi Error: {}", error);
    };
  })?;

  Ok(())
}

pub fn spawn_local_promise<R, F, Fut>(
  env: &Env,
  callback: F,
) -> napi::Result<JsObject>
where
  R: NapiValue + 'static,
  F: FnOnce(Env) -> Fut + 'static,
  Fut: Future<Output = napi::Result<R>> + 'static,
{
  env.create_promise(Box::new(move |env, resolve_func, reject_func| {
    let future = callback(env);
    runtime::spawn_async_local(&env, async move {
      match future.await {
        Ok(result) => resolve_func(result),
        Err(error) => reject_func(error),
      };
    })
  }))
}
