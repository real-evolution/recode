use std::marker::PhantomData;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    util::{BufExt, Remaining},
    Decoder,
    Encoder,
    Error,
    RawDecoder,
};

/// A buffer that is not prefixed with its length.
pub type Unprefixed = LengthPrefixed<Remaining>;

/// An encoder/decoder for length-prefixed buffers.
///
/// This currently only supports encoding/decoding [`Bytes`](bytes::Bytes).
#[derive(Debug, Clone, Copy, Default)]
pub struct LengthPrefixed<L>(PhantomData<L>);

impl<L> Decoder<BytesMut> for LengthPrefixed<L>
where
    L: RawDecoder<usize>,
    Error: From<<L as RawDecoder<usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut BytesMut) -> Result<BytesMut, Self::Error> {
        let (len, rx) = L::raw_decode(buf.chunk())?;

        buf.require_n(len)?;
        buf.advance(rx);

        Ok(buf.split_to(len))
    }
}

impl<L> Decoder<Bytes> for LengthPrefixed<L>
where
    L: RawDecoder<usize>,
    Error: From<<L as RawDecoder<usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut BytesMut) -> Result<Bytes, Self::Error> {
        <Self as Decoder<BytesMut>>::decode(buf).map(BytesMut::freeze)
    }
}

impl<L, T> Encoder<T> for LengthPrefixed<L>
where
    T: AsRef<[u8]>,
    L: Encoder<usize>,
    Error: From<<L as Encoder<usize>>::Error>,
{
    type Error = Error;

    fn encode(item: &T, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let slice = item.as_ref();

        L::encode(&slice.len(), buf)?;
        buf.put_slice(slice);

        Ok(())
    }

    #[inline]
    fn size_of(item: &T) -> usize {
        let slice = item.as_ref();

        L::size_of(&slice.len()) + slice.len()
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use fake::{Fake, Faker};

    #[cfg(all(test, feature = "ux"))]
    use crate::codec::ux::*;
    use crate::{
        codec::{length_prefixed::Unprefixed, *},
        Decoder,
        Encoder,
    };

    #[test]
    fn unprefixed_decode_test() {
        let len: usize = (128..=10240).fake();
        let bytes = BytesMut::from_iter((0..len).map(|_| Faker.fake::<u8>()));

        assert_eq!(len, bytes.len());

        let buffer: Bytes = Unprefixed::decode(&mut bytes.clone()).unwrap();

        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.as_ref(), bytes.as_ref());
        assert_eq!(Unprefixed::size_of(&buffer), len);

        let mut encoded = BytesMut::new();
        Unprefixed::encode(&buffer, &mut encoded).unwrap();

        assert_eq!(encoded.len(), len);
        assert_eq!(encoded.as_ref(), buffer.as_ref());
    }

    #[test]
    fn whole_prefix_test() {
        let full_len: usize = (128..=10240).fake();
        let use_len: usize = (0..full_len).fake();
        let pool = Bytes::from_iter((0..full_len).map(|_| Faker.fake::<u8>()));

        assert!(use_len < full_len);
        assert!(pool.len() == full_len);

        let buffer = pool.slice(0..use_len);
        let mut bytes = BytesMut::new();

        LengthPrefixed::<u32>::encode(&buffer, &mut bytes).unwrap();

        assert_eq!(
            LengthPrefixed::<u32>::size_of(&buffer),
            buffer.len() + u32::size_of(&buffer.len())
        );

        assert_eq!(buffer.len(), use_len);
        assert_eq!(bytes.len(), 4 + use_len);
        assert_eq!((use_len as u32).to_be_bytes(), bytes[..4]);
        assert_eq!(buffer.as_ref(), bytes[4..].as_ref());

        let decoded: Bytes = LengthPrefixed::<u32>::decode(&mut bytes).unwrap();

        assert_eq!(decoded.len(), use_len);
        assert_eq!(decoded.as_ref(), buffer.as_ref());
    }

    macro_rules! test_ux_len {
        ($t:ty; size: $s:literal; rep: $r:ty ) => {
            paste::paste! {
                #[test]
                #[cfg(all(test, feature = "ux"))]
                fn [<sub_prefix_test_ $t>]() {
                    const REPR_LEN: usize = std::mem::size_of::<$r>();

                    let max: usize = <$t>::MAX.try_into().unwrap();
                    let full_len: usize =
                        (128..=std::cmp::min(10240usize, max)).fake();
                    let use_len: usize = (0..full_len).fake();
                    let pool =
                        Bytes::from_iter((0..full_len).map(|_| Faker.fake::<u8>()));

                    assert!(use_len < full_len);
                    assert!(pool.len() == full_len);

                    let buffer = pool.slice(0..use_len);
                    let mut bytes = BytesMut::new();

                    assert_eq!(
                        LengthPrefixed::<$t>::size_of(&buffer),
                        buffer.len() + <$t>::size_of(&buffer.len())
                    );

                    LengthPrefixed::<$t>::encode(&buffer, &mut bytes).unwrap();

                    let len_bytes = &(use_len as $r)
                        .to_be_bytes()[(REPR_LEN - $s)..];

                    assert_eq!(buffer.len(), use_len);
                    assert_eq!(bytes.len(), $s + use_len);
                    assert_eq!(len_bytes, &bytes[0..$s]);
                    assert_eq!(buffer.as_ref(), bytes[$s..].as_ref());

                    let decoded: Bytes = LengthPrefixed::<$t>::decode(&mut bytes).unwrap();

                    assert_eq!(decoded.len(), use_len);
                    assert_eq!(decoded.as_ref(), buffer.as_ref());
                }
            }
        };
    }

    test_ux_len!(u24; size: 3; rep: u32);
    test_ux_len!(u40; size: 5; rep: u64);
    test_ux_len!(u48; size: 6; rep: u64);
    test_ux_len!(u56; size: 7; rep: u64);
}
