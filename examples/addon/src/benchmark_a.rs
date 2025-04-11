use std::thread;

use async_std::channel::unbounded;
use napi::{bindgen_prelude::External, threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode}, Env, JsFunction, JsObject, JsUndefined, JsUnknown};
use napi_ext::*;
use kanal::unbounded_async;

pub struct Control (ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>);

impl Control {
    fn run(&self) {
        self.0.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
    }
}

#[napi_derive::napi]
pub fn benchmark_a_control_before(
    env: Env,
  callback: JsFunction,
) -> napi::Result<External<ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>>> {
    let mut func: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |_ctx: napi::threadsafe_function::ThreadSafeCallContext<()>| Ok::<Vec<JsUnknown>, napi::Error>(vec![]))?;
    // func.unref(&env)?;
    let ext = External::new(func);
    Ok(ext)
}

#[napi_derive::napi]
pub fn benchmark_a_control(
    env: Env,
    callback: JsFunction,
//   callback: External<ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>>,
) -> napi::Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;
    let func: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |_ctx: napi::threadsafe_function::ThreadSafeCallContext<()>| Ok::<Vec<JsUnknown>, napi::Error>(vec![]))?;

    thread::spawn(move || {
        for i in 0..100_000 {
            func.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
        }
        deferred.resolve(|env| env.get_undefined());
    });

    Ok(promise)
}

#[napi_derive::napi]
pub fn benchmark_a_experiment_before(
) -> napi::Result<()> {
    Ok(())
}

#[napi_derive::napi]
pub fn benchmark_a_experiment(
  env: Env,
  callback: JsRc<JsFunction>,
) -> napi::Result<JsObject> {
    let (tx, rx) = unbounded_async();
    
    thread::spawn(move || {
        for i in 0..100_000 {
            tx.as_sync().send(()).unwrap();
        }
    });
    
    env.spawn_local_promise(async move {
        while let Ok(v) = rx.recv().await {
            // println!("{}", v);
            callback.call_without_args(None)?;
        }
        env.get_undefined()
    })
}