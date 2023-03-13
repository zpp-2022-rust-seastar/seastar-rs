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

#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn main(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as proc_macro2::TokenStream);
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let output = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        let msg = "the async keyword is missing from the function declaration";
        return syn::Error::new_spanned(input.sig, msg)
            .to_compile_error()
            .into();
    }

    if name != "main" {
        let msg = "only the main function is allowed to use #[seastar::main]";
        return syn::Error::new_spanned(input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    if !inputs.is_empty() {
        let msg = "the main function cannot accept arguments";
        return syn::Error::new_spanned(input.sig, msg)
            .to_compile_error()
            .into();
    }

    if !args.is_empty() {
        let msg = "arguments for #[seastar::test] are not supported yet";
        return syn::Error::new_spanned(args, msg).to_compile_error().into();
    }

    let ret_type = match output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => syn::parse_quote! { () },
    };

    let output = quote::quote! {
        #(#attrs)*
        fn main() #output {
            let ret_holder : std::rc::Rc<std::cell::Cell<Option<#ret_type>>> = Default::default();

            let ret_holder_clone = ret_holder.clone();
            let fut = async move {
                let ret = #body;
                ret_holder_clone.set(Some(ret));
                Ok(())
            };

            let mut app = seastar::AppTemplate::default();
            app.run_void(std::env::args(), fut);
            ret_holder.take().unwrap()
        }
    };

    output.into()
}
