use crate::bytes::{Buf, BufMut};
use crate::{Decoder, Encoder};

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
