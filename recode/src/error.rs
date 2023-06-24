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
}
