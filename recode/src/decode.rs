use bytes::Buf;

/// A trait for types that can decode values of type [`Decoder::Output`] from
/// a bytes buffer.
pub trait Decoder {
    /// The type of the value that will be decoded.
    type Output;

    /// The type of error that can occur if decoding fails.
    type Error;

    /// Decodes a value from the given buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer to decode the value from.
    ///
    /// # Returns
    /// The decoded value.
    fn decode<B: Buf>(buf: &mut B) -> Result<Self::Output, Self::Error>;
}
