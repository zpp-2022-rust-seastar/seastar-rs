extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// A macro intended for running asynchronous tests.
/// Tests are spawned in a separate thread.
/// This is done to ensure thread_local cleanup between them
/// (at the time of writing, Seastar doesn't do it itself).
///
/// # Usage
///
/// ```rust
/// #[seastar_macros::test]
/// async fn my_test() {
///     assert!(true);
/// }
/// ```
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
                let _guard = crate::acquire_guard_for_seastar_test();
                let mut app = AppTemplate::default();
                let args = std::env::args().collect::<Vec<_>>();
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
