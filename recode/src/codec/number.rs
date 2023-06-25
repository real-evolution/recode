macro_rules! impl_int {
    ($t:ty) => {
        paste::paste! {
            impl crate::encode::Encoder for $t {
                type Input = Self;

                fn encode<B: bytes::BufMut>(
                    &self,
                    input: Self::Input,
                    buf: &mut B,
                ) {
                    buf.[<put_ $t>](input)
                }
            }

            impl crate::decode::Decoder for $t {
                type Output = Self;
                type Error = crate::error::Error;

                fn decode<B: bytes::Buf>(
                    buf: &mut B,
                ) -> Result<Self::Output, Self::Error> {
                    const FULL_EN: usize = std::mem::size_of::<$t>();

                    if buf.remaining() < FULL_EN {
                        return Err(crate::error::Error::BytesNeeded {
                            needed: FULL_EN - buf.remaining(),
                            full_len: FULL_EN,
                            available: buf.remaining(),
                        });
                    }

                    Ok(buf.[<get_ $t>]())
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

impl_int!(f32);
impl_int!(f64);
