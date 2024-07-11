use std::cell::Cell;

use futures::Future;
use napi::bindgen_prelude::ToNapiValue;
use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::JsUnknown;
use napi::NapiValue;

use crate::async_local;
use crate::store;
#[allow(unused)]
use crate::JsRc;

pub trait SpawnLocalExt {
  /// Spawns a non-blocking future on the local thread. Returns a Promise with the value 
  /// returned in the async closure. Normal [`NapiValue`] types can be interacted with in 
  /// the async context. Supports channels, timers, etc.
  /// 
  /// To ensure the availability of [`NapiValue`] types beyond the life of the parent function scope,
  /// ensure that [`NapiValue`] types that will be used in an async closure are wrapped in a [`JsRc`].
  /// 
  /// ### Usage:
  /// 
  /// #### Running a Callback:
  /// 
  /// ```
  /// use std::time::Duration;
  /// 
  /// use napi::*;
  /// use napi_derive::napi;
  /// use napi_async_local::JsRc;
  /// use napi_async_local::JsRcExt;
  /// use napi_async_local::SpawnLocalExt;
  /// use async_std::task;
  /// 
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<JsObject> {
  ///   env.spawn_local(move |env| async move {
  ///     task::sleep(Duration::from_millis(1000)).await;
  ///     callback.inner(&env)?.call_without_args(None)?;
  ///     Ok(())
  ///   })
  /// }
  /// ```
  /// 
  /// #### Using Channels:
  /// 
  /// ```
  /// use std::thread;
  /// use std::time::Duration;
  /// 
  /// use napi::*;
  /// use napi_derive::napi;
  /// use napi_async_local::JsRc;
  /// use napi_async_local::JsRcExt;
  /// use napi_async_local::SpawnLocalExt;
  /// use async_std::channel;
  /// 
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<JsObject> {
  ///   let (tx, rx) = channel::unbounded();
  /// 
  ///   thread::spawn(move || {
  ///     for i in 0..10 {
  ///       tx.send_blocking(i).unwrap();
  ///       thread::sleep(Duration::from_millis(1000));
  ///     }
  ///   });
  /// 
  ///   env.spawn_local(move |env| async move {
  ///     while let Ok(value) = rx.recv().await {
  ///       println!("Got number: {}", value);
  ///       callback.inner(&env)?.call(None, &[env.create_int32(value)?])?;
  ///     }
  /// 
  ///     Ok(())
  ///   })
  /// }
  /// ```
  fn spawn_local<R, F, Fut>(
    &self,
    future: F,
  ) -> napi::Result<JsObject>
  where
    R: ToNapiValue,
    F: FnOnce(Env) -> Fut + 'static,
    Fut: Future<Output = napi::Result<R>> + 'static;
}

impl SpawnLocalExt for Env {
  fn spawn_local<R, F, Fut>(
    &self,
    callback: F,
  ) -> napi::Result<JsObject>
  where
    R: ToNapiValue,
    F: FnOnce(Env) -> Fut + 'static,
    Fut: Future<Output = napi::Result<R>> + 'static,
  {
    let promise_key = self.create_string("Promise")?;
    let promise_ctor: JsFunction = self.get_global()?.get_property_unchecked(promise_key)?;
    let callback = to_fn(callback);

    let resolver = self.create_function_from_closure("napi::promise::executor", move |ctx| {
      let resolve_func: JsFunction = ctx.get(0)?;
      let reject_func: JsFunction = ctx.get(1)?;

      let resolve_func_key = store::set_store_value(ctx.env, resolve_func)?;
      let reject_func_key = store::set_store_value(ctx.env, reject_func)?;

      let env_raw = ctx.env.raw();
      let future = callback(unsafe { Env::from_raw(env_raw.clone()) });

      async_local::spawn_async_local(ctx.env, async move {
        let env = unsafe { Env::from_raw(env_raw.clone()) };

        let emit_error = |error: napi::Error| {
          let reject_func =
            store::get_store_value(&env, &reject_func_key).expect("Unable to get stored value");
          let reject_func = unsafe { reject_func.cast::<JsFunction>() };
          let error = env.create_error(error).unwrap();
          reject_func
            .call(None, &vec![error])
            .expect("Unable to call reject function");
        };

        match future.await {
          Ok(result) => 'block: {
            let resolve_func =
              store::get_store_value(&env, &resolve_func_key).expect("Unable to get stored value");
            let resolve_func = unsafe { resolve_func.cast::<JsFunction>() };

            let Ok(napi_val) = (unsafe { R::to_napi_value(env.raw(), result) }) else {
              emit_error(napi::Error::from_reason("Unable to parse return value"));
              break 'block;
            };

            let unknown = unsafe { JsUnknown::from_raw_unchecked(env_raw, napi_val) };

            if resolve_func.call(None, &vec![unknown]).is_err() {
              emit_error(napi::Error::from_reason("Unable to call resolve function"));
              break 'block;
            }
          }
          Err(error) => emit_error(error),
        }
        
        store::delete_store_value(resolve_func_key);
        store::delete_store_value(reject_func_key);
      })?;

      Ok(Vec::<JsUnknown>::new())
    })?;

    let promise = promise_ctor.new_instance(&[resolver])?;
    Ok(promise)
  }
}

fn to_fn<A, R>(f: impl FnOnce(A) -> R) -> impl Fn(A) -> R {
  let cell = Cell::new(Some(f));
  move |a| {
    cell
      .take()
      .expect("This function should not be called more than once")(a)
  }
}
