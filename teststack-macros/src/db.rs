use syn::{LitStr, Token, parenthesized, parse::Parse};

pub mod keyword {
    syn::custom_keyword!(postgres);
    syn::custom_keyword!(mysql);
    syn::custom_keyword!(random_db_name);
    syn::custom_keyword!(db_name);
}

pub(super) fn peek(input: &syn::parse::ParseStream) -> bool {
    input.peek(keyword::postgres) || input.peek(keyword::mysql)
}

#[derive(Default, Debug)]
pub struct DbConf {
    pub db_name: DbName,
}

impl Parse for DbConf {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::random_db_name) {
            let _k: keyword::random_db_name = input.parse()?;
            Ok(Self {
                db_name: DbName::Random,
            })
        } else {
            let _k: keyword::db_name = input.parse()?;
            let _t: Token![=] = input.parse()?;
            let name: LitStr = input.parse()?;
            Ok(Self {
                db_name: DbName::Static(name.value()),
            })
        }
    }
}

#[derive(Default, Debug)]
pub enum DbName {
    Random,
    Static(String),
    #[default]
    Default,
}

#[derive(Default, Debug)]
pub struct DbContainer {
    pub name: &'static str,
    pub conf: DbConf,
}

impl Parse for DbContainer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = if input.peek(keyword::postgres) {
            let _k: keyword::postgres = input.parse()?;
            "postgres"
        } else if input.peek(keyword::mysql) {
            let _k: keyword::mysql = input.parse()?;
            "mysql"
        } else {
            return Err(input.error("expected 'postgres' or 'mysql'"));
        };
        let conf = if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            Default::default()
        };
        Ok(Self { name, conf })
    }
}
