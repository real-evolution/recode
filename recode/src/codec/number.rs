use crate::bytes::{Buf, BufMut};
use crate::{Decoder, Encoder};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromIntError(pub(crate) ());

impl From<std::num::TryFromIntError> for TryFromIntError {
    fn from(_: std::num::TryFromIntError) -> TryFromIntError {
        TryFromIntError(())
    }
}

impl From<std::convert::Infallible> for TryFromIntError {
    fn from(_: std::convert::Infallible) -> TryFromIntError {
        TryFromIntError(())
    }
}

impl std::fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("integer overflow")
    }
}

impl std::error::Error for TryFromIntError {}
macro_rules! impl_int {
    ($t:ty) => {
        paste::paste! {
            impl<B: Buf> Decoder<B> for $t {
                type Error = crate::Error;

                fn decode(buf: &mut B) -> Result<Self, Self::Error> {
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

                fn encode(item: &$t, buf: &mut B) -> Result<(), Self::Error> {
                    buf.[<put_ $t>](*item);

                    Ok(())
                }
            }

            impl<B: Buf> Decoder<B, usize> for $t {
                type Error = crate::Error;

                fn decode(buf: &mut B) -> Result<usize, Self::Error> {
                    let value = <Self as crate::Decoder<B>>::decode(buf)?;

                    usize::try_from(value)
                        .map_err(TryFromIntError::from)
                        .map_err(Into::into)
                }
            }

            impl<B: BufMut> Encoder<B, usize> for $t {
                type Error = crate::Error;

                fn encode(item: &usize, buf: &mut B) -> Result<(), Self::Error> {
                    let value = Self::try_from(*item)
                        .map_err(TryFromIntError::from)?;

                    Self::encode(&value, buf).map_err(Into::into)
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
