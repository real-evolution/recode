use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(skip, skip_if, map), supports(struct_named))]
pub(super) struct Decoder {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), DecoderField>,
    error: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
}

#[derive(Clone, Debug, darling::FromField)]
#[darling(attributes(decoder))]
struct DecoderField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    skip: bool,
    skip_if: Option<syn::Expr>,
    map: Option<syn::Expr>,
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
        let field_declare_stmts =
            fields.iter().map(|f| f.to_declare_statement(&buffer_name));

        tokens.extend(quote! {
            impl #imp ValueType for #ident #ty #wher {
                type Output = Self;
                type Error = #error;

                fn decode<B: recode::bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
                    #(#field_declare_stmts)*

                    Ok(Self {
                        #(#field_names),*
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

    fn skip_if_expr(&self) -> syn::Expr {
        self.skip_if
            .clone()
            .unwrap_or(syn::Expr::parse.parse2(quote!(false)).unwrap())
    }

    fn map_expr(&self) -> syn::Expr {
        self.map
            .clone()
            .unwrap_or(syn::Expr::parse.parse2(quote!(|i| i)).unwrap())
    }

    fn to_declare_statement(
        &self,
        buf_ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let ty = &self.ty;

        if self.skip {
            return quote! {
                let #ident = #ty::Default();
            };
        }

        let skip_if = self.skip_if_expr();
        let map = self.map_expr();

        quote::quote! {
            let #ident = if #skip_if {
                Default::default()
            } else {
                #ty::decode(#buf_ident).map(#map)?
            };
        }
    }
}
