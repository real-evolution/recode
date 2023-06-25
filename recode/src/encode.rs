use bytes::BufMut;

/// A trait to be implemented by types that encode [`Encoder::Input`] values.
pub trait Encoder {
    /// The type of error that can occur if encoding fails.
    type Error;

    /// Encodes the given input into the output buffer.
    ///
    /// This method does not return a result because calls to
    /// [`Encoder::encode`] never do.
    ///
    /// # Arguments
    /// * `buf` - The output buffer to write the encoded input to.
    fn encode<B: BufMut>(
        &self,
        buf: &mut B,
    ) -> Result<(), Self::Error>;
}
