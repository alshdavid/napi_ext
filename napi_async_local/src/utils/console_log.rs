use napi::Env;
use napi::JsFunction;
use napi::JsObject;
use napi::JsString;
use napi::NapiRaw;

pub fn console_log<V>(
  env: &Env,
  args: &[V],
) -> napi::Result<()>
where
  V: NapiRaw,
{
  let key_console = env.create_string("console")?;
  let key_log = env.create_string("log")?;

  env
    .get_global()?
    .get_property_unchecked::<JsString, JsObject>(key_console)?
    .get_property_unchecked::<JsString, JsFunction>(key_log)?
    .call(None, args)?;

  Ok(())
}
