use std::{error::Error as StdError, fmt, io, marker::PhantomData};

use bytes::{Buf, BytesMut};
use recode::{util::EncoderExt, Decoder, Encoder};
use tokio_util::codec::{Decoder as TokioDecoder, Encoder as TokioEncoder};

/// Trait for frames that can be decoded for encoded as a length-delimited
/// sequence of bytes.
pub trait LengthDelimitedFrame: Decoder + Encoder + Sized {
    /// Maximum frame length.
    const MAX_FRAME_LEN: usize = 8 * 1024 * 1024;

    /// Type representing the length of a frame.
    type Length: Decoder<usize> + Encoder<usize>;

    /// Error type that can be returned when decoding/encoding a frame.
    type Error: From<std::io::Error>
        + From<<Self as Decoder>::Error>
        + From<<Self as Encoder>::Error>
        + From<<Self::Length as Decoder<usize>>::Error>
        + From<<Self::Length as Encoder<usize>>::Error>;
}

/// A codec for decoding and decoding length-delimited frames that implement
/// [`LengthDelimitedFrame`].
#[derive(Debug, Clone)]
pub struct LengthDelimitedCodec<F> {
    state: DecodeState,
    _marker: PhantomData<F>,
}

/// Error returned when decoding a frame.
pub struct LengthDelimitedCodecError(&'static str);

/// Current decode state.
#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl<F> LengthDelimitedCodec<F> {
    /// Create a new [`LengthDelimitedCodec`] instance for [`F`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: DecodeState::Head,
            _marker: PhantomData,
        }
    }
}

impl<F> TokioDecoder for LengthDelimitedCodec<F>
where
    F: LengthDelimitedFrame,
{
    type Error = <F as LengthDelimitedFrame>::Error;
    type Item = F;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            | DecodeState::Head => {
                if <F::Length>::has_enough_bytes(src) {
                    let len = <F::Length>::decode(src)?;
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

impl<F> TokioEncoder<F> for LengthDelimitedCodec<F>
where
    F: LengthDelimitedFrame,
{
    type Error = <F as LengthDelimitedFrame>::Error;

    fn encode(
        &mut self,
        item: F,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let len = item.size();

        dst.reserve(len);

        <F::Length>::encode(&len, dst)?;
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
