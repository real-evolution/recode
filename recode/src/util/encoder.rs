use crate::Encoder;

/// An extension trait for [`Encoder`].
pub trait EncoderExt<B>: Encoder<B> + Sized {
    /// Encodes `self` into `buf`.
    ///
    /// This is just an alias to [`Encoder::encode`], with `self` passed in
    /// the place of `item`.
    #[inline(always)]
    fn encode_to(&self, buf: &mut B) -> Result<(), Self::Error> {
        <Self as Encoder<B>>::encode(self, buf)
    }

    /// Returns the number of bytes required to encode `self`.
    fn size(&self, buf: &B) -> usize {
        <Self as Encoder<B>>::size_of(self, buf)
    }
}

impl<T, B> EncoderExt<B> for T where T: Encoder<B> + Sized {}
