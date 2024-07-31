use napi::Env;
use napi::JsObject;
use napi::NapiValue;

use super::console_log;
use super::create_promise;

pub trait UtilsExt {
  /// Runs console.log() in the JavaScript context.
  /// useful for debugging [`NapiValue`] types
  fn console_log<V>(
    &self,
    args: &[V],
  ) -> napi::Result<()>
  where
    V: NapiValue;

  fn create_promise<Res>(
    &self,
    executor: Box<dyn FnOnce(Env, Box<dyn Fn(Res)>, Box<dyn Fn(napi::Error)>) -> napi::Result<()>>,
  ) -> napi::Result<JsObject>
  where
    Res: NapiValue + 'static;
}

impl UtilsExt for Env {
  fn console_log<V>(
    &self,
    args: &[V],
  ) -> napi::Result<()>
  where
    V: NapiValue,
  {
    console_log(self, args)
  }

  fn create_promise<Res>(
    &self,
    executor: Box<dyn FnOnce(Env, Box<dyn Fn(Res)>, Box<dyn Fn(napi::Error)>) -> napi::Result<()>>,
  ) -> napi::Result<JsObject>
  where
    Res: NapiValue + 'static,
  {
    create_promise(self, executor)
  }
}
