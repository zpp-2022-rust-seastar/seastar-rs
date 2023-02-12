#[proc_macro_attribute]
pub fn test(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as proc_macro2::TokenStream);
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        let msg = "the async keyword is missing from the function declaration";
        return syn::Error::new_spanned(input.sig, msg)
            .to_compile_error()
            .into();
    }

    if !args.is_empty() {
        let msg = "arguments for #[seastar::test] are not supported yet";
        return syn::Error::new_spanned(args, msg).to_compile_error().into();
    }

    let output = quote::quote! {
        #[test]
        #(#attrs)*
        fn #name() {
            std::thread::spawn(|| {
                let _guard = seastar::acquire_guard_for_seastar_test();
                let mut app = seastar::AppTemplate::default();
                let fut = async {
                    #body
                    Ok(())
                };
                app.run_void(std::env::args().take(1), fut);
            })
            .join()
            .unwrap();
        }
    };

    output.into()
}
