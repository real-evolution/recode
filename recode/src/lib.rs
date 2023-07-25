pub mod codec;
pub mod decode;
pub mod encode;
pub mod error;
pub mod util;

/// Re-export of [`bytes`](https://docs.rs/bytes) crate.
pub use bytes;
#[doc(inline)]
pub use decode::{Decoder, RawDecoder};
#[doc(inline)]
pub use encode::Encoder;
pub use error::{Error, Result};
#[cfg(feature = "derive")]
pub use recode_derive::{Decoder, Encoder, Recode};
