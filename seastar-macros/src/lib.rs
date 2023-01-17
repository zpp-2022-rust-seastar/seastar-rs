extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Procedural macro which provides easy way to write
/// tests in Rust that run on Seastar runtime.
#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        let msg = "the async keyword is missing from the function declaration";
        return syn::Error::new_spanned(input.sig.fn_token, msg)
            .to_compile_error()
            .into();
    }

    let output = quote! {
        #[test]
        #(#attrs)*
        fn #name() #ret {
            let app = AppTemplate::new_from_options(Options::new());
            app.run_void(&vec![String::from("test")], || { block_on(async { #body }); });
        }
    };

    output.into()
}
