use super::Buffer;
use crate::util::EncoderExt;
use crate::{Decoder, Encoder, Error};

macro_rules! define_encoding {
    ($name:ident; doc: $d:literal; validate: |$i:ident| $v:stmt) => {
        #[derive(Debug, Clone, Default)]
        #[doc = $d]
        pub struct $name<L = crate::util::Remaining>(Buffer<L>);

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
                Ok(item.0.encode_to(buf)?)
            }

            #[inline]
            fn size_of(item: &Self, buf: &B) -> usize {
                item.0.size(buf)
            }
        }

        impl<L> std::ops::Deref for $name<L> {
            type Target = str;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
            }
        }

        impl<L> From<&'static str> for $name<L> {
            #[inline(always)]
            fn from(value: &'static str) -> Self {
                Self(Buffer::from_static(value.as_bytes()))
            }
        }
    };
}

define_encoding! {
    Ascii;
    doc: "A type that represents a text encoded as ASCII";
    validate: |inner| {
        if !inner.as_ref().is_ascii() {
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
