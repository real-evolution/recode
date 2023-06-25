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

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate as recode;
    use crate::Decoder;

    #[test]
    fn basic_test() {
        #[derive(Decoder)]
        #[decoder(buffer_name = "buf")]
        struct TestType {
            age: u32,
            salary: u64,
            first_name: crate::codec::Ascii<u8>,
            last_name: crate::codec::Utf8<u16>,
            image: crate::codec::Buffer<u32>,
        }

        const BUF: [u8; 78] = [
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
            // leftover bytes (10 bytes ascii text)
            b't', b'o', b' ', b'b', b'e', b' ', b'l', b'e', b'f', b't',
        ];

        let mut bytes = bytes::Bytes::from_static(&BUF);
        let test = TestType::decode(&mut bytes).unwrap();

        assert_eq!(0x01234567, test.age);
        assert_eq!(0x1122334455667788, test.salary);
        assert_eq!("ayman", test.first_name.deref());
        assert_eq!("القاضي", test.last_name.deref());
        assert_eq!(
            &[
                0xba, 0x3e, 0x9d, 0x6b, 0xae, 0xf5, 0x91, 0xcc, 0xe2, 0xf0,
                0xcb, 0x4f, 0xbb, 0x5b, 0x2b, 0xbe, 0xa7, 0xf4, 0x9d, 0xfb,
                0x87, 0x43, 0x0e, 0xdf, 0x30, 0x6f, 0x7d, 0x6e, 0x22, 0xab,
                0xcc, 0x47,
            ],
            test.image.as_ref()
        );
        assert_eq!(bytes.as_ref(), b"to be left");
    }
}
