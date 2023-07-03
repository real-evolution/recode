#[derive(Debug)]
pub struct ContextBuffer<B, C> {
    inner: B,
    context: C,
}

impl<B, C> ContextBuffer<B, C> {
    /// Creates a new [`ContextBuffer<B, C>`] instance.
    ///
    /// # Parameters
    /// * `inner`: The inner buffer type to wrap.
    /// * `context`: The context to provide.
    ///
    /// # Returns
    /// The created [`ContextBuffer<B, C>`] instance.
    #[inline]
    pub const fn new(inner: B, context: C) -> Self {
        Self { inner, context }
    }

    /// Gets the wrapped buffer.
    #[inline]
    pub const fn inner(&self) -> &B {
        &self.inner
    }

    /// Gets the wrapped context.
    #[inline]
    pub const fn context(&self) -> &C {
        &self.context
    }

    /// Gets the wrapped buffer and context, consuming `self`.
    #[inline]
    pub fn into_parts(self) -> (B, C) {
        (self.inner, self.context)
    }
}

impl<B, C> crate::bytes::Buf for ContextBuffer<B, C>
where
    B: crate::bytes::Buf,
{
    delegate::delegate! {
        to self.inner {
            fn remaining(&self) -> usize;
            fn chunk(&self) -> &[u8];
            fn advance(&mut self, cnt: usize);

            fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize;
            fn has_remaining(&self) -> bool;
            fn copy_to_slice(&mut self, dst: &mut [u8]);

            fn get_u8(&mut self) -> u8;
            fn get_i8(&mut self) -> i8;

            fn get_u16(&mut self) -> u16;
            fn get_u16_le(&mut self) -> u16;
            fn get_u16_ne(&mut self) -> u16;
            fn get_i16(&mut self) -> i16;
            fn get_i16_le(&mut self) -> i16;
            fn get_i16_ne(&mut self) -> i16;

            fn get_u32(&mut self) -> u32;
            fn get_u32_le(&mut self) -> u32;
            fn get_u32_ne(&mut self) -> u32;
            fn get_i32(&mut self) -> i32;
            fn get_i32_le(&mut self) -> i32;
            fn get_i32_ne(&mut self) -> i32;

            fn get_u64(&mut self) -> u64;
            fn get_u64_le(&mut self) -> u64;
            fn get_u64_ne(&mut self) -> u64;
            fn get_i64(&mut self) -> i64;
            fn get_i64_le(&mut self) -> i64;
            fn get_i64_ne(&mut self) -> i64;

            fn get_u128(&mut self) -> u128;
            fn get_u128_le(&mut self) -> u128;
            fn get_u128_ne(&mut self) -> u128;
            fn get_i128(&mut self) -> i128;
            fn get_i128_le(&mut self) -> i128;
            fn get_i128_ne(&mut self) -> i128;

            fn get_uint(&mut self, nbytes: usize) -> u64;
            fn get_uint_le(&mut self, nbytes: usize) -> u64;
            fn get_uint_ne(&mut self, nbytes: usize) -> u64;
            fn get_int(&mut self, nbytes: usize) -> i64;
            fn get_int_le(&mut self, nbytes: usize) -> i64;
            fn get_int_ne(&mut self, nbytes: usize) -> i64;

            fn get_f32(&mut self) -> f32;
            fn get_f32_le(&mut self) -> f32;
            fn get_f32_ne(&mut self) -> f32;

            fn get_f64(&mut self) -> f64;
            fn get_f64_le(&mut self) -> f64;
            fn get_f64_ne(&mut self) -> f64;

            fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes;
        }

    }
}

unsafe impl<B, C> crate::bytes::BufMut for ContextBuffer<B, C>
where
    B: crate::bytes::BufMut,
{
    delegate::delegate! {
        to self.inner {
            fn remaining_mut(&self) -> usize;
            unsafe fn advance_mut(&mut self, cnt: usize);
            fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice;
            fn has_remaining_mut(&self) -> bool;

            fn put<T: bytes::buf::Buf>(&mut self, src: T)
            where
                Self: Sized;

            fn put_slice(&mut self, src: &[u8]);
            fn put_bytes(&mut self, val: u8, cnt: usize);

            fn put_u8(&mut self, n: u8);
            fn put_i8(&mut self, n: i8);

            fn put_u16(&mut self, n: u16);
            fn put_u16_le(&mut self, n: u16);
            fn put_u16_ne(&mut self, n: u16);
            fn put_i16(&mut self, n: i16);
            fn put_i16_le(&mut self, n: i16);
            fn put_i16_ne(&mut self, n: i16);

            fn put_u32(&mut self, n: u32);
            fn put_u32_le(&mut self, n: u32);
            fn put_u32_ne(&mut self, n: u32);
            fn put_i32(&mut self, n: i32);
            fn put_i32_le(&mut self, n: i32);
            fn put_i32_ne(&mut self, n: i32);

            fn put_u64(&mut self, n: u64);
            fn put_u64_le(&mut self, n: u64);
            fn put_u64_ne(&mut self, n: u64);
            fn put_i64(&mut self, n: i64);
            fn put_i64_le(&mut self, n: i64);
            fn put_i64_ne(&mut self, n: i64);

            fn put_u128(&mut self, n: u128);
            fn put_u128_le(&mut self, n: u128);
            fn put_u128_ne(&mut self, n: u128);
            fn put_i128(&mut self, n: i128);
            fn put_i128_le(&mut self, n: i128);
            fn put_i128_ne(&mut self, n: i128);

            fn put_uint(&mut self, n: u64, nbytes: usize);
            fn put_uint_le(&mut self, n: u64, nbytes: usize);
            fn put_uint_ne(&mut self, n: u64, nbytes: usize);
            fn put_int(&mut self, n: i64, nbytes: usize);
            fn put_int_le(&mut self, n: i64, nbytes: usize);
            fn put_int_ne(&mut self, n: i64, nbytes: usize);

            fn put_f32(&mut self, n: f32);
            fn put_f32_le(&mut self, n: f32);
            fn put_f32_ne(&mut self, n: f32);

            fn put_f64(&mut self, n: f64);
            fn put_f64_le(&mut self, n: f64);
            fn put_f64_ne(&mut self, n: f64);
        }
    }
}
