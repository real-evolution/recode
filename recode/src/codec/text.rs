use super::LengthTraits;
use crate::{Decoder, Error};

/// A type alias for ASCII-encoded [`Text`].
pub type AsciiText<L = ()> = Text<Ascii, L>;

/// A type alias for UTF8-encoded [`Text`].
pub type Utf8Text<L = ()> = Text<Utf8, L>;

/// A marker type to indicate that [`Text`] object is encoded/decoded as ASCII.
#[derive(Debug)]
pub struct Ascii;

/// A marker type to indicate that [`Text`] object is encoded/decoded as UTf-8.
#[derive(Debug)]
pub struct Utf8;

/// A wrapper type for textual data.
///
/// This type is a wrapper type for textual data. It is used to allow encoding
/// and decoding of textual data in a generic way.
///
/// # Type parameters
/// - `C`: Text encoding/decoding marker.
/// - `L`: If not [`()`], it should be a numerical type that implements
/// [`super::length::LengthTraits`] that represent the length prefix of the text.
///
#[derive(Debug, Clone)]
pub struct Text<C = Utf8, L = ()> {
    inner: bytes::Bytes,
    _marker: std::marker::PhantomData<(C, L)>,
}

impl<C, L> Text<C, L> {
    /// Creates a new [`Text`] object from a [`bytes::Bytes`] instance.
    ///
    /// # Parameters
    /// - `bytes`: The bytes to wrap.
    ///
    /// # Returns
    /// A new [`Text`] object.
    fn from_bytes(bytes: bytes::Bytes) -> Self {
        Self {
            inner: bytes,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Decoder for Ascii {
    type Output = bytes::Bytes;
    type Error = TextError;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let buf = buf.copy_to_bytes(buf.remaining());

        if !buf.is_ascii() {
            return Err(TextError::Ascii("invalid ascii data"))?;
        }

        Ok(buf)
    }
}

impl Decoder for Utf8 {
    type Output = bytes::Bytes;
    type Error = TextError;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let buf = buf.copy_to_bytes(buf.remaining());

        _ = std::str::from_utf8(buf.as_ref())?;

        Ok(buf)
    }
}

impl<C> Decoder for Text<C>
where
    C: Decoder<Output = bytes::Bytes>,
    Error: From<C::Error>,
{
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let bytes = C::decode(buf)?;

        Ok(Self::from_bytes(bytes))
    }
}

impl<C, L> Decoder for Text<C, L>
where
    C: Decoder<Output = bytes::Bytes>,
    L: LengthTraits,
    Error: From<C::Error>
        + From<<L as Decoder>::Error>
        + From<<usize as TryFrom<L>>::Error>,
    usize: TryFrom<L>,
{
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        if buf.remaining() < L::BYTE_COUNT {
            return Err(Error::BytesNeeded {
                needed: L::BYTE_COUNT - buf.remaining(),
                full_len: L::BYTE_COUNT,
                available: buf.remaining(),
            });
        }

        let len = L::decode_usize(buf)?;
        let buf = C::decode(&mut buf.copy_to_bytes(len))?;

        Ok(Self::from_bytes(buf))
    }
}

impl<C, L> AsRef<[u8]> for Text<C, L> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl<C, L> AsRef<str> for Text<C, L> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.inner.as_ref()) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("ascii error: {0}")]
    Ascii(&'static str),
}
