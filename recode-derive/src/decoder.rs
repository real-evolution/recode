use darling::util::Flag;
use proc_macro2::TokenStream;

use crate::util::*;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(recode), supports(struct_named, struct_unit))]
pub(crate) struct Decoder {
    pub(crate) ident: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: darling::ast::Data<(), DecoderField>,
    #[darling(default)]
    pub(crate) decoder: DecoderOpts,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
#[darling(default)]
pub(crate) struct DecoderOpts {
    disable: Flag,
    output_type: Option<syn::Type>,
    error: Option<syn::Type>,
    buffer_type: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(recode))]
pub(crate) struct DecoderField {
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) ty: syn::Type,
    #[darling(default)]
    pub(crate) decoder: DecoderFieldOpts,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
#[darling(default)]
pub(crate) struct DecoderFieldOpts {
    skip: Flag,
    skip_if: Option<syn::Expr>,
    map: Option<syn::Expr>,
    with: Option<syn::Type>,
    validate: Option<syn::Expr>,
}

impl darling::ToTokens for Decoder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::quote;
        use syn::parse::{Parse, Parser};

        let Decoder {
            ident,
            generics,
            data,
            decoder:
                DecoderOpts {
                    disable,
                    output_type,
                    error,
                    buffer_type,
                    buffer_name,
                },
        } = self;

        if disable.is_present() {
            return;
        }

        let mut generics = OwnedGenerics::new(generics.clone());

        let output_type = output_type
            .clone()
            .unwrap_or(syn::Type::Verbatim(quote!(Self)));
        let error = error.clone().unwrap_or(box_type());
        let buffer_name = buffer_name.clone().unwrap_or(default_buffer_name());
        let buffer_type = buffer_type.clone().unwrap_or(
            generics.push_impl_param(
                syn::TypeParam::parse
                    .parse2(quote!(B: recode::bytes::Buf))
                    .unwrap(),
            ),
        );

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_names = fields.iter().map(|f| f.ident());
        let field_exprs =
            fields.iter().map(|&f| f.to_decode_stmt(&buffer_name));

        let (imp, ty, wher) = generics.split_for_impl();

        tokens.extend(quote::quote! {
            impl #imp recode::Decoder<#buffer_type, #output_type> for #ident #ty #wher {
                type Error = #error;

                fn decode(#buffer_name: &mut B) -> Result<#output_type, Self::Error> {
                    use recode::Decoder;

                    #( #field_exprs )*

                    Ok(#output_type {
                        #(#field_names), *
                    })
                }
            }
        });
    }
}

impl DecoderField {
    fn ident(&self) -> &syn::Ident {
        self.ident
            .as_ref()
            .expect("only named fields are currently supported")
    }

    fn to_decode_stmt(&self, buf_ident: &syn::Ident) -> TokenStream {
        use quote::quote;

        let DecoderField {
            ident,
            ty,
            decoder:
                DecoderFieldOpts {
                    skip,
                    skip_if,
                    map,
                    with,
                    validate,
                },
        } = self;

        if skip.is_present() {
            return quote::quote! ( let #ident = Default::default(); );
        }

        let with = with.as_ref().unwrap_or(ty);
        let map = map
            .as_ref()
            .map(|m| quote!(.map(#m)))
            .unwrap_or(TokenStream::new());
        let validate = validate
            .as_ref()
            .map(|v| quote!((#v)(&#ident, #buf_ident)?;))
            .unwrap_or(TokenStream::new());

        if let Some(ref skip_if) = skip_if {
            quote::quote! {
                let #ident = if #skip_if {
                    Default::default()
                } else {
                    <#with as recode::Decoder<_, #ty>>::decode(#buf_ident) #map ?
                };

                #validate
            }
        } else {
            quote::quote! {
                let #ident = <#with as recode::Decoder<_, #ty>>::decode(#buf_ident) #map ?;

                #validate
            }
        }
    }
}
