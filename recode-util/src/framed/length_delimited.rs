use std::{error::Error as StdError, fmt, io, marker::PhantomData};

use bytes::{Buf, BytesMut};
use recode::{util::EncoderExt, Decoder, Encoder};
use tokio_util::codec::{Decoder as TokioDecoder, Encoder as TokioEncoder};

/// A codec for decoding and decoding length-delimited frames that implement
/// [`LengthDelimitedFrame`].
#[derive(Debug)]
pub struct LengthDelimitedCodec<L, F, E> {
    max_frame_len: usize,
    state: DecodeState,
    _marker: PhantomData<(L, F, E)>,
}

/// Error returned when decoding a frame.
pub struct LengthDelimitedCodecError(&'static str);

/// Current decode state.
#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl<L, F, E> LengthDelimitedCodec<L, F, E> {
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

impl<L, F, E> TokioDecoder for LengthDelimitedCodec<L, F, E>
where
    L: Decoder<usize>,
    F: Decoder,
    E: From<std::io::Error>
        + From<<L as Decoder<usize>>::Error>
        + From<<F as Decoder>::Error>,
{
    type Error = E;
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

impl<L, F, E> TokioEncoder<F> for LengthDelimitedCodec<L, F, E>
where
    L: Encoder<usize>,
    F: Encoder,
    E: From<std::io::Error>
        + From<<L as Encoder<usize>>::Error>
        + From<<F as Encoder>::Error>,
{
    type Error = E;

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
