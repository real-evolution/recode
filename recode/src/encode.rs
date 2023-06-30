use bytes::BufMut;

/// A trait to be implemented by types that encode [`Encoder::Input`] values.
pub trait Encoder {
    /// The type of input that this encoder can encode.
    type Input;

    /// The type of error that can occur if encoding fails.
    type Error;

    /// Encodes the given input into the output buffer.
    ///
    /// This method does not return a result because calls to
    /// [`Encoder::encode`] never do.
    ///
    /// # Arguments
    /// * `input` - The input to encode.
    /// * `buf` - The output buffer to write the encoded input to.
    fn encode<B: BufMut>(
        input: &Self::Input,
        buf: &mut B,
    ) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use crate as recode;
    use crate::Encoder;

    #[test]
    fn basic_test() {
        #[derive(Encoder)]
        #[recode(encoder(buffer_name = "buf", error = "crate::Error"))]
        struct TestType {
            age: u32,
            salary: u64,
            first_name: crate::codec::Ascii<u8>,
            last_name: crate::codec::Utf8<u16>,
            image: crate::codec::Buffer<u32>,
        }

        const IMAGE_BUF: [u8; 32] = [
            0xBA, 0x3E, 0x9D, 0x6B, 0xAE, 0xF5, 0x91, 0xCC, 0xE2, 0xF0, 0xCB,
            0x4F, 0xBB, 0x5B, 0x2B, 0xBE, 0xA7, 0xF4, 0x9D, 0xFB, 0x87, 0x43,
            0x0E, 0xDF, 0x30, 0x6F, 0x7D, 0x6E, 0x22, 0xAB, 0xCC, 0x47,
        ];

        let test_item = TestType {
            age: 0x01234567,
            salary: 0x1122334455667788,
            first_name: "ayman".into(),
            last_name: "القاضي".into(),
            image: IMAGE_BUF.as_ref().into(),
        };

        let mut buf = bytes::BytesMut::new();
        TestType::encode(&test_item, &mut buf).unwrap();

        const BUF: [u8; 68] = [
            // age (4 bytes)
            0x01, 0x23, 0x45, 0x67, // salary (8 bytes)
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            // first name (1 bytes prefixed ascii; 6 bytes)
            0x05, b'a', b'y', b'm', b'a', b'n',
            // last name (2 bytes prefixed utf8; 6 bytes)
            0x00, 0x0C, 0xD8, 0xA7, 0xD9, 0x84, 0xD9, 0x82, 0xD8, 0xA7, 0xD8,
            0xB6, 0xD9, 0x8A,
            // image (4 bytes prefixed buffer; 36 bytes)
            0x00, 0x00, 0x00, 0x20, 0xBA, 0x3E, 0x9D, 0x6B, 0xAE, 0xF5, 0x91,
            0xCC, 0xE2, 0xF0, 0xCB, 0x4F, 0xBB, 0x5B, 0x2B, 0xBE, 0xA7, 0xF4,
            0x9D, 0xFB, 0x87, 0x43, 0x0E, 0xDF, 0x30, 0x6F, 0x7D, 0x6E, 0x22,
            0xAB, 0xCC, 0x47,
        ];

        assert_eq!(buf, BUF.as_ref());
    }
}
