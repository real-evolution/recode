use syn::parse::{Parse, Parser};

#[derive(Debug, darling::FromField)]
#[darling(attributes(decoder))]
pub(crate) struct RecodeField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    skip: bool,
    skip_if: Option<syn::Expr>,
    map: Option<syn::Expr>,
}

impl RecodeField {
    fn ident(&self) -> &syn::Ident {
        self.ident
            .as_ref()
            .expect("only named fields are currently supported")
    }

    fn map_expr(&self) -> syn::Expr {
        self.map
            .clone()
            .unwrap_or(syn::Expr::parse.parse2(quote::quote!(|i| i)).unwrap())
    }

    pub(crate) fn to_decode_expr(
        &self,
        buf_ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let ty = &self.ty;

        if self.skip {
            return quote::quote! ( #ident: Default::default() );
        }

        let map = self.map_expr();

        if let Some(ref skip_if) = self.skip_if {
            quote::quote! {
                #ident: #skip_if {
                    Default::default()
                } else {
                    <#ty as recode::Decoder>::decode(#buf_ident).map(#map)?
                }
            }
        } else {
            quote::quote! {
                #ident:  <#ty as recode::Decoder>::decode(#buf_ident).map(#map)?
            }
        }
    }
}
