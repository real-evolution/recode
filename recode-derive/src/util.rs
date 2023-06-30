use syn::parse::{Parse, Parser};

pub(super) fn str_to_type(ty: &str) -> syn::Type {
    syn::Type::parse
        .parse_str(ty)
        .unwrap_or_else(|_| panic!("invalid type: {}", ty))
}

pub(super) fn default_buffer_name() -> syn::Ident {
    quote::format_ident!("__buf")
}

pub(super) fn box_type() -> syn::Type {
    str_to_type("Box<dyn std::error::Error>")
}
