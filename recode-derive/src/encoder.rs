use darling::util::Flag;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(recode), supports(struct_named))]
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
    #[darling(default = "Flag::present")]
    enable: Flag,
    #[darling(default = r#"crate::util::default_error_type"#)]
    error: Option<syn::Type>,
    #[darling(default = r#"crate::util::default_buffer_name"#)]
    buffer_name: Option<syn::Ident>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(recode))]
pub(crate) struct EncoderField {
    pub(crate) ident: Option<syn::Ident>,
    #[darling(default)]
    pub(crate) encoder: EncoderFieldOpts,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
#[darling(default)]
pub(crate) struct EncoderFieldOpts {
    skip: Flag,
    skip_if: Option<syn::Expr>,
    map: Option<syn::Expr>,
}

impl darling::ToTokens for Encoder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Encoder {
            ident,
            generics,
            data,
            encoder:
                EncoderOpts {
                    enable,
                    error,
                    buffer_name,
                },
        } = self;

        if !enable.is_present() {
            return;
        }

        let (imp, ty, wher) = generics.split_for_impl();
        let error = error.as_ref().unwrap();
        let buffer_name = buffer_name.as_ref().unwrap();

        let fields: Vec<_> = data
            .as_ref()
            .take_struct()
            .expect("only structs are supported")
            .fields;

        let field_stmts = fields.iter().map(|&f| f.to_encode_stmt(buffer_name));

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

impl EncoderField {
    pub(crate) fn to_encode_stmt(
        &self,
        buf_ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let EncoderField {
            ident,
            encoder: EncoderFieldOpts { skip, skip_if, map },
        } = self;

        if skip.is_present() {
            return Default::default();
        }

        let mapped_encode = if let Some(ref map) = map {
            quote::quote! {
                ((#map)(#ident)).encode(#buf_ident)?;
            }
        } else {
            quote::quote! {
                #ident.encode(#buf_ident)?;
            }
        };

        if let Some(ref skip_if) = skip_if {
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
