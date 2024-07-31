use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::Ident;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn napi_async(
  _args: TokenStream,
  input: TokenStream,
) -> TokenStream {
  convert(input.into())
    .unwrap_or_else(|err| err.into_compile_error())
    .into()
}

fn convert(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
  let mut func = syn::parse2::<ItemFn>(input)?;

  if func.sig.asyncness.is_none() {
    return Err(syn::Error::new_spanned(func, "Failed to do the thing"));
  }

  let raw_inputs = &func.sig.inputs;
  let mut input_names = proc_macro2::TokenStream::new();

  for input in raw_inputs.iter() {
    match &input {
      syn::FnArg::Receiver(_r) => continue,
      syn::FnArg::Typed(t) => {
        input_names.append_all(t.pat.to_token_stream());
        input_names.append_all(quote! {,});
      }
    }
  }

  let ident = func.sig.ident.clone();
  func.sig.ident = Ident::new(&format!("async_local_{}", ident.to_string()), ident.span());
  let new_ident = &func.sig.ident;

  let ret = match &func.sig.output {
    syn::ReturnType::Default => quote! {napi::Result<napi::JsUndefined>},
    syn::ReturnType::Type(_, v) => quote!(#v),
  };

  Ok(quote! {
    #func

    #[napi_derive::napi]
    fn #ident(#raw_inputs) -> napi::Result<JsObject> {
      let fut = #new_ident(#input_names);
      env.spawn_local_promise(move |env| async move {
        unsafe {
          let env_raw = env.raw();
          <#ret as napi::bindgen_prelude::ToNapiValue>::
            to_napi_value(env_raw, fut.await)
            .and_then(|v| JsUnknown::from_napi_value(env_raw, v))
        }
      })
    }
  })
}
