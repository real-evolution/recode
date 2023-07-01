use super::Buffer;
use crate::{Decoder, Encoder, Error};

macro_rules! define_encoding {
    ($name:ident; doc: $d:literal; validate: |$i:ident| $v:stmt) => {
        #[derive(Debug)]
        #[doc = $d]
        pub struct $name<L = ()>(Buffer<L>);

        impl<B, L> Decoder<B> for $name<L>
        where
            Buffer<L>: Decoder<B>,
            Error: From<<Buffer<L> as Decoder<B>>::Error>,
        {
            type Error = Error;

            fn decode(buf: &mut B) -> Result<Self, Self::Error> {
                let $i = Buffer::<L>::decode(buf)?;

                $v

                Ok(Self($i))
            }
        }

        impl<B, L> Encoder<B> for $name<L>
        where
            Buffer<L>: Encoder<B>,
            Error: From<<Buffer<L> as Encoder<B>>::Error>,
        {
            type Error = Error;

            #[inline(always)]
            fn encode(item: &Self, buf: &mut B) -> Result<(), Self::Error> {
                Ok(Buffer::<L>::encode(&item.0, buf)?)
            }
        }

        impl<L> std::ops::Deref for $name<L> {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                let buf = self.0.deref();

                unsafe { std::str::from_utf8_unchecked(buf.as_ref()) }
            }
        }

        impl<L> From<&'static str> for $name<L> {
            fn from(value: &'static str) -> Self {
                let buf = Buffer::from_static(value.as_bytes());

                Self(buf)
            }
        }
    };
}

define_encoding! {
    Ascii;
    doc: "A type that represents a text encoded as ASCII";
    validate: |inner| {
        if !inner.is_ascii() {
            return Err(TextError::Ascii("invalid ascii data"))?;
        }
    }
}

define_encoding! {
    Utf8;
    doc: "A type that represents a text encoded as UTF-8";
    validate: |inner| {
        _ = std::str::from_utf8(inner.as_ref()).map_err(TextError::Utf8)?;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("ascii error: {0}")]
    Ascii(&'static str),
}
