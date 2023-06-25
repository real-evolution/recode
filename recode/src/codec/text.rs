use super::Buffer;
use crate::{Decoder, Encoder, Error};

/// A marker type to indicate that [`Text`] object is encoded/decoded as ASCII.
#[derive(Debug)]
pub struct Ascii<L = ()>(Buffer<L>);

/// A marker type to indicate that [`Text`] object is encoded/decoded as UTf-8.
#[derive(Debug)]
pub struct Utf8<L = ()>(Buffer<L>);

impl<L> Decoder for Ascii<L>
where
    Buffer<L>: Decoder<Output = Buffer<L>>,
    Error: From<<Buffer<L> as Decoder>::Error>,
{
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let inner = Buffer::<L>::decode(buf)?;

        if !inner.is_ascii() {
            return Err(TextError::Ascii("invalid ascii data"))?;
        }

        Ok(Self(inner))
    }
}

impl<L> Encoder for Ascii<L>
where
    Buffer<L>: Encoder,
    Error: From<<Buffer<L> as Encoder>::Error>,
{
    type Error = Error;

    #[inline(always)]
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
        Ok(self.0.encode(buf)?)
    }
}

impl<L> Decoder for Utf8<L>
where
    Buffer<L>: Decoder<Output = Buffer<L>>,
    Error: From<<Buffer<L> as Decoder>::Error>,
{
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let inner = Buffer::<L>::decode(buf)?;

        _ = std::str::from_utf8(inner.as_ref()).map_err(TextError::Utf8)?;

        Ok(Self(inner))
    }
}

impl<L> Encoder for Utf8<L>
where
    Buffer<L>: Encoder,
    Error: From<<Buffer<L> as Encoder>::Error>,
{
    type Error = Error;

    #[inline(always)]
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
        Ok(self.0.encode(buf)?)
    }
}

impl<L> std::ops::Deref for Ascii<L> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let buf = self.0.deref();

        unsafe { std::str::from_utf8_unchecked(buf.as_ref()) }
    }
}

impl<L> std::ops::Deref for Utf8<L> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let buf = self.0.deref();

        unsafe { std::str::from_utf8_unchecked(buf.as_ref()) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("ascii error: {0}")]
    Ascii(&'static str),
}
