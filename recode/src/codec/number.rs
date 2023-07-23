use crate::{
    bytes::{Buf, BytesMut, BufMut},
    util::EncoderExt,
    Decoder,
    Encoder,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromIntError(pub(crate) ());

impl From<std::num::TryFromIntError> for TryFromIntError {
    #[inline]
    fn from(_: std::num::TryFromIntError) -> TryFromIntError {
        TryFromIntError(())
    }
}

impl From<std::convert::Infallible> for TryFromIntError {
    #[inline]
    fn from(_: std::convert::Infallible) -> TryFromIntError {
        TryFromIntError(())
    }
}

impl std::fmt::Display for TryFromIntError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("integer overflow")
    }
}

impl std::error::Error for TryFromIntError {}
macro_rules! impl_int {
    ($t:ty) => {
        paste::paste! {
            impl Decoder for $t {
                type Error = crate::Error;

                #[inline]
                fn decode(buf: &mut BytesMut) -> Result<Self, Self::Error> {
                    const FULL_EN: usize = std::mem::size_of::<$t>();

                    if buf.remaining() < FULL_EN {
                        return Err(crate::Error::BytesNeeded {
                            needed: FULL_EN - buf.remaining(),
                            full_len: FULL_EN,
                            available: buf.remaining(),
                        });
                    }

                    Ok(buf.[<get_ $t>]())
                }
            }

            impl<B: BufMut> Encoder<B> for $t {
                type Error = std::convert::Infallible;

                #[inline]
                fn encode(item: &$t, buf: &mut B) -> Result<(), Self::Error> {
                    buf.[<put_ $t>](*item);

                    Ok(())
                }

                #[inline]
                fn size_of(_: &$t, _: &B) -> usize {
                    std::mem::size_of::<$t>()
                }
            }

            impl Decoder<usize> for $t {
                type Error = crate::Error;

                #[inline]
                fn decode(buf: &mut BytesMut) -> Result<usize, Self::Error> {
                    usize::try_from(<Self as crate::Decoder>::decode(buf)?)
                        .map_err(TryFromIntError::from)
                        .map_err(Into::into)
                }
            }

            impl<B: BufMut> Encoder<B, usize> for $t {
                type Error = crate::Error;

                #[inline]
                fn encode(item: &usize, buf: &mut B) -> Result<(), Self::Error> {
                    Self::try_from(*item)
                        .map_err(TryFromIntError::from)?
                        .encode_to(buf)
                        .map_err(Into::into)
                }

                #[inline]
                fn size_of(_: &usize, _: &B) -> usize {
                    std::mem::size_of::<$t>()
                }
            }
        }
    };
}

impl_int!(i8);
impl_int!(u8);

impl_int!(i16);
impl_int!(u16);

impl_int!(i32);
impl_int!(u32);

impl_int!(i64);
impl_int!(u64);

impl_int!(i128);
impl_int!(u128);

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use fake::Fake;

    use crate::util::EncoderExt;

    macro_rules! test_int {
        ($t:ty) => {
            paste::paste! {
                #[test]
                fn [<test_ $t>]() {
                    const LEN: usize = std::mem::size_of::<$t>();
                    const MAX: $t = <$t>::MAX;

                    let value: $t = (0..MAX).fake();

                    if stringify!($t).starts_with("u") {
                        assert_eq!(MAX.trailing_ones() as usize, LEN * 8);
                    } else {
                        assert_eq!(MAX.trailing_ones() as usize, LEN * 8 - 1);
                    }

                    assert_eq!(value & !MAX, 0);

                    let mut bytes = BytesMut::new();

                    assert_eq!(LEN, value.size(&bytes));

                    value.encode_to(&mut bytes).unwrap();

                    assert_eq!(LEN, bytes.len());
                    assert_eq!(&value.to_be_bytes()[..], &bytes[..]);
                }
            }
        };
    }

    test_int!(i8);
    test_int!(u8);

    test_int!(i16);
    test_int!(u16);

    test_int!(i32);
    test_int!(u32);

    test_int!(i64);
    test_int!(u64);

    test_int!(i128);
    test_int!(u128);
}
