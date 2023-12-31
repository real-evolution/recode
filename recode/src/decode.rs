use bytes::BytesMut;

/// A trait for types that can decode values of type [`Decoder::Output`] from
/// a bytes buffer of type [`Bytes`](bytes::Bytes).
pub trait Decoder<Item = Self> {
    /// The type of error that can occur if decoding fails.
    type Error;

    /// Decodes a value from the given buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer to decode the value from.
    ///
    /// # Returns
    /// The decoded value.
    fn decode(buf: &mut BytesMut) -> Result<Item, Self::Error>;

    /// Returns `true` if the given buffer can be decoded.
    ///
    /// This is used to determine if the buffer has enough bytes to decode a
    /// value.
    fn has_enough_bytes(_buf: &BytesMut) -> bool {
        true
    }
}

pub trait RawDecoder<Item = Self> {
    /// The type of error that can occur if decoding fails.
    type Error;

    /// Decodes a value from the given slice.
    ///
    /// # Arguments
    /// * `buf` - The slice to decode the value from.
    ///
    /// # Returns
    /// The decoded value and the number of bytes read.
    fn raw_decode<'a>(buf: &'a [u8]) -> Result<(Item, usize), Self::Error>
    where
        Item: 'a;
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use bytes::{Bytes, BytesMut};

    use crate as recode;
    use crate::{codec::LengthPrefixed, Decoder};

    #[test]
    fn basic_test() {
        #[derive(Decoder)]
        #[recode(decoder(buffer_name = "buf"))]
        struct TestType {
            age: u32,
            salary: u64,
            #[recode(decoder(with = "LengthPrefixed::<u8>"))]
            first_name: Bytes,
            #[recode(decoder(with = "LengthPrefixed::<u16>"))]
            last_name: Bytes,
            #[recode(decoder(with = "LengthPrefixed::<u32>"))]
            image: Bytes,
        }

        const BUF: [u8; 78] = [
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
            // leftover bytes (10 bytes ascii text)
            b't', b'o', b' ', b'b', b'e', b' ', b'l', b'e', b'f', b't',
        ];

        let mut bytes = BytesMut::from_iter(BUF.iter());
        let test = TestType::decode(&mut bytes).unwrap();

        assert_eq!(0x01234567, test.age);
        assert_eq!(0x1122334455667788, test.salary);
        assert_eq!(b"ayman", test.first_name.deref());
        assert_eq!(
            "القاضي",
            std::str::from_utf8(test.last_name.deref()).unwrap()
        );
        assert_eq!(
            &[
                0xBA, 0x3E, 0x9D, 0x6B, 0xAE, 0xF5, 0x91, 0xCC, 0xE2, 0xF0,
                0xCB, 0x4F, 0xBB, 0x5B, 0x2B, 0xBE, 0xA7, 0xF4, 0x9D, 0xFB,
                0x87, 0x43, 0x0E, 0xDF, 0x30, 0x6F, 0x7D, 0x6E, 0x22, 0xAB,
                0xCC, 0x47,
            ],
            test.image.as_ref()
        );
        assert_eq!(bytes.as_ref(), b"to be left");
    }
}
