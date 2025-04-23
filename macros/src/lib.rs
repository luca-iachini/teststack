use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn postgres(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let args = &input.sig.inputs;

    quote! {
        #[allow(clippy::unnameable_test_items)]
        #[::core::prelude::v1::test]
        fn #name() #ret {
            ::stack::postgres();
            #(#attrs)*
            fn #name(#args) #ret {
                #body
            }
            #name()
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn mysql(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let args = &input.sig.inputs;

    quote! {
        #[allow(clippy::unnameable_test_items)]
        #[::core::prelude::v1::test]
        fn #name() #ret {
            ::stack::mysql();
            #(#attrs)*
            fn #name(#args) #ret {
                #body
            }
            #name()
        }
    }
    .into()
}
