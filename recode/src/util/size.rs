use crate::bytes::{Buf, BufMut};
use crate::{Decoder, Encoder, Error};

impl<B: Buf, T> Decoder<B, usize> for T
where
    T: Decoder<B>,
    usize: TryFrom<T>,
    Error: From<<usize as TryFrom<T>>::Error> + From<T::Error>,
{
    type Error = Error;

    fn decode(buf: &mut B) -> Result<usize, Self::Error> {
        let value = <Self as crate::Decoder<B>>::decode(buf)?;

        usize::try_from(value).map_err(Into::into)
    }
}

impl<B: BufMut, T> Encoder<B, usize> for T
where
    T: Encoder<B> + TryFrom<usize>,
    Error: From<<T as TryFrom<usize>>::Error> + From<<T as Encoder<B>>::Error>,
{
    type Error = Error;

    fn encode(item: &usize, buf: &mut B) -> Result<(), Self::Error> {
        let value = T::try_from(*item)?;

        T::encode(&value, buf).map_err(Into::into)
    }
}
