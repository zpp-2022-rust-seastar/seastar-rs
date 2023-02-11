extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

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
        fn #name() {
            std::thread::spawn(|| {
                let _guard = seastar::acquire_guard_for_seastar_test();
                let mut app = seastar::AppTemplate::default();
                let args = vec!["test"]; // TODO: replace with actual args
                let fut = async {
                    #body
                    Ok(())
                };
                app.run_void(&args[..], fut);
            })
            .join()
            .unwrap();
        }
    };

    output.into()
}
