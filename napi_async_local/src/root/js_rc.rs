use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use napi::bindgen_prelude::FromNapiValue;
use napi::sys::napi_env;
use napi::sys::napi_value;
use napi::Env;
use napi::NapiRaw;
use napi::NapiValue;

use super::store;

pub struct JsRc<T: NapiRaw> {
  count: Rc<RefCell<usize>>,
  identifier: Rc<i32>,
  kind: PhantomData<T>,
}

impl<T: NapiValue> JsRc<T> {
  pub fn new(
    env: &Env,
    value: T,
  ) -> napi::Result<Self> {
    Ok(Self {
      count: Rc::new(RefCell::new(1)),
      identifier: Rc::new(store::set_store_value(env, value)?),
      kind: Default::default(),
    })
  }

  pub fn inner<'a>(
    &self,
    env: &'a Env,
  ) -> napi::Result<JsRcHandle<'a, T>> {
    let value = store::get_store_value(env, &self.identifier)?;
    let value: T = unsafe { value.cast() };
    Ok(JsRcHandle {
      inner: value,
      _env: env,
    })
  }
}

impl<T: NapiRaw> Drop for JsRc<T> {
  fn drop(&mut self) {
    let mut count = self.count.borrow_mut();
    *count -= 1;
    if *count == 0 {
      store::delete_store_value(*self.identifier)
    }
  }
}

impl<T: NapiRaw> Clone for JsRc<T> {
  fn clone(&self) -> Self {
    let mut count = self.count.borrow_mut();
    *count += 1;

    Self {
      count: self.count.clone(),
      identifier: self.identifier.clone(),
      kind: self.kind.clone(),
    }
  }
}

impl<T: NapiValue> FromNapiValue for JsRc<T> {
  unsafe fn from_napi_value(
    env: napi_env,
    napi_val: napi_value,
  ) -> napi::Result<Self> {
    let value = T::from_raw_unchecked(env, napi_val);
    let env = Env::from_raw(env);
    Self::new(&env, value)
  }
}

pub struct JsRcHandle<'a, T: NapiValue> {
  inner: T,
  _env: &'a Env,
}

impl<'a, T: NapiValue> Deref for JsRcHandle<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
