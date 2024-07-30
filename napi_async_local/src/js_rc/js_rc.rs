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

/// Reference counter for JavaScript values.
///
/// Moves ownership of a JavaScript value to the Rust addon, preventing
/// Nodejs's GC from dropping the value.
///
/// The value will be dropped when the reference count goes to 0
///
/// ### Usage:
///
/// #### From Napi Function Argument
///
/// ```
/// use napi::*;
/// use napi_derive::napi;
/// use napi_async_local::JsRc;
///
/// #[napi]
/// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<()> {
///   let inner = callback.inner(&env)?;
///   inner.call_without_args(None);
///   Ok(())
/// }
/// ```
///
/// #### Apply to Existing Value
///
/// Simplified with [`super::JsRcExt`]
///
/// ```
/// use napi::*;
/// use napi_derive::napi;
/// use napi_async_local::JsRc;
/// use napi_async_local::UtilsExt;
///
/// #[napi]
/// fn my_js_func(env: Env) -> napi::Result<()> {
///   let value = env.create_string("hello world")?;
///   let value_ref = JsRc::new(value)?;
///   let inner = value_ref.inner(&env)?;
///   
///   env.console_log(&[inner])?; // From UtilsExt
///   Ok(())
/// }
/// ```
///
/// #### Apply to Existing Value [`super::JsRcExt`]
///
/// ```
/// use napi::*;
/// use napi_derive::napi;
/// use napi_async_local::JsRcExt;
/// use napi_async_local::UtilsExt;
///
/// #[napi]
/// fn my_js_func(env: Env) -> napi::Result<()> {
///   let value = env.create_string("hello world")?;
///   let value_ref = value.into_rc(&env)?; // From JsRcExt
///   let inner = value_ref.inner(&env)?;
///   
///   env.console_log(&[inner])?; // From UtilsExt
///   Ok(())
/// }
/// ```
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

  /// Obtain a reference to the inner JavaScript value
  pub fn inner<'a>(
    &self,
    env: &'a Env,
  ) -> napi::Result<JsRcRef<'a, T>> {
    let value = store::get_store_value::<T>(env, &self.identifier)?;
    Ok(JsRcRef {
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
  /// Clone the container and increment the reference count by 1
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

// Makes the value usable as an argument in a Napi function
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

/// Container for a referenced value within a [`JsRc`]
pub struct JsRcRef<'a, T: NapiValue> {
  inner: T,
  _env: &'a Env,
}

impl<'a, T: NapiValue> Deref for JsRcRef<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
