use bytes::BytesMut;

/// A to represent a length type with zero length.
///
/// This type is useful for encoding/decoding types that have no length prefix,
/// and should either:
/// - decode as the rest of the buffer.
/// - encode with no length prefix
#[derive(Debug, Clone, Copy, Default)]
pub struct Remaining;

impl crate::Decoder<usize> for Remaining {
    type Error = crate::Error;

    #[inline]
    fn decode(buf: &mut BytesMut) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }
}

impl crate::RawDecoder<usize> for Remaining {
    type Error = crate::Error;

    #[inline]
    fn raw_decode<'a>(buf: &'a [u8]) -> Result<(usize, usize), Self::Error>
    where
        usize: 'a,
    {
        Ok((buf.len(), 0))
    }
}

impl crate::Encoder<usize> for Remaining {
    type Error = crate::Error;

    #[inline]
    fn encode(_input: &usize, _buf: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn size_of(_input: &usize) -> usize {
        0
    }
}
