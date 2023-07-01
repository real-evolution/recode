pub use ux::{i24, i40, i48, i56, u24, u40, u48, u56};

macro_rules! impl_ux {
    ($t:ty; size: $s:literal; rep: $r:ty ) => {
        impl crate::Encoder for $t {
            type Error = std::convert::Infallible;
            type Input = $t;

            fn encode<B: bytes::BufMut>(
                input: &Self::Input,
                buf: &mut B,
            ) -> Result<(), Self::Error> {
                let bytes = &<$r>::from(*input).to_be_bytes()[..$s];

                buf.put_slice(bytes);

                Ok(())
            }
        }

        impl<B> crate::Decoder<B> for $t
        where
            B: crate::bytes::Buf,
        {
            type Error = crate::Error;

            fn decode(buf: &mut B) -> Result<Self, Self::Error> {
                const REPR_LEN: usize = std::mem::size_of::<$r>();

                if buf.remaining() < $s {
                    return Err(crate::Error::BytesNeeded {
                        needed: $s - buf.remaining(),
                        full_len: $s,
                        available: buf.remaining(),
                    });
                }

                let mut be_repr = [0u8; REPR_LEN];
                buf.copy_to_slice(&mut be_repr[(REPR_LEN - $s)..REPR_LEN]);

                Ok(<$t>::new(<$r>::from_be_bytes(be_repr)))
            }
        }
    };
}

impl_ux!(i24; size: 3; rep: i32);
impl_ux!(u24; size: 3; rep: u32);

impl_ux!(i40; size: 5; rep: i64);
impl_ux!(u40; size: 5; rep: u64);

impl_ux!(i48; size: 6; rep: i64);
impl_ux!(u48; size: 6; rep: u64);

impl_ux!(i56; size: 7; rep: i64);
impl_ux!(u56; size: 7; rep: u64);
