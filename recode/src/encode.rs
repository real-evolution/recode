/// A trait to be implemented by types that encode [`Item`] values into a
/// buffer of type [`B`].
pub trait Encoder<B, Item = Self> {
    /// The type of error that can occur if encoding fails.
    type Error;

    /// Encodes the given input into the output buffer.
    ///
    /// This method does not return a result because calls to
    /// [`Encoder::encode`] never do.
    ///
    /// # Arguments
    /// * `item` - The input to encode.
    /// * `buf` - The output buffer to write the encoded input to.
    fn encode(item: &Item, buf: &mut B) -> Result<(), Self::Error>;

    /// Returns the number of bytes required to encode the given input.
    ///
    /// This method is useful for pre-allocating buffers that will be used for
    /// encoding.
    ///
    /// # Arguments
    /// * `item` - The input to encode.
    /// * `buf` - The output buffer used for encoding.
    ///
    /// # Returns
    /// The number of bytes required to encode the given input.
    fn size_of(item: &Item, buf: &B) -> usize;
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate as recode;
    use crate::{codec::LengthPrefixed, util::EncoderExt, Encoder};

    #[test]
    fn basic_test() {
        #[derive(Encoder)]
        #[recode(encoder(buffer_name = "buf", error = "crate::Error"))]
        struct TestType {
            age: u32,
            salary: u64,
            #[recode(encoder(with = "LengthPrefixed::<u8>"))]
            first_name: Bytes,
            #[recode(encoder(with = "LengthPrefixed::<u16>"))]
            last_name: Bytes,
            #[recode(encoder(with = "LengthPrefixed::<u32>"))]
            image: Bytes,
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

        assert_eq!(test_item.age.size(&buf), 4);
        assert_eq!(test_item.salary.size(&buf), 8);
        assert_eq!(
            LengthPrefixed::<u8>::size_of(&test_item.first_name, &buf),
            1 + 5
        );
        assert_eq!(
            LengthPrefixed::<u16>::size_of(&test_item.last_name, &buf),
            2 + 12
        );
        assert_eq!(
            LengthPrefixed::<u32>::size_of(&test_item.image, &buf),
            4 + 32
        );
        assert_eq!(test_item.size(&buf), 4 + 8 + (1 + 5) + (2 + 12) + (4 + 32));

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
