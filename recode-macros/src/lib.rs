mod decoder;

use darling::{FromDeriveInput, ToTokens};
use proc_macro::TokenStream;

macro_rules! parse_derive_input {
    ($input:ident => $parser:ty) => {{
        let input = syn::parse_macro_input!($input as syn::DeriveInput);

        let ts: TokenStream = match <$parser>::from_derive_input(&input) {
            | Ok(tokens) => tokens,
            | Err(err) => return err.write_errors().into(),
        }
        .into_token_stream()
        .into();

        ts
    }};
}

macro_rules! define_proc_macro {
    ($trait:ident with $impl:ty [ $($attr:ident),* ]) => {
        paste::paste! {
            #[proc_macro_error::proc_macro_error]
            #[proc_macro_derive($trait, attributes($($attr),*))]
            pub fn [<derive_ $trait:snake>](input: TokenStream) -> TokenStream {
                parse_derive_input!(input => $impl).into()
            }
        }
    };

    ($trait:ident [ $($attr:ident),* ]) => {
        paste::paste! {
            define_proc_macro!($trait with [<$trait:snake>]::$trait [ $($attr),* ]);
        }
    };
}

define_proc_macro!(Decoder[]);
