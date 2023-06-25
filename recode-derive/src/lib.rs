mod decoder;
mod field;

use darling::{FromDeriveInput, ToTokens};
use proc_macro::TokenStream;

macro_rules! emit_impl_or_error {
    ($m:ty [$i:ident]) => {
        match <$m>::from_derive_input(&syn::parse_macro_input!($i)) {
            | Ok(val) => val.into_token_stream(),
            | Err(err) => err.write_errors(),
        }
    };
}

#[proc_macro_derive(Decoder, attributes(decoder, skip, skip_if, map))]
pub fn derive_decoder(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(decoder::Decoder[input]).into()
}
