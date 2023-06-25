use crate::{Decoder, Error, Result};

pub trait LengthTraits: Decoder<Output = Self> + Sized
where
    usize: TryFrom<Self>,
    Error: From<<Self as Decoder>::Error>,
    Error: From<<usize as TryFrom<Self>>::Error>,
{
    const BYTE_COUNT: usize = std::mem::size_of::<Self>();

    fn decode_usize<B: bytes::Buf>(buf: &mut B) -> Result<usize> {
        let value = Self::decode(buf)?;

        Ok(value.try_into()?)
    }
}

impl LengthTraits for u16 {}
impl LengthTraits for u32 {}
impl LengthTraits for u64 {}
impl LengthTraits for u128 {}
