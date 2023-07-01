use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "{} more bytes needed to read {} bytes ({} bytes available)",
        needed,
        full_len,
        available
    )]
    BytesNeeded {
        needed: usize,
        full_len: usize,
        available: usize,
    },

    #[error("integer conversion")]
    IntConversion(#[from] crate::codec::TryFromIntError),

    #[error("text: {0}")]
    Text(#[from] crate::codec::TextError),
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
