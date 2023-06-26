use syn::parse::{Parse, Parser};

use crate::field::RecodeField;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(decoder), supports(struct_named))]
pub(super) struct Encoder {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), RecodeField>,
    error: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
}

impl darling::ToTokens for Encoder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Encoder {
            ident,
            generics,
            data,
            error,
            buffer_name,
        } = self;

        let (imp, ty, wher) = generics.split_for_impl();

        let error = error.clone().unwrap_or(
            syn::Type::parse
                .parse2(quote::quote!(Box<dyn std::error::Error>))
                .unwrap(),
        );

        let buffer_name =
            buffer_name.clone().unwrap_or(quote::format_ident!("__buf"));

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_stmts =
            fields.iter().map(|&f| f.to_encode_stmt(&buffer_name));

        tokens.extend(quote::quote! {
            impl #imp recode::Encoder for #ident #ty #wher {
                type Error = #error;

                fn encode<B: recode::bytes::BufMut>(
                    &self,
                    #buffer_name: &mut B,
                ) -> Result<(), Self::Error> {
                    use recode::Encoder;

                    #( self.#field_stmts; )*

                    Ok(())
                }
            }

        });
    }
}
