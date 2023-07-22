use crate::{
    bytes::{Buf, BufMut},
    util::EncoderExt,
    Decoder,
    Encoder,
    Error,
};

/// A type alias for a [`Buffer`] without a length prefix.
pub type UnprefixedBuffer = Buffer<crate::util::Remaining>;

/// A wrapper type for consecutive bytes.
///
/// # Type Parameters
/// - `L`: If not [`()`], it should be a numerical type that implements
/// [`Decoder<Output = L`] and [`TryFrom<usize>`]` which represents the length
/// prefix of the buffer.
#[derive(Debug, Clone, Default)]
pub struct Buffer<L> {
    inner: bytes::Bytes,
    _marker: std::marker::PhantomData<L>,
}

impl<L> Buffer<L> {
    /// Creates a new [`Buffer<L>`] object from a [`bytes::Bytes`] instance.
    ///
    /// # Parameters
    /// - `bytes`: The [`bytes::Bytes`] instance to wrap.
    ///
    /// # Returns
    /// A new [`Buffer<L>`] object.
    pub fn new(bytes: bytes::Bytes) -> Self {
        Self {
            inner: bytes,
            _marker: Default::default(),
        }
    }

    /// Creates a new [`Buffer<L>`] object from a [`&'static [u8]`].
    ///
    /// This is a shorthand for
    /// [`Buffer::new`]`(`[`bytes::Bytes::from_static`]`(`[`&'static [u8]`]`))`.
    ///
    /// # Parameters
    /// - `bytes`: The [`&'static [u8]`] instance to wrap.
    ///
    /// # Returns
    /// A new [`Buffer<L>`] object.
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self::new(bytes.into())
    }

    /// Consumes the [`Buffer<L>`] object and returns the inner
    /// [`bytes::Bytes`] instance.
    pub fn into_inner(self) -> bytes::Bytes {
        self.inner
    }

    #[inline]
    fn take_n_bytes<B: Buf>(buf: &mut B, len: usize) -> crate::Result<Self> {
        if buf.remaining() < len {
            return Err(Error::BytesNeeded {
                needed: len - buf.remaining(),
                full_len: len,
                available: buf.remaining(),
            });
        }

        Ok(Self::new(buf.copy_to_bytes(len)))
    }
}

impl<B, L> Decoder<B> for Buffer<L>
where
    B: Buf,
    L: Decoder<B, usize>,
    Error: From<<L as Decoder<B, usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut B) -> Result<Self, Self::Error> {
        let len = L::decode(buf)?;

        Self::take_n_bytes(buf, len)
    }
}

impl<B, L> Decoder<B> for Option<Buffer<L>>
where
    B: Buf,
    L: Decoder<B, usize>,
    Error: From<<L as Decoder<B, usize>>::Error>,
{
    type Error = Error;

    #[inline]
    fn decode(buf: &mut B) -> Result<Self, Self::Error> {
        let ret = match L::decode(buf)? {
            | 0 => None,
            | len => Some(Buffer::take_n_bytes(buf, len)?),
        };

        Ok(ret)
    }
}

impl<B, L> Encoder<B> for Buffer<L>
where
    B: BufMut,
    L: Encoder<B, usize>,
    Error: From<<L as Encoder<B, usize>>::Error>,
{
    type Error = Error;

    fn encode(item: &Self, buf: &mut B) -> Result<(), Self::Error> {
        let len = item.inner.len();

        L::encode(&len, buf)?;
        buf.put(item.inner.as_ref());

        Ok(())
    }

    #[inline]
    fn size_of(item: &Self, buf: &B) -> usize {
        L::size_of(&item.inner.len(), buf) + item.inner.len()
    }
}

impl<B, L> Encoder<B> for Option<Buffer<L>>
where
    B: BufMut,
    L: Encoder<B, usize> + Default,
    Error: From<<L as Encoder<B, usize>>::Error>,
{
    type Error = Error;

    fn encode(item: &Self, buf: &mut B) -> Result<(), Self::Error> {
        let Some(buffer) = item else {
            L::encode(&0, buf)?;
            return Ok(());
        };

        buffer.encode_to(buf)
    }

    #[inline]
    fn size_of(item: &Self, buf: &B) -> usize {
        match item {
            | Some(inner) => inner.size(buf),
            | None => 0,
        }
    }
}

impl<L> AsRef<[u8]> for Buffer<L> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl<L> std::ops::Deref for Buffer<L> {
    type Target = bytes::Bytes;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<L> std::ops::DerefMut for Buffer<L> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<L> From<&'static [u8]> for Buffer<L> {
    #[inline(always)]
    fn from(value: &'static [u8]) -> Self {
        Self::from_static(value)
    }
}

impl<L> From<bytes::Bytes> for Buffer<L> {
    #[inline(always)]
    fn from(value: bytes::Bytes) -> Self {
        Self::new(value)
    }
}

impl<L> From<Buffer<L>> for bytes::Bytes {
    #[inline(always)]
    fn from(value: Buffer<L>) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use fake::{Fake, Faker};

    #[cfg(all(test, feature = "ux"))]
    use crate::codec::ux::*;
    use crate::{codec::*, util::EncoderExt, *};

