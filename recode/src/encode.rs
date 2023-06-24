use bytes::BufMut;

/// A trait to be implemented by types that encode [`Encoder::Input`] values.
pub trait Encoder {
    /// The type of the data this encoder is capable of encoding.
    type Input;

    /// Encodes the given input into the output buffer.
    ///
    /// This method does not return a result because calls to
    /// [`Encoder::encode`] never do.
    ///
    /// # Arguments
    /// * `input` - The input to encode.
    /// * `buf` - The output buffer to write the encoded input to.
    fn encode<B: BufMut>(&self, input: Self::Input, buf: &mut B);
}
