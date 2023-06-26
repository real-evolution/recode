use proc_macro2::TokenStream;

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
    pub(crate) fn ident(&self) -> &syn::Ident {
        self.ident
            .as_ref()
            .expect("only named fields are currently supported")
    }

    pub(crate) fn to_decode_stmt(&self, buf_ident: &syn::Ident) -> TokenStream {
        let ident = self.ident();
        let ty = &self.ty;

        if self.skip {
            return quote::quote! ( #ident: Default::default() );
        }

        let map = if let Some(ref map) = self.map {
            quote::quote!(.map(#map))
        } else {
            TokenStream::new()
        };

        if let Some(ref skip_if) = self.skip_if {
            quote::quote! {
                let #ident = #skip_if {
                    Default::default()
                } else {
                    <#ty as recode::Decoder>::decode(#buf_ident) #map ?
                }
            }
        } else {
            quote::quote! {
                let #ident = <#ty as recode::Decoder>::decode(#buf_ident) #map ?
            }
        }
    }

    pub(crate) fn to_encode_stmt(&self, buf_ident: &syn::Ident) -> TokenStream {
        let ident = self.ident();

        if self.skip {
            return TokenStream::new();
        }

        let mapped_encode = if let Some(ref map) = self.map {
            quote::quote! {
                ((#map)(#ident)).encode(#buf_ident)?;
            }
        } else {
            quote::quote! {
                #ident.encode(#buf_ident)?;
            }
        };

        if let Some(ref skip_if) = self.skip_if {
            quote::quote! {
                if !(#skip_if) {
                    #mapped_encode
                }
            }
        } else {
            mapped_encode
        }
    }
}
