use bindgen_prelude::FromNapiValue;
use napi::*;
use napi_async_local::*;

#[napi_async]
pub async fn example_e(env: Env, value: JsRc<JsString>) -> napi::Result<JsUndefined> {
  let value = value.into_inner(&env)?;
  
  env.console_log(&[value])?;
  env.get_undefined()
}
