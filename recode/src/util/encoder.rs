use bytes::BytesMut;

use crate::Encoder;

/// An extension trait for [`Encoder`](crate::Encoder).
pub trait EncoderExt: Encoder + Sized {
    /// Encodes `self` into `buf`.
    ///
    /// This is just an alias to [`Encoder::encode`], with `self` passed in
    /// the place of `item`.
    #[inline(always)]
    fn encode_to(&self, buf: &mut BytesMut) -> Result<(), Self::Error> {
        <Self as Encoder>::encode(self, buf)
    }

    /// Returns the number of bytes required to encode `self`.
    fn size(&self) -> usize {
        <Self as Encoder>::size_of(self)
    }
}

impl<T> EncoderExt for T where T: Encoder + Sized {}
