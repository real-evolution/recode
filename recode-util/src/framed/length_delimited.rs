use std::{error::Error as StdError, fmt, io, marker::PhantomData};

use bytes::{Buf, BytesMut};
use recode::{util::EncoderExt, Decoder, Encoder};
use tokio_util::codec::{Decoder as TokioDecoder, Encoder as TokioEncoder};

/// Trait for frames that can be decoded for encoded as a length-delimited
/// sequence of bytes.
pub trait LengthDelimitedFrame<L>: Decoder + Encoder + Sized
where
    L: Decoder<usize> + Encoder<usize>,
{
    /// Error type that can be returned when decoding/encoding a frame.
    type Error: From<std::io::Error>
        + From<<Self as Decoder>::Error>
        + From<<Self as Encoder>::Error>
        + From<<L as Decoder<usize>>::Error>
        + From<<L as Encoder<usize>>::Error>;
}

/// A codec for decoding and decoding length-delimited frames that implement
/// [`LengthDelimitedFrame`].
#[derive(Debug, Clone)]
pub struct LengthDelimitedCodec<F, L> {
    max_frame_len: usize,
    state: DecodeState,
    _marker: PhantomData<fn() -> (F, L)>,
}

/// Error returned when decoding a frame.
pub struct LengthDelimitedCodecError(&'static str);

/// Current decode state.
#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl<F, L> LengthDelimitedCodec<F, L> {
    /// Create a new [`LengthDelimitedCodec`] instance for [`F`].
    #[inline]
    pub const fn new(max_frame_len: usize) -> Self {
        Self {
            max_frame_len,
            state: DecodeState::Head,
            _marker: PhantomData,
        }
    }
}

impl<F, L> TokioDecoder for LengthDelimitedCodec<F, L>
where
    F: LengthDelimitedFrame<L>,
    L: Decoder<usize> + Encoder<usize>,
{
    type Error = <F as LengthDelimitedFrame<L>>::Error;
    type Item = F;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            | DecodeState::Head => {
                if L::has_enough_bytes(src) {
                    let len = L::decode(src)?;

                    if len > self.max_frame_len {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            LengthDelimitedCodecError(
                                "frame length exceeds maximum",
                            ),
                        ))?;
                    }

                    src.reserve(len);
                    self.state = DecodeState::Data(len);
                }

                Ok(None)
            }
            | DecodeState::Data(len) => {
                if src.remaining() < len {
                    return Ok(None);
                }

                let mut src = src.split_to(len);
                let frame = F::decode(&mut src)?;

                if !src.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        LengthDelimitedCodecError(
                            "bytes remaining after frame",
                        ),
                    ))?;
                }

                self.state = DecodeState::Head;

                Ok(Some(frame))
            }
        }
    }
}

impl<F, L> TokioEncoder<F> for LengthDelimitedCodec<F, L>
where
    F: LengthDelimitedFrame<L>,
    L: Decoder<usize> + Encoder<usize>,
{
    type Error = <F as LengthDelimitedFrame<L>>::Error;

    fn encode(
        &mut self,
        item: F,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let len = item.size();

        dst.reserve(len);

        <L>::encode(&len, dst)?;
        <F>::encode(&item, dst)?;

        Ok(())
    }
}

impl fmt::Debug for LengthDelimitedCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LengthDelimitedCodecError").finish()
    }
}

impl fmt::Display for LengthDelimitedCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl StdError for LengthDelimitedCodecError {}
