use darling::util::Flag;

use crate::util::*;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(recode), supports(struct_named, struct_unit))]
pub(crate) struct Encoder {
    pub(crate) ident: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: darling::ast::Data<(), EncoderField>,
    #[darling(default)]
    pub(crate) encoder: EncoderOpts,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
#[darling(default)]
pub(crate) struct EncoderOpts {
    disable: Flag,
    error: Option<syn::Type>,
    buffer_type: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
    input_type: Option<syn::Type>,
    input_name: Option<syn::Ident>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(recode))]
pub(crate) struct EncoderField {
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) ty: syn::Type,
    #[darling(default)]
    pub(crate) encoder: EncoderFieldOpts,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
#[darling(default)]
pub(crate) struct EncoderFieldOpts {
    skip: Flag,
    skip_if: Option<syn::Expr>,
    map: Option<syn::Expr>,
    with: Option<syn::Type>,
}

impl darling::ToTokens for Encoder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        use syn::parse::{Parse, Parser};

        let Encoder {
            ident,
            generics,
            data,
            encoder:
                EncoderOpts {
                    disable,
                    error,
                    buffer_type,
                    buffer_name,
                    input_type,
                    input_name,
                },
        } = self;

        if disable.is_present() {
            return;
        }

        let mut generics = OwnedGenerics::new(generics.clone());

        let input_type = input_type
            .clone()
            .unwrap_or(syn::Type::Verbatim(quote!(Self)));
        let input_name =
            input_name.clone().unwrap_or(quote::format_ident!("input"));

        let error = error.clone().unwrap_or(box_type());
        let buffer_name = buffer_name.clone().unwrap_or(default_buffer_name());
        let buffer_type = buffer_type.clone().unwrap_or_else(|| {
            generics.push_impl_param(
                syn::TypeParam::parse
                    .parse2(quote!(B: recode::bytes::BufMut))
                    .unwrap(),
            )
        });

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_stmts = fields
            .iter()
            .map(|&f| f.to_encode_stmt(&input_name, &buffer_name));

        let (imp, ty, wher) = generics.split_for_impl();

        tokens.extend(quote::quote! {
            impl #imp recode::Encoder<#buffer_type, #input_type> for #ident #ty #wher {
                type Error = #error;

                fn encode(
                    #input_name: &#input_type,
                    #buffer_name: &mut B,
                ) -> Result<(), Self::Error> {
                    use recode::Encoder;

                    #( #field_stmts )*

                    Ok(())
                }
            }

        });
    }
}

impl EncoderField {
    pub(crate) fn to_encode_stmt(
        &self,
        input_ident: &syn::Ident,
        buf_ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        use quote::quote;

        let EncoderField {
            ident,
            ty,
            encoder:
                EncoderFieldOpts {
                    skip,
                    skip_if,
                    map,
                    with,
                },
        } = self;

        if skip.is_present() {
            return Default::default();
        }

        let with = with.as_ref().unwrap_or(ty);
        let input = map
            .as_ref()
            .map(|m| quote! (((#m)(#input_ident.#ident))))
            .unwrap_or(quote! (#input_ident.#ident));
        let stmt = quote! {
            <#with as recode::Encoder<_, #ty>>::encode(&#input, #buf_ident)?;
        };

        skip_if
            .as_ref()
            .map(|s| quote::quote! (if !(#s) { #stmt }))
            .unwrap_or(stmt)
    }
}
