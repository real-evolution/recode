use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};

use crate::field::RecodeField;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(decoder), supports(struct_named))]
pub(super) struct Decoder {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), RecodeField>,
    error: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
}

impl darling::ToTokens for Decoder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Decoder {
            ref ident,
            ref generics,
            ref data,
            ref error,
            ref buffer_name,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let error = error.clone().unwrap_or(
            syn::Type::parse
                .parse2(quote!(Box<dyn std::error::Error>))
                .unwrap(),
        );

        let buffer_name =
            buffer_name.clone().unwrap_or(quote::format_ident!("__buf"));

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_names = fields.iter().map(|f| f.ident());
        let field_exprs =
            fields.iter().map(|&f| f.to_decode_stmt(&buffer_name));

        tokens.extend(quote! {
            impl #imp recode::Decoder for #ident #ty #wher {
                type Output = Self;
                type Error = #error;

                fn decode<B: recode::bytes::Buf>(#buffer_name: &mut B)
                    -> Result<Self::Output, Self::Error>
                {
                    use recode::Decoder;

                    #( #field_exprs; )*

                    Ok(Self {
                        #(#field_names), *
                    })
                }
            }
        });
    }
}
