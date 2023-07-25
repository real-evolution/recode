pub use ux::{i24, i40, i48, i56, u24, u40, u48, u56};

use crate::{
    bytes::{Buf, BufMut, BytesMut},
    util::EncoderExt,
    Decoder,
    Encoder,
    RawDecoder,
};

macro_rules! impl_ux {
    ($t:ty; size: $s:literal; rep: $r:ty) => {
        impl Decoder for $t {
            type Error = crate::Error;

            #[inline]
            fn decode(buf: &mut BytesMut) -> Result<Self, Self::Error> {
                let (num, off) = Self::raw_decode(buf)?;
                buf.advance(off);

                Ok(num)
            }

            #[inline]
            fn has_enough_bytes(buf: &BytesMut) -> bool {
                buf.remaining() >= $s
            }
        }

        impl RawDecoder for $t {
            type Error = crate::Error;

            fn raw_decode<'a>(buf: &'a [u8]) -> Result<($t, usize), Self::Error>
            where
                $t: 'a,
            {
                const REPR_LEN: usize = std::mem::size_of::<$r>();

                if buf.remaining() < $s {
                    return Err(crate::Error::BytesNeeded {
                        needed: $s - buf.remaining(),
                        full_len: $s,
                        available: buf.remaining(),
                    });
                }

                let mut be_repr = [0u8; REPR_LEN];
                be_repr[(REPR_LEN - $s)..].copy_from_slice(&buf[..$s]);

                Ok((<$t>::new(<$r>::from_be_bytes(be_repr)), $s))
            }
        }

        impl Encoder for $t {
            type Error = std::convert::Infallible;

            fn encode(
                item: &$t,
                buf: &mut BytesMut,
            ) -> Result<(), Self::Error> {
                const REPR_LEN: usize = std::mem::size_of::<$r>();

                let bytes = &<$r>::from(*item).to_be_bytes()[(REPR_LEN - $s)..];

                buf.put_slice(bytes);

                Ok(())
            }

            #[inline]
            fn size_of(_: &$t) -> usize {
                $s
            }
        }

        impl Decoder<usize> for $t {
            type Error = crate::Error;

            fn decode(buf: &mut bytes::BytesMut) -> Result<usize, Self::Error> {
                let value = <Self as Decoder>::decode(buf)?;

                usize::try_from(<$r>::from(value))
                    .map_err(|_| super::number::TryFromIntError(()))
                    .map_err(Into::into)
            }

            #[inline]
            fn has_enough_bytes(buf: &BytesMut) -> bool {
                <Self as Decoder>::has_enough_bytes(buf)
            }
        }

        impl RawDecoder<usize> for $t {
            type Error = crate::Error;

            #[inline]
            fn raw_decode<'a>(
                buf: &'a [u8],
            ) -> Result<(usize, usize), Self::Error>
            where
                $t: 'a,
            {
                let (value, rx) = <Self as RawDecoder>::raw_decode(buf)?;
                let value: usize = <$r>::from(value)
                    .try_into()
                    .map_err(|_| super::number::TryFromIntError(()))?;

                Ok((value, rx))
            }
        }

        impl Encoder<usize> for $t {
            type Error = crate::Error;

            fn encode(
                item: &usize,
                buf: &mut BytesMut,
            ) -> Result<(), Self::Error> {
                let value = <$r>::try_from(*item)
                    .map_err(|_| super::number::TryFromIntError(()))?;

                <$t>::new(value).encode_to(buf).map_err(Into::into)
            }

            #[inline]
            fn size_of(_: &usize) -> usize {
                $s
            }
        }
    };
}

impl_ux!(i24; size: 3; rep: i32);
impl_ux!(u24; size: 3; rep: u32);

impl_ux!(i40; size: 5; rep: i64);
impl_ux!(u40; size: 5; rep: u64);

impl_ux!(i48; size: 6; rep: i64);
impl_ux!(u48; size: 6; rep: u64);

impl_ux!(i56; size: 7; rep: i64);
impl_ux!(u56; size: 7; rep: u64);

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use fake::Fake;

    use super::*;
    use crate::util::EncoderExt;

    macro_rules! test_ux {
        ($t:ty; size: $s:literal; rep: $r:ty ) => {
            paste::paste! {
                #[test]
                fn [<test_ $t>]() {
                    const REPR_LEN: usize = std::mem::size_of::<$r>();

                    let rmax: $r = <$t>::MAX.into();
                    let repr: $r = (0..rmax).fake();

                    if stringify!($t).starts_with("u") {
                        assert_eq!(rmax.trailing_ones(), $s * 8);
                    } else {
                        assert_eq!(rmax.trailing_ones(), $s * 8 - 1);
                    }

                    assert_eq!(repr & !rmax, 0);

                    let value = <$t>::new(repr);
                    let mut bytes = BytesMut::new();

                    assert_eq!($s, value.size());

                    value.encode_to(&mut bytes).unwrap();

                    assert_eq!($s, bytes.len());
                    assert_eq!(&repr.to_be_bytes()[(REPR_LEN - $s)..], &bytes[..]);

                    let (peek, rx): ($t, usize) =
                                      <$t>::raw_decode(bytes.chunk()).unwrap();

                    assert_eq!(rx, $s);
                    assert_eq!(peek, value);

                    let decoded: $t = <$t>::decode(&mut bytes).unwrap();

                    assert_eq!(decoded, value);
                    assert!(!bytes.has_remaining());
                }
            }
        };
    }

    test_ux!(i24; size: 3; rep: i32);
    test_ux!(u24; size: 3; rep: u32);

    test_ux!(i40; size: 5; rep: i64);
    test_ux!(u40; size: 5; rep: u64);

    test_ux!(i48; size: 6; rep: i64);
    test_ux!(u48; size: 6; rep: u64);

    test_ux!(i56; size: 7; rep: i64);
    test_ux!(u56; size: 7; rep: u64);
}
