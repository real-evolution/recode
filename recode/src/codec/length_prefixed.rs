use std::marker::PhantomData;

use bytes::{Buf, BufMut, Bytes};

use crate::{util::Remaining, Decoder, Encoder, Error};

/// A buffer that is not prefixed with its length.
pub type Unprefixed = LengthPrefixed<Remaining>;

/// An encoder/decoder for length-prefixed buffers.
///
/// This currently only supports encoding/decoding [`Bytes`](bytes::Bytes).
#[derive(Debug, Clone, Copy, Default)]
pub struct LengthPrefixed<L>(PhantomData<L>);

impl<B, L> Decoder<B, Bytes> for LengthPrefixed<L>
where
    B: Buf,
    L: Decoder<B, usize>,
    Error: From<<L as Decoder<B, usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut B) -> Result<Bytes, Self::Error> {
        let len = L::decode(buf)?;

        take_n_bytes(buf, len)
    }
}

impl<B, L> Decoder<B, Option<Bytes>> for LengthPrefixed<L>
where
    B: Buf,
    L: Decoder<B, usize>,
    Error: From<<L as Decoder<B, usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut B) -> Result<Option<Bytes>, Self::Error> {
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
fn take_n_bytes<B: Buf>(buf: &mut B, len: usize) -> crate::Result<Bytes> {
    if buf.remaining() < len {
        return Err(Error::BytesNeeded {
            needed: len - buf.remaining(),
            full_len: len,
            available: buf.remaining(),
        });
    }

    Ok(buf.copy_to_bytes(len))
}
