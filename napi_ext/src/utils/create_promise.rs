use std::cell::Cell;

use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::NapiValue;

use crate::store;

const SYM_PROMISE: &str = "Promise";
const SYM_PROMISE_EXECUTOR: &str = "napi::promise::executor";

pub type PromiseExecutor<Res> =
  Box<dyn FnOnce(Env, Box<dyn Fn(Res)>, Box<dyn Fn(napi::Error)>) -> napi::Result<()>>;

pub fn create_promise<Res>(
  env: &Env,
  executor: PromiseExecutor<Res>,
) -> napi::Result<JsObject>
where
  Res: NapiValue + 'static,
{
  let promise_key = env.create_string(SYM_PROMISE)?;
  let promise_ctor: JsFunction = env.get_global()?.get_property_unchecked(promise_key)?;

  let executor = {
    let cell = Cell::new(Some(executor));
    move |env, res, rej| {
      cell
        .take()
        .expect("This function should not be called more than once")(env, res, rej)
    }
  };

  let executor = env.create_function_from_closure(SYM_PROMISE_EXECUTOR, move |ctx| {
    let resolve_func_js: JsFunction = ctx.get(0)?;
    let reject_func: JsFunction = ctx.get(1)?;

    let resolve_func_key = store::set_store_value(ctx.env, resolve_func_js)?;
    let reject_func_key = store::set_store_value(ctx.env, reject_func)?;

    executor(
      ctx.env.to_owned(),
      Box::new({
        let env = *ctx.env;
        move |r| {
          let resolve_func = store::get_store_value::<JsFunction>(&env, &resolve_func_key).unwrap();
          resolve_func.call(None, &[r]).unwrap();
          store::delete_store_value(resolve_func_key);
          store::delete_store_value(reject_func_key);
        }
      }),
      Box::new({
        let env = *ctx.env;
        move |e| {
          let reject_func = store::get_store_value::<JsFunction>(&env, &reject_func_key).unwrap();
          let error = (env).create_error(e).unwrap();
          reject_func.call(None, &[error]).unwrap();
          store::delete_store_value(resolve_func_key);
          store::delete_store_value(reject_func_key);
        }
      }),
    )?;

    Ok(())
  })?;

  let promise = promise_ctor.new_instance(&[executor])?;
  Ok(promise)
}
