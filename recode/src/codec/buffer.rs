use std::ops::Deref;

use crate::{bytes::Buf, Decoder, Encoder, Error};

/// A type alias for a [`Buffer`] without a length prefix.
pub type UnprefixedBuffer = Buffer<()>;

/// A wrapper type for consecutive bytes.
///
/// # Type Parameters
/// - `L`: If not [`()`], it should be a numerical type that implements
/// [`Decoder<Output = L`] and [`TryFrom<usize>`]` which represents the length
/// prefix of the buffer.
#[derive(Debug, Clone)]
pub struct Buffer<L = crate::util::Remaining> {
    inner: bytes::Bytes,
    _marker: std::marker::PhantomData<L>,
}

impl<L> Buffer<L> {
    /// Creates a new [`Buffer<L>`] object from a [`bytes::Bytes`] instance.
    ///
    /// # Parameters
    /// - `bytes`: The [`bytes::Bytes`] instance to wrap.
    ///
    /// # Returns
    /// A new [`Buffer<L>`] object.
    pub fn new(bytes: bytes::Bytes) -> Self {
        Self {
            inner: bytes,
            _marker: Default::default(),
        }
    }

    /// Creates a new [`Buffer<L>`] object from a [`&'static [u8]`].
    ///
    /// This is a shorthand for
    /// [`Buffer::new`]`(`[`bytes::Bytes::from_static`]`(`[`&'static [u8]`]`))`.
    ///
    /// # Parameters
    /// - `bytes`: The [`&'static [u8]`] instance to wrap.
    ///
    /// # Returns
    /// A new [`Buffer<L>`] object.
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self::new(bytes.into())
    }
}

impl<B, L> Decoder<B> for Buffer<L>
where
    B: Buf,
    L: Decoder<B, usize>,
    Error: From<<L as Decoder<B, usize>>::Error>,
{
    type Error = Error;

    fn decode(buf: &mut B) -> Result<Self, Self::Error> {
        let len = L::decode(buf)?;

        if buf.remaining() < len {
            return Err(Error::BytesNeeded {
                needed: len - buf.remaining(),
                full_len: len,
                available: buf.remaining(),
            });
        }

        Ok(Self::new(buf.copy_to_bytes(len)))
    }
}

impl<L> Encoder for Buffer<L>
where
    L: Encoder<Input = L> + TryFrom<usize>,
    Error: From<<L as Encoder>::Error> + From<<L as TryFrom<usize>>::Error>,
{
    type Error = Error;
    type Input = Self;

    fn encode<B: bytes::BufMut>(
        input: &Self::Input,
        buf: &mut B,
    ) -> Result<(), Self::Error> {
        let len = L::try_from(input.inner.len())?;

        L::encode(&len, buf)?;
        buf.put(input.inner.as_ref());

        Ok(())
    }
}

impl<L> Deref for Buffer<L> {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<L> From<&'static [u8]> for Buffer<L> {
    fn from(value: &'static [u8]) -> Self {
        Self::from_static(value)
    }
}
