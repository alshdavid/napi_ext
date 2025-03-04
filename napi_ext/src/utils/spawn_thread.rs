use std::sync::Arc;
use std::thread;

use napi::bindgen_prelude::FromNapiValue;
use napi::bindgen_prelude::ToNapiValue;
use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::ThreadSafeCallContext;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::JsUnknown;
use once_cell::sync::OnceCell;

type MapJsParams = Box<dyn Send + FnOnce(&Env) -> napi::Result<Vec<JsUnknown>> + 'static>;

/// Creates a system thread and returns a Promise back to JavaScript.
/// Captures errors and returns them as a JavaScript tuple
pub fn spawn_thread<ThreadFunc, NapiFunc, NapiRet>(
  env: &Env,
  func: ThreadFunc,
) -> napi::Result<JsObject>
where
  ThreadFunc: FnOnce() -> napi::Result<NapiFunc> + Send + 'static,
  NapiFunc: FnOnce(Env) -> napi::Result<NapiRet> + Send + 'static,
  NapiRet: ToNapiValue,
{
  // Captures the executor function of Promise creation as a threadsafe function
  //   new Promise(resolve => {})
  //               -------  <- this bit
  let resolve_fn = Arc::new(OnceCell::new());

  // This is the callback supplied to `new Promise(executor)`
  let executor = env.create_function_from_closure("Promise::executor", {
    let resolve_fn = resolve_fn.clone();
    move |ctx| {
      let resolve: ThreadsafeFunction<MapJsParams, ErrorStrategy::Fatal> = ctx
        .get::<JsFunction>(0)?
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<MapJsParams>| {
          (ctx.value)(&ctx.env)
        })?;
      resolve_fn.set(resolve).ok();
      Ok(())
    }
  })?;

  // Construct a new Promise
  let promise_ctor: JsFunction = env.get_global()?.get_named_property("Promise")?;
  let promise = promise_ctor.new_instance(&[&executor])?;

  // Spawn a thread to execute the off-thread work
  // then calls the Promise.resolve function (threadsafe).
  thread::spawn(move || {
    // Call the function on the new thread
    let result = func();

    // Process the return value on the JS thread
    resolve_fn.wait().call(
      Box::new(move |env| match result {
        Ok(value) => {
          // Execute the function passed in by the caller
          let data = match value(*env) {
            Ok(data) => data,
            Err(error) => return Err(error),
          };

          // Safety: value is checked as being a ToNapiValue on the caller
          let js_value = unsafe {
            JsUnknown::from_napi_value(env.raw(), NapiRet::to_napi_value(env.raw(), data)?)?
          };

          Ok(vec![js_value])
        }
        // Capture errors and and return them as string values
        Err(error) => Err(error),
      }),
      ThreadsafeFunctionCallMode::NonBlocking,
    );
  });

  Ok(promise)
}
