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
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {

    use crate as recode;
    use crate::Encoder;

    #[test]
    fn basic_test() {
        #[derive(Encoder)]
        #[encoder(buffer_name = "buf", error = "crate::Error")]
        struct TestType {
            age: u32,
            salary: u64,
            first_name: crate::codec::Ascii<u8>,
            last_name: crate::codec::Utf8<u16>,
            image: crate::codec::Buffer<u32>,
        }

        const IMAGE_BUF: [u8; 32] = [
            0xba, 0x3e, 0x9d, 0x6b, 0xae, 0xf5, 0x91, 0xcc, 0xe2, 0xf0, 0xcb,
            0x4f, 0xbb, 0x5b, 0x2b, 0xbe, 0xa7, 0xf4, 0x9d, 0xfb, 0x87, 0x43,
            0x0e, 0xdf, 0x30, 0x6f, 0x7d, 0x6e, 0x22, 0xab, 0xcc, 0x47,
        ];

        let test_item = TestType {
            age: 0x01234567,
            salary: 0x1122334455667788,
            first_name: "ayman".into(),
            last_name: "القاضي".into(),
            image: IMAGE_BUF.as_ref().into(),
        };

        let mut buf = bytes::BytesMut::new();
        test_item.encode(&mut buf).unwrap();

        const BUF: [u8; 68] = [
            // age (4 bytes)
            0x01, 0x23, 0x45, 0x67, // salary (8 bytes)
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            // first name (1 bytes prefixed ascii; 6 bytes)
            0x05, b'a', b'y', b'm', b'a', b'n',
            // last name (2 bytes prefixed utf8; 6 bytes)
            0x00, 0x0c, 0xd8, 0xa7, 0xd9, 0x84, 0xd9, 0x82, 0xd8, 0xa7, 0xd8,
            0xb6, 0xd9, 0x8a,
            // image (4 bytes prefixed buffer; 36 bytes)
            0x00, 0x00, 0x00, 0x20, 0xba, 0x3e, 0x9d, 0x6b, 0xae, 0xf5, 0x91,
            0xcc, 0xe2, 0xf0, 0xcb, 0x4f, 0xbb, 0x5b, 0x2b, 0xbe, 0xa7, 0xf4,
            0x9d, 0xfb, 0x87, 0x43, 0x0e, 0xdf, 0x30, 0x6f, 0x7d, 0x6e, 0x22,
            0xab, 0xcc, 0x47,
        ];

        assert_eq!(buf, BUF.as_ref());
    }
}
