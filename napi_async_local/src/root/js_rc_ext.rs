use napi::Env;
use napi::NapiRaw;
use napi::NapiValue;

use crate::JsRc;

pub trait JsRcExt<T: NapiRaw> {
  fn into_rc(
    self,
    env: &Env,
  ) -> napi::Result<JsRc<T>>;
}

impl<T: NapiValue> JsRcExt<T> for T {
  fn into_rc(
    self,
    env: &Env,
  ) -> napi::Result<JsRc<T>> {
    JsRc::new(env, self)
  }
}
