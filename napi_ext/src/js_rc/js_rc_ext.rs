use napi::Env;
use napi::NapiRaw;
use napi::NapiValue;

use super::JsRc;

pub trait JsRcExt<T: NapiRaw> {
  /// Wraps the JavaScript value in a reference counted container
  /// that prevents Nodejs's GC from dropping it
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
