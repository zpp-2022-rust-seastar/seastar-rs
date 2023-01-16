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
        use futures::executor::block_on;
        use crate::config_and_start_seastar::{AppTemplate, Options};
        #[test]
        #(#attrs)*
        fn #name() #ret {
            let app = AppTemplate::new_from_options(Options::new());
            app.run_void(&vec![String::from("test")], || { block_on(async { #body }); });
        }
    };

    output.into()
}

/// The macro that runs the `main` function's contents in Seastar's runtime.
/// **Only** `main` is allowed to use this macro - Seastar apps may only use
/// one `AppTemplate` instance.
///
/// # Options
/// TODO
///
/// # Usage
///
/// ## Using default
///
/// ```rust
/// #[seastar::main]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        let msg = "the async keyword is missing from the function declaration";
        return syn::Error::new_spanned(input.sig.fn_token, msg)
            .to_compile_error()
            .into();
    }

    if name != "main" {
        let msg = "only the main function is allowed to use #[seastar::main]";
        return syn::Error::new_spanned(&input.sig.inputs, msg)
            .to_compile_error()
            .into();
    }

    if !inputs.is_empty() {
        let msg = "the main function cannot accept arguments";
        return syn::Error::new_spanned(&input.sig.inputs, msg)
            .to_compile_error()
            .into();
    }

    if !args.is_empty() {
        let msg = "arguments for #[seastar::main] are not supported yet";
        return syn::Error::new_spanned(&input.sig.inputs, msg)
            .to_compile_error()
            .into();
    }

    let output = quote! {
        use futures::executor::block_on;
        use crate::config_and_start_seastar::{AppTemplate, Options};
        #(#attrs)*
        fn main {
            let app = AppTemplate::default();
            let args = std::env::args().collect::<Vec<_>>();
            app.run_void(&args, || { block_on(async { #body }); });
        }
    };

    output.into()
}
