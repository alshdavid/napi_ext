use std::cell::Cell;

use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::NapiValue;

const SYM_JS_PROMISE_THEN: &str = "napi::promise::then";

pub struct JsPromise<'a> {
  env: &'a Env,
  inner: JsObject,
}

impl<'a> JsPromise<'a> {
  pub fn from_object(
    env: &'a Env,
    inner: JsObject,
  ) -> napi::Result<Self> {
    Ok(Self {
      // inner: inner.into_rc(env)?,
      inner,
      env,
    })
  }

  pub fn then<V>(
    &self,
    callback: impl FnOnce(Env, V) -> napi::Result<()> + 'static,
  ) -> napi::Result<&Self>
  where
    V: NapiValue,
  {
    let callback = {
      let cell = Cell::new(Some(callback));
      move |env, arg0| {
        cell
          .take()
          .expect("This function should not be called more than once")(env, arg0)
      }
    };

    let then_callback = self
      .env
      .create_function_from_closure(SYM_JS_PROMISE_THEN, move |ctx| {
        let arg0 = ctx.get(0)?;
        callback(ctx.env.clone(), arg0)
      })?;

    self
      .inner
      .get_named_property::<JsFunction>("then")?
      .call::<JsFunction>(Some(&self.inner), &[then_callback])?;

    Ok(self)
  }

  pub fn catch<V>(
    &self,
    callback: impl FnOnce(Env, V) -> napi::Result<()> + 'static,
  ) -> napi::Result<&Self>
  where
    V: NapiValue,
  {
    let callback = {
      let cell = Cell::new(Some(callback));
      move |env, arg0| {
        cell
          .take()
          .expect("This function should not be called more than once")(env, arg0)
      }
    };

    let then_callback = self
      .env
      .create_function_from_closure(SYM_JS_PROMISE_THEN, move |ctx| {
        let arg0 = ctx.get(0)?;
        callback(ctx.env.clone(), arg0)
      })?;

    self
      .inner
      .get_named_property::<JsFunction>("catch")?
      .call::<JsFunction>(Some(&self.inner), &[then_callback])?;

    Ok(self)
  }
}