    #[test]
    fn optional_unprefixed_test() {
        let buffer =
            Option::<UnprefixedBuffer>::decode(&mut BytesMut::new()).unwrap();

        assert!(buffer.is_none());

        let len: usize = (128..=10240).fake();
        let bytes = BytesMut::from_iter((0..len).map(|_| Faker.fake::<u8>()));

        assert_eq!(len, bytes.len());

        let buffer: UnprefixedBuffer =
            Option::<UnprefixedBuffer>::decode(&mut bytes.clone())
                .unwrap()
                .unwrap();

        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.as_ref(), bytes.as_ref());
        assert_eq!(buffer.size(&bytes), len);

        let mut encoded = BytesMut::new();
        Option::<UnprefixedBuffer>::default()
            .encode_to(&mut encoded)
            .unwrap();

        assert!(encoded.is_empty());

        buffer.encode_to(&mut encoded).unwrap();

        assert_eq!(encoded.len(), len);
        assert_eq!(encoded.as_ref(), buffer.as_ref());
    }

    #[test]
    fn unprefixed_decode_test() {
        let len: usize = (128..=10240).fake();
        let bytes = BytesMut::from_iter((0..len).map(|_| Faker.fake::<u8>()));

        assert_eq!(len, bytes.len());

        let buffer = UnprefixedBuffer::decode(&mut bytes.clone()).unwrap();

        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.as_ref(), bytes.as_ref());
        assert_eq!(buffer.size(&bytes), len);

        let mut encoded = BytesMut::new();
        buffer.encode_to(&mut encoded).unwrap();

        assert_eq!(encoded.len(), len);
        assert_eq!(encoded.as_ref(), buffer.as_ref());
    }

    #[test]
    fn whole_prefix_test() {
        let full_len: usize = (128..=10240).fake();
        let use_len: usize = (0..full_len).fake();
        let pool = Bytes::from_iter((0..full_len).map(|_| Faker.fake::<u8>()));

        assert!(use_len < full_len);
        assert!(pool.len() == full_len);

        let buffer = super::Buffer::<u32>::new(pool.slice(0..use_len));
        let mut bytes = BytesMut::new();

        buffer.encode_to(&mut bytes).unwrap();

        assert_eq!(
            buffer.size(&bytes),
            buffer.len() + u32::size_of(&buffer.len(), &bytes)
        );

        assert_eq!(buffer.len(), use_len);
        assert_eq!(bytes.len(), 4 + use_len);
        assert_eq!((use_len as u32).to_be_bytes(), bytes[..4]);
        assert_eq!(buffer.as_ref(), bytes[4..].as_ref());

        let decoded = Buffer::<u32>::decode(&mut bytes).unwrap();

        assert_eq!(decoded.len(), use_len);
        assert_eq!(decoded.as_ref(), buffer.as_ref());
    }

    macro_rules! test_ux_len {
        ($t:ty; size: $s:literal; rep: $r:ty ) => {
            paste::paste! {
                #[test]
                #[cfg(all(test, feature = "ux"))]
                fn [<sub_prefix_test_ $t>]() {
                    const REPR_LEN: usize = std::mem::size_of::<$r>();

                    let max: usize = <$t>::MAX.try_into().unwrap();
                    let full_len: usize =
                        (128..=std::cmp::min(10240usize, max)).fake();
                    let use_len: usize = (0..full_len).fake();
                    let pool =
                        Bytes::from_iter((0..full_len).map(|_| Faker.fake::<u8>()));

                    assert!(use_len < full_len);
                    assert!(pool.len() == full_len);

                    let buffer = super::Buffer::<$t>::new(pool.slice(0..use_len));
                    let mut bytes = BytesMut::new();

                    assert_eq!(
                        buffer.size(&bytes),
                        buffer.len() + <$t>::size_of(&buffer.len(), &bytes)
                    );

                    Option::<Buffer::<$t>>::default()
                        .encode_to(&mut bytes)
                        .unwrap();

                    assert_eq!(bytes.len(), $s);
                    assert!(bytes.iter().all(|&b| b == 0));

                    let decoded = Option::<Buffer::<$t>>::decode(&mut bytes)
                        .unwrap();

                    assert!(decoded.is_none());
                    assert!(bytes.is_empty());

                    buffer.encode_to(&mut bytes).unwrap();

                    let len_bytes = &(use_len as $r)
                        .to_be_bytes()[(REPR_LEN - $s)..];

                    assert_eq!(buffer.len(), use_len);
                    assert_eq!(bytes.len(), $s + use_len);
                    assert_eq!(len_bytes, &bytes[0..$s]);
                    assert_eq!(buffer.as_ref(), bytes[$s..].as_ref());

                    let decoded = Buffer::<$t>::decode(&mut bytes).unwrap();

                    assert_eq!(decoded.len(), use_len);
                    assert_eq!(decoded.as_ref(), buffer.as_ref());
                }
            }
        };
    }

    test_ux_len!(u24; size: 3; rep: u32);
    test_ux_len!(u40; size: 5; rep: u64);
    test_ux_len!(u48; size: 6; rep: u64);
    test_ux_len!(u56; size: 7; rep: u64);
}
