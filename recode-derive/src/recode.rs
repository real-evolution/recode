#![allow(dead_code)]

use darling::util::Flag;

use crate::{decoder, encoder};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(recode), supports(struct_named, struct_unit))]
pub(crate) struct Recode {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), RecodeField>,
    error: Option<syn::Type>,
    buffer_name: Option<syn::Ident>,
    #[darling(default)]
    decoder: decoder::DecoderOpts,
    #[darling(default)]
    encoder: encoder::EncoderOpts,
}

#[derive(Debug, Clone, darling::FromField)]
#[darling(attributes(recode))]
struct RecodeField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    skip: Flag,
    skip_if: Option<syn::Expr>,
    with: Option<syn::Type>,
    validate: Option<syn::Expr>,
    #[darling(default)]
    decoder: decoder::DecoderFieldOpts,
    #[darling(default)]
    encoder: encoder::EncoderFieldOpts,
}

impl darling::ToTokens for Recode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        decoder::Decoder {
            ident: self.ident.clone(),
            generics: self.generics.clone(),
            data: self.get_decoder_data(),
            decoder: decoder::DecoderOpts {
                error: self.decoder.error.clone().or(self.error.clone()),
                buffer_name: self
                    .decoder
                    .buffer_name
                    .clone()
                    .or(self.buffer_name.clone()),
                ..self.decoder.clone()
            },
        }
        .to_tokens(tokens);

        encoder::Encoder {
            ident: self.ident.clone(),
            generics: self.generics.clone(),
            data: self.get_encoder_data(),
            encoder: encoder::EncoderOpts {
                error: self.encoder.error.clone().or(self.error.clone()),
                buffer_name: self
                    .encoder
                    .buffer_name
                    .clone()
                    .or(self.buffer_name.clone()),
                ..self.encoder.clone()
            },
        }
        .to_tokens(tokens);
    }
}

impl Recode {
    #[inline(always)]
    fn get_decoder_data(
        &self,
    ) -> darling::ast::Data<(), decoder::DecoderField> {
        self.data
            .clone()
            .map_struct_fields(|f| decoder::DecoderField {
                ident: f.ident,
                ty: f.ty,
                decoder: decoder::DecoderFieldOpts {
                    skip: if f.skip.is_present() {
                        Flag::present()
                    } else {
                        f.encoder.skip
                    },
                    skip_if: f.skip_if.or(f.encoder.skip_if),
                    with: f.decoder.with.or(f.with),
                    validate: f.decoder.validate.or(f.validate),
                    ..f.decoder
                },
            })
    }

    #[inline(always)]
    fn get_encoder_data(
        &self,
    ) -> darling::ast::Data<(), encoder::EncoderField> {
        self.data
            .clone()
            .map_struct_fields(|f| encoder::EncoderField {
                ident: f.ident,
                ty: f.ty,
                encoder: encoder::EncoderFieldOpts {
                    skip: if f.skip.is_present() {
                        Flag::present()
                    } else {
                        f.encoder.skip
                    },
                    skip_if: f.skip_if.or(f.encoder.skip_if),
                    with: f.encoder.with.or(f.with),
                    validate: f.encoder.validate.or(f.validate),
                    ..f.encoder
                },
            })
    }
}
