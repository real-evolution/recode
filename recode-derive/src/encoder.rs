use darling::{util::Flag, ToTokens};

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
    pub(crate) disable: Flag,
    pub(crate) error: Option<syn::Type>,
    pub(crate) buffer_name: Option<syn::Ident>,
    pub(crate) input_type: Option<syn::Type>,
    pub(crate) input_name: Option<syn::Ident>,
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
    pub(crate) skip: Flag,
    pub(crate) skip_if: Option<syn::Expr>,
    pub(crate) map: Option<syn::Expr>,
    pub(crate) with: Option<syn::Type>,
    pub(crate) size: Option<syn::Expr>,
}

impl darling::ToTokens for Encoder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let Encoder {
            ident,
            generics,
            data,
            encoder:
                EncoderOpts {
                    disable,
                    error,
                    buffer_name,
                    input_type,
                    input_name,
                },
        } = self;

        if disable.is_present() {
            return;
        }

        let input_type = input_type
            .clone()
            .unwrap_or(syn::Type::Verbatim(quote!(Self)));
        let input_name =
            input_name.clone().unwrap_or(quote::format_ident!("input"));

        let error = error.clone().unwrap_or(box_type());
        let buffer_name = buffer_name.clone().unwrap_or(default_buffer_name());

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_stmts = fields
            .iter()
            .map(|&f| f.to_encode_stmt(&input_name, &buffer_name));

        let size_exprs = fields.iter().map(|&f| f.to_size_expr(&input_name));

        let (imp, ty, wher) = generics.split_for_impl();

        tokens.extend(quote::quote! {
            impl #imp recode::Encoder<#input_type> for #ident #ty #wher {
                type Error = #error;

                fn encode(
                    #input_name: &#input_type,
                    #buffer_name: &mut recode::bytes::BytesMut,
                ) -> Result<(), Self::Error> {
                    use recode::Encoder;

                    #( #field_stmts )*

                    Ok(())
                }

                fn size_of(#input_name: &#input_type) -> usize {
                    0 #( + #size_exprs )*
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
                    size: _,
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
            <#with as recode::Encoder<#ty>>::encode(&#input, #buf_ident)?;
        };

        skip_if
            .as_ref()
            .map(|s| quote::quote! (if !(#s) { #stmt }))
            .unwrap_or(stmt)
    }

    pub(crate) fn to_size_expr(
        &self,
        input_ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        use quote::quote;

        if let Some(ref expr) = self.encoder.size {
            return expr.to_token_stream();
        }

        let ident = self.ident.as_ref();
        let ty = &self.ty;
        let with = self.encoder.with.as_ref().unwrap_or(&self.ty);

        quote! {
            <#with as recode::Encoder<#ty>>::size_of(&#input_ident.#ident)
        }
    }
}
