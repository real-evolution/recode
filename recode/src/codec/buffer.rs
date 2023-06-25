use std::ops::Deref;

use crate::{Decoder, Encoder, Error};

/// A type alias for a [`Buffer`] without a length prefix.
pub type UnprefixedBuffer = Buffer<()>;

/// A wrapper type for consecutive bytes.
///
/// # Type Parameters
/// - `L`: If not [`()`], it should be a numerical type that implements
/// [`Decoder<Output = L`] and [`TryFrom<usize>`]` which represents the length
/// prefix of the buffer.
#[derive(Debug, Clone)]
pub struct Buffer<L = ()> {
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
}

impl Decoder for Buffer {
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let buf = buf.copy_to_bytes(buf.remaining());

        Ok(Self::new(buf))
    }
}

impl Encoder for Buffer {
    type Error = Error;

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
        buf.put(self.inner.as_ref());

        Ok(())
    }
}

impl<L> Decoder for Buffer<L>
where
    L: Decoder,
    Error: From<<L as Decoder>::Error>
        + From<<usize as TryFrom<L::Output>>::Error>,
    usize: TryFrom<L::Output>,
{
    type Output = Self;
    type Error = Error;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        let len: usize = L::decode(buf)?.try_into()?;

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
    L: Encoder + TryFrom<usize>,
    Error: From<<L as Encoder>::Error> + From<<L as TryFrom<usize>>::Error>,
{
    type Error = Error;

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
        let len = L::try_from(self.inner.len())?;

        len.encode(buf)?;
        buf.put(self.inner.as_ref());

        Ok(())
    }
}

impl<L> Deref for Buffer<L> {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
