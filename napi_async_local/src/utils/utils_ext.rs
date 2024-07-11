use napi::Env;
use napi::NapiValue;

use super::console_log;

pub trait UtilsExt {
  /// Runs console.log() in the JavaScript context.
  /// useful for debugging [`NapiValue`] types
  fn console_log<V>(
    &self,
    args: &[V],
  ) -> napi::Result<()>
  where
    V: NapiValue;
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
}
