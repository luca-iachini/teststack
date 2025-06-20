use custom::CustomContainer;
use db::{DbContainer, DbName};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, ItemFn};

mod custom;
mod db;

enum Container {
    Db(DbContainer),
    Custom(CustomContainer),
}

impl Parse for Container {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if db::peek(&input) {
            Ok(Container::Db(DbContainer::parse(input)?))
        } else if custom::peek(&input) {
            Ok(Container::Custom(CustomContainer::parse(input)?))
        } else {
            Err(input.error("expected a container"))
        }
    }
}

#[proc_macro_attribute]
pub fn stack(attr: TokenStream, item: TokenStream) -> TokenStream {
    let containers =
        parse_macro_input!(attr with Punctuated::<Container, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(item as ItemFn);
    let containers = containers.into_iter().collect();
    expand_test(input, containers)
}

fn expand_test(input: ItemFn, containers: Vec<Container>) -> TokenStream {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let args = &input.sig.inputs;
    let sqlx_test = attrs
        .iter()
        .any(|attr| attr.path().segments.iter().any(|s| s.ident == "sqlx"));

    let container_vars = (0..containers.len())
        .map(|i| format_ident!("container_{i}"))
        .collect::<Vec<_>>();

    let containers = containers
        .iter()
        .map(|container| match container {
            Container::Db(db) => {
                let db_name = match &db.conf.db_name {
                    DbName::Random => quote! {::teststack::DbName::Random },
                    DbName::Static(name) => {
                        quote! {::teststack::DbName::Static(#name.to_string()) }
                    }
                    DbName::Default => quote! { ::teststack::DbName::Default },
                };
                match db.name {
                    "postgres" => quote! { ::teststack::postgres(#db_name) },
                    "mysql" => quote! { ::teststack::mysql(#db_name) },
                    _ => panic!("Unknown container type: {}", name),
                }
            }
            Container::Custom(custom) => {
                let expr = &custom.expr;
                quote! { ::teststack::custom(#expr) }
            }
        })
        .collect::<Vec<_>>();

    if sqlx_test {
        quote! {
            #[allow(unnameable_test_items)]
            #[::core::prelude::v1::test]
            fn #name() #ret {
                let rt = ::tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    ::tokio::join!(#(#containers),*);
                });
                #(#attrs)*
                fn #name(#args) #ret {
                    #body
                }
                #name()
            }
        }
        .into()
    } else {
        let test_args = args.iter().enumerate().map(|(i, arg)| {
            let arg = match arg {
                syn::FnArg::Typed(arg) => arg,
                _ => panic!("Expected a typed argument"),
            };
            let ty = &arg.ty;
            let container_ident = format_ident!("container_{i}");
            quote! { ::teststack::Init::<#ty>::init(#container_ident).await }
        });
        quote! {
            #(#attrs)*
            async fn #name() #ret {
                use ::teststack::Init;
               let (#(#container_vars),*,) = ::tokio::join!(#(#containers),*);
                async fn #name(#args) #ret {
                    #body
                }
                #name(#(#test_args),*).await
            }
        }
        .into()
    }
}
