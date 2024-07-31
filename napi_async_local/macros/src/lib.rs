use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn napi_async(
  _args: TokenStream,
  input: TokenStream,
) -> TokenStream {
  convert(input.into())
    .unwrap_or_else(|err| err.into_compile_error())
    .into()
  // if func.sig.asyncness.is_none() {
  // eprintln!("`napi` macro expand failed.");
  // return quote!(#func).into();
  // }

  // quote!(#func).into()
  // format!("{:#}", item).parse().unwrap()
  // format!("#[napi]\n{}", input).parse().unwrap()
}

fn convert(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
  let func = syn::parse2::<ItemFn>(input)?;

  if func.sig.asyncness.is_none() {
    return Err(syn::Error::new_spanned(func, "Failed to do the thing"));
  }

  let ident = func.sig.ident;
  let block = func.block;

  let ret = match func.sig.output {
    syn::ReturnType::Default => quote! {},
    syn::ReturnType::Type(_, v) => quote! {#v},
  };

  Ok(quote! {
    fn #ident(env: Env) -> napi::Result<napi::JsObject> {
      env.spawn_local_promise::<#ret,_,_>(|env| async move
        #block
      )
    }
  })
}
