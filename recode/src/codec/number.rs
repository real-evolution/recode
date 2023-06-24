use std::convert::Infallible;

macro_rules! impl_int {
    ($t:ty) => {
        paste::paste! {
            impl crate::Encoder for $t {
                type Input = Self;

                fn encode<B: bytes::BufMut>(
                    &self,
                    input: Self::Input,
                    buf: &mut B,
                ) {
                    buf.[<put_ $t>](input)
                }
            }

            impl crate::Decoder for $t {
                type Output = Self;
                type Error = Infallible;

                fn decode<B: bytes::Buf>(
                    buf: &mut B,
                ) -> Result<Self::Output, Self::Error> {
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

impl_int!(f32);
impl_int!(f64);
