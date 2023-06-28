#![allow(dead_code)]

use crate::{decoder, encoder};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
#[darling(attributes(recode), supports(struct_named))]
pub(crate) struct Recode {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), RecodeField>,
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
            decoder: self.decoder.clone(),
        }
        .to_tokens(tokens);

        encoder::Encoder {
            ident: self.ident.clone(),
            generics: self.generics.clone(),
            data: self.get_encoder_data(),
            encoder: self.encoder.clone(),
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
                decoder: f.decoder,
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
                encoder: f.encoder,
            })
    }
}
