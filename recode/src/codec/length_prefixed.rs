use std::marker::PhantomData;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{util::Remaining, Decoder, Encoder, Error};

/// A buffer that is not prefixed with its length.
pub type Unprefixed = LengthPrefixed<Remaining>;

/// An encoder/decoder for length-prefixed buffers.
///
/// This currently only supports encoding/decoding [`Bytes`](bytes::Bytes).
#[derive(Debug, Clone, Copy, Default)]
pub struct LengthPrefixed<L>(PhantomData<L>);

impl<L> Decoder<Bytes> for LengthPrefixed<L>
where
    L: Decoder<usize>,
    Error: From<<L as Decoder<usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut BytesMut) -> Result<Bytes, Self::Error> {
        let len = L::decode(buf)?;

        take_n_bytes(buf, len)
    }
}

impl<L> Decoder<Option<Bytes>> for LengthPrefixed<L>
where
    L: Decoder<usize>,
    Error: From<<L as Decoder<usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut BytesMut) -> Result<Option<Bytes>, Self::Error> {
        let ret = match L::decode(buf)? {
            | 0 => None,
            | len => Some(take_n_bytes(buf, len)?),
        };

        Ok(ret)
    }
}

impl<B, L> Encoder<B, Bytes> for LengthPrefixed<L>
where
    B: BufMut,
    L: Encoder<B, usize>,
    Error: From<<L as Encoder<B, usize>>::Error>,
{
    type Error = Error;

    fn encode(item: &Bytes, buf: &mut B) -> Result<(), Self::Error> {
        let len = item.len();

        L::encode(&len, buf)?;
        buf.put_slice(item.as_ref());

        Ok(())
    }

    #[inline]
    fn size_of(item: &Bytes, buf: &B) -> usize {
        L::size_of(&item.len(), buf) + item.len()
    }
}

impl<B, L> Encoder<B, Option<Bytes>> for LengthPrefixed<L>
where
    B: BufMut,
    L: Encoder<B, usize> + Default,
    Error: From<<L as Encoder<B, usize>>::Error>,
{
    type Error = Error;

    fn encode(item: &Option<Bytes>, buf: &mut B) -> Result<(), Self::Error> {
        let Some(buffer) = item else {
            L::encode(&0, buf)?;
            return Ok(());
        };

        Self::encode(buffer, buf)
    }

    #[inline]
    fn size_of(item: &Option<Bytes>, buf: &B) -> usize {
        match item {
            | Some(item) => Self::size_of(item, buf),
            | None => 0,
        }
    }
}

#[inline]
fn take_n_bytes(buf: &mut BytesMut, len: usize) -> crate::Result<Bytes> {
    if buf.remaining() < len {
        return Err(Error::BytesNeeded {
            needed: len - buf.remaining(),
            full_len: len,
            available: buf.remaining(),
        });
    }

    Ok(buf.split_to(len).freeze())
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
    fn optional_unprefixed_test() {
        let buffer: Option<Bytes> =
            Unprefixed::decode(&mut BytesMut::new()).unwrap();

        assert!(buffer.is_none());

        let len: usize = (128..=10240).fake();
        let bytes = BytesMut::from_iter((0..len).map(|_| Faker.fake::<u8>()));

        assert_eq!(len, bytes.len());

        let buffer: Option<Bytes> =
            Unprefixed::decode(&mut bytes.clone()).unwrap();
        let buffer = buffer.unwrap();

        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.as_ref(), bytes.as_ref());
        assert_eq!(Unprefixed::size_of(&buffer, &bytes), len);

        let mut encoded = BytesMut::new();
        Unprefixed::encode(&Bytes::default(), &mut encoded).unwrap();

        assert!(encoded.is_empty());

        Unprefixed::encode(&buffer, &mut encoded).unwrap();

        assert_eq!(encoded.len(), len);
        assert_eq!(encoded.as_ref(), buffer.as_ref());
    }

    #[test]
    fn unprefixed_decode_test() {
        let len: usize = (128..=10240).fake();
        let bytes = BytesMut::from_iter((0..len).map(|_| Faker.fake::<u8>()));

        assert_eq!(len, bytes.len());

        let buffer: Bytes = Unprefixed::decode(&mut bytes.clone()).unwrap();

        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.as_ref(), bytes.as_ref());
        assert_eq!(Unprefixed::size_of(&buffer, &bytes), len);

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
            LengthPrefixed::<u32>::size_of(&buffer, &bytes),
            buffer.len() + u32::size_of(&buffer.len(), &bytes)
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
                        LengthPrefixed::<$t>::size_of(&buffer, &bytes),
                        buffer.len() + <$t>::size_of(&buffer.len(), &bytes)
                    );

                    LengthPrefixed::<$t>::encode(&None, &mut bytes).unwrap();

                    assert_eq!(bytes.len(), $s);
                    assert!(bytes.iter().all(|&b| b == 0));

                    let decoded: Option<Bytes> = LengthPrefixed::<$t>::decode(&mut bytes)
                        .unwrap();

                    assert!(decoded.is_none());
                    assert!(bytes.is_empty());

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
