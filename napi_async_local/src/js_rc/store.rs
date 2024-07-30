use napi::bindgen_prelude::FromNapiValue;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::Env;
use napi::JsFunction;
use napi::JsNumber;
use napi::JsUnknown;
use napi::NapiValue;
use once_cell::unsync::OnceCell;

use super::root::RootRef;

thread_local! {
  static GLOBAL: OnceCell<RootRef> = Default::default();
  static DROP: OnceCell<ThreadsafeFunction<i32>> = Default::default();
}

pub fn with_global<F, R>(
  env: &Env,
  func: F,
) -> napi::Result<R>
where
  F: FnOnce(&RootRef) -> napi::Result<R>,
{
  DROP.with(|drop| -> napi::Result<()> {
    if drop.get().is_some() {
      return Ok(());
    }

    let jsfn = env.create_function_from_closure::<Vec<JsUnknown>, _>("", |ctx| {
      GLOBAL.with(|global| {
        let global_ref = global.get().unwrap();
        let mut store = global_ref.into_inner(&ctx.env)?;
        let index = ctx.get::<JsNumber>(1)?;
        store.set_property(index, ctx.env.get_undefined()?)?;
        Ok(vec![])
      })
    })?;

    let mut tsfn = env.create_threadsafe_function::<i32, JsNumber, _>(&jsfn, 0, |ctx| {
      let number = ctx.env.create_int32(ctx.value)?;
      Ok(vec![number])
    })?;

    tsfn.unref(env)?;
    drop.set(tsfn).ok();
    Ok(())
  })?;

  GLOBAL.with(move |inner| -> napi::Result<R> {
    let global_ref = inner.get_or_try_init(move || RootRef::new_array(env))?;
    func(global_ref)
  })
}

pub fn set_store_value(
  env: &Env,
  value: impl NapiValue,
) -> napi::Result<i32> {
  with_global(env, |global_ref| {
    let store = global_ref.into_inner(&env)?;

    let push: JsFunction = store.get_named_property_unchecked("push")?;
    let length = JsNumber::from_unknown(push.call(Some(&store), &[value])?)?;
    Ok(length.get_int32()? - 1)
  })
}

pub fn get_store_value<R: NapiValue>(
  env: &Env,
  identifier: &i32,
) -> napi::Result<R> {
  with_global(env, |global_ref| {
    let store = global_ref.into_inner(&env)?;

    let index = env.create_int32(*identifier)?;
    store.get_property_unchecked(index)
  })
}

pub fn delete_store_value(identifier: i32) {
  DROP.with(|drop| {
    if let Some(drop) = drop.get() {
      drop.call(Ok(identifier), ThreadsafeFunctionCallMode::Blocking);
    }
  });
}
