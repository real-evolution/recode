use bytes::Buf;

/// Extension trait for [`Buf`](bytes::Buf).
pub trait BufExt: Buf {
    /// Checks if the buffer has at least `n` bytes remaining.
    ///
    /// If there are less than `n` bytes remaining, returns an
    /// [`crate::Error::BytesNeeded`] error with requirement information.
    #[inline]
    fn require_n(&self, n: usize) -> crate::Result<()> {
        if self.remaining() < n {
            return Err(crate::Error::BytesNeeded {
                needed: n - self.remaining(),
                full_len: n,
                available: self.remaining(),
            });
        }

        Ok(())
    }
}

impl<T> BufExt for T where T: Buf {}
