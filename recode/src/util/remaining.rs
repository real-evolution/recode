/// A to represent a length type with zero length.
///
/// This type is useful for encoding/decoding types that have no length prefix,
/// and should either:
/// - decode as the rest of the buffer.
/// - encode with no length prefix
#[derive(Debug, Clone, Copy, Default)]
pub struct Remaining;

impl<B> crate::Decoder<B, usize> for Remaining
where
    B: crate::bytes::Buf,
{
    type Error = crate::Error;

    fn decode(buf: &mut B) -> Result<usize, Self::Error> {
        Ok(buf.remaining())
    }
}

impl crate::Encoder for Remaining {
    type Input = Self;
    type Error = crate::Error;

    fn encode<B: bytes::BufMut>(
        _input: &Self::Input,
        _buf: &mut B,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}