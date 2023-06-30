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
    #[darling(default = "crate::util::default_error_type")]
    error: Option<syn::Type>,
    #[darling(default = "crate::util::default_buffer_name")]
    buffer_name: Option<syn::Ident>,
    #[darling(default = "default_input_type")]
    input_type: Option<syn::Type>,
    #[darling(default = "default_input_name")]
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
        let Encoder {
            ident,
            generics,
            data,
            encoder:
                EncoderOpts {
                    enable,
                    error,
                    buffer_name,
                    input_type,
                    input_name,
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

        let field_stmts = fields.iter().map(|&f| {
            f.to_encode_stmt(input_name.as_ref().unwrap(), buffer_name)
        });

        tokens.extend(quote::quote! {
            impl #imp recode::Encoder for #ident #ty #wher {
                type Input = #input_type;
                type Error = #error;

                fn encode<B: recode::bytes::BufMut>(
                    #input_name: &Self::Input,
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
            <#with as recode::Encoder>::encode(&#input, #buf_ident)?;
        };

        skip_if
            .as_ref()
            .map(|s| quote::quote! (if !(#s) { #stmt }))
            .unwrap_or( stmt)
    }
}

fn default_input_type() -> Option<syn::Type> {
    use syn::parse::{Parse, Parser};

    Some(syn::Type::parse.parse2(quote::quote!(Self)).unwrap())
}

fn default_input_name() -> Option<syn::Ident> {
    Some(quote::format_ident!("input"))
}
