use syn::parse::{Parse, Parser};

pub(crate) fn default_buffer_name() -> Option<syn::Ident> {
    Some(quote::format_ident!("__buf"))
}

pub(crate) fn default_error_type() -> Option<syn::Type> {
    Some(
        syn::Type::parse
            .parse2(quote::quote!(Box<dyn std::error::Error>))
            .unwrap(),
    )
}

