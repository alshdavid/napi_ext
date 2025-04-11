use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::Ident;
use syn::ItemFn;
use syn::ReturnType;

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
  let mut pre_body = proc_macro2::TokenStream::new();

  let mut insert_env = proc_macro2::TokenStream::new();
  let mut has_env = false;

  for input in raw_inputs.iter() {
    match &input {
      syn::FnArg::Receiver(_r) => continue,
      syn::FnArg::Typed(t) => {
        let pat = &*t.pat;
        if let syn::Type::Path(p) = &*t.ty {
          if let Some(segment) = p.path.segments.last() {
            if segment.ident == "Env" {
              has_env = true;
            } else if segment.ident == "JsString"
              || segment.ident == "JsUnknown"
              || segment.ident == "JsUndefined"
              || segment.ident == "JsNull"
              || segment.ident == "JsBoolean"
              || segment.ident == "JsBuffer"
              || segment.ident == "JsArrayBuffer"
              || segment.ident == "JsTypedArray"
              || segment.ident == "JsDataView"
              || segment.ident == "JsNumber"
              || segment.ident == "JsString"
              || segment.ident == "JsObject"
              || segment.ident == "JsGlobal"
              || segment.ident == "JsDate"
              || segment.ident == "JsFunction"
              || segment.ident == "JsExternal"
              || segment.ident == "JsSymbol"
              || segment.ident == "JsTimeout"
              || segment.ident == "JSON"
            {
              pre_body.append_all(
                quote! {let #pat = #pat.into_rc(&env).unwrap().into_inner(&env).unwrap();
                },
              );
            };
          }
        }
        input_names.append_all(pat.to_token_stream());
        input_names.append_all(quote! {,});
      }
    }
  }

  if !has_env {
    insert_env = quote! {env: Env,}
  }

  if let ReturnType::Default = func.sig.output {
    let new_output = syn::parse2::<ItemFn>(quote! {fn x() -> () { () }})?;
    func.sig.output = new_output.sig.output;
  }

  let ident = func.sig.ident.clone();
  func.sig.ident = Ident::new(&format!("async_local_{}", ident), ident.span());
  let new_ident = &func.sig.ident;

  let ret = match &func.sig.output {
    syn::ReturnType::Default => quote! {napi::Result<napi::JsUndefined>},
    syn::ReturnType::Type(_, v) => quote!(#v),
  };

  Ok(quote! {
    #func

    #[napi_derive::napi]
    fn #ident(#insert_env #raw_inputs) -> napi::Result<JsObject> {
      #pre_body

      let fut = #new_ident(#input_names);
      env.spawn_local_promise(fut)
    }
  })
}
