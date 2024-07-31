use std::ptr;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use napi::bindgen_prelude::FromNapiValue;
use napi::check_status;
use napi::sys;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::Env;
use napi::JsObject;
use napi::JsUnknown;
use napi::NapiRaw;

pub enum RootObjectType {
  Object,
  Array,
}

pub struct RootRef {
  pub raw_ref: sys::napi_ref,
  pub count: Arc<AtomicU32>,
  dropper: Arc<ThreadsafeFunction<()>>,
}

impl RootRef {
  pub fn new_array(env: &Env) -> napi::Result<RootRef> {
    Self::new(env, RootObjectType::Array)
  }

  pub fn new_object(env: &Env) -> napi::Result<RootRef> {
    Self::new(env, RootObjectType::Object)
  }

  pub fn new(
    env: &Env,
    kind: RootObjectType,
  ) -> napi::Result<RootRef> {
    let obj = match kind {
      RootObjectType::Object => env.create_object()?,
      RootObjectType::Array => env.create_empty_array()?,
    };

    let obj_raw = unsafe { obj.raw() };

    let count = Arc::new(AtomicU32::new(1));

    let mut raw_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env.raw(), obj_raw, 1, &mut raw_ref) })?;

    let jsfn = env.create_function_from_closure::<Vec<JsUnknown>, _>("", |_ctx| Ok(vec![]))?;

    let mut tsfn = env.create_threadsafe_function::<(), JsUnknown, _>(&jsfn, 0, {
      let count = count.clone();
      let raw_ref = raw_ref.clone() as usize;

      move |ctx| {
        let mut count: u32 = count.fetch_sub(1, Ordering::Relaxed);
        let raw_ref = raw_ref as sys::napi_ref;
        check_status!(unsafe { sys::napi_reference_unref(ctx.env.raw(), raw_ref, &mut count) })?;
        if count == 0 {
          check_status!(unsafe { sys::napi_delete_reference(ctx.env.raw(), raw_ref) })?;
        }

        Ok(vec![])
      }
    })?;

    tsfn.unref(env)?;

    Ok(RootRef {
      raw_ref,
      count,
      dropper: Arc::new(tsfn),
    })
  }

  pub fn into_inner(
    &self,
    env: &Env,
  ) -> napi::Result<JsObject> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env.raw(), self.raw_ref, &mut result) },
      "Failed to get reference value"
    )?;
    unsafe { JsObject::from_napi_value(env.raw(), result) }
  }

  pub fn clone(
    &self,
    env: &Env,
  ) -> napi::Result<Self> {
    let mut count: u32 = self.count.fetch_add(1, Ordering::Relaxed);
    check_status!(unsafe { sys::napi_reference_ref(env.raw(), self.raw_ref, &mut count) })?;
    Ok(RootRef {
      raw_ref: self.raw_ref.clone(),
      count: self.count.clone(),
      dropper: self.dropper.clone(),
    })
  }

  pub fn get(
    &self,
    env: &Env,
  ) -> napi::Result<JsObject> {
    let mut napi_value = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_get_reference_value(env.raw(), self.raw_ref, &mut napi_value)
    })?;
    let value = unsafe { JsObject::from_napi_value(env.raw(), napi_value)? };
    Ok(value)
  }
}

impl Drop for RootRef {
  fn drop(&mut self) {
    self
      .dropper
      .call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
  }
}
