pub mod codec;
pub mod decode;
pub mod encode;
pub mod error;

pub use decode::Decoder;
pub use encode::Encoder;
pub use error::{Error, Result};

pub use bytes;

#[cfg(feature = "derive")]
pub use recode_derive::{Decoder, Encoder, Recode};
