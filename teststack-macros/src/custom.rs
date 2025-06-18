use syn::{Expr, parenthesized, parse::Parse};

mod keyword {
    syn::custom_keyword!(container);
}

pub(super) fn peek(input: &syn::parse::ParseStream) -> bool {
    input.peek(keyword::container)
}

pub(super) struct CustomContainer {
    pub expr: Expr,
}

impl Parse for CustomContainer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _k = input.parse::<keyword::container>()?;
        let content;
        parenthesized!(content in input);
        let expr = content.parse()?;
        Ok(CustomContainer { expr })
    }
}
