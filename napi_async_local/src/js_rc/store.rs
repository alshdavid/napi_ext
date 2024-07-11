use std::rc::Rc;

use napi::bindgen_prelude::FromNapiValue;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::Env;
use napi::JsFunction;
use napi::JsNumber;
use napi::JsObject;
use napi::JsUnknown;
use napi::NapiValue;
use once_cell::unsync::OnceCell;

static GLOBAL_KEY: &str = "__napi_jsrc_store__";

thread_local! {
  static STORE: OnceCell<()> = Default::default();
  static DROP: OnceCell<Rc<ThreadsafeFunction<i32>>> = Default::default();
}

/*
  Currently this takes JavaScript values and stores them in a JavaScript array
  on the Nodejs "globalThis" object.

  This ensures they won't be GC'd by Nodejs and allows their life cycle to be
  managed by the Rust addon.

  The downside is that this is a publicly accessible property.

  TODO & NOTES:
    Find a way to make this inaccessible.

    I have tried making the "__napi_jsrc_store__" array a napi ref however this
    didn't seem to work reliably. Will need play around a bit more to get this
    working.

  globalThis = {
    __napi_jsrc_store__: []
  }
*/
pub fn initialize(env: &Env) -> napi::Result<()> {
  STORE.with(|init| {
    if init.get().is_some() {
      return Ok(());
    }

    let mut global = env.get_global()?;

    if !global.has_named_property(GLOBAL_KEY)? {
      let obj = env.create_empty_array()?;
      global.set_named_property(GLOBAL_KEY, obj)?;
    }

    DROP.with(|drop| -> napi::Result<()> {
      if drop.get().is_some() {
        return Ok(());
      }

      let jsfn = env.create_function_from_closure::<Vec<JsUnknown>, _>("", |ctx| {
        let mut store = get_store(&ctx.env).unwrap();
        let index = ctx.get::<JsNumber>(1)?;
        store.set_property(index, ctx.env.get_undefined()?)?;
        Ok(vec![])
      })?;

      let mut tsfn = env.create_threadsafe_function::<i32, JsNumber, _>(&jsfn, 0, |ctx| {
        let number = ctx.env.create_int32(ctx.value)?;
        Ok(vec![number])
      })?;

      tsfn.unref(env)?;
      drop.set(Rc::new(tsfn)).ok();
      Ok(())
    })?;

    init.set(()).unwrap();
    Ok(())
  })
}

pub fn get_store(env: &Env) -> napi::Result<JsObject> {
  initialize(env)?;
  let global = env.get_global()?;
  global.get_named_property_unchecked::<JsObject>(GLOBAL_KEY)
}

pub fn set_store_value(
  env: &Env,
  value: impl NapiValue,
) -> napi::Result<i32> {
  let store = get_store(env)?;
  let push: JsFunction = store.get_named_property_unchecked("push")?;
  let length = JsNumber::from_unknown(push.call(Some(&store), &[value])?)?;
  Ok(length.get_int32()? - 1)
}

pub fn get_store_value(
  env: &Env,
  identifier: &i32,
) -> napi::Result<JsUnknown> {
  let index = env.create_int32(*identifier)?;
  let store = get_store(env)?;
  store.get_property_unchecked(index)
}

pub fn delete_store_value(identifier: i32) {
  DROP.with(|drop| {
    if let Some(drop) = drop.get() {
      drop.call(Ok(identifier), ThreadsafeFunctionCallMode::Blocking);
    }
  });
}
