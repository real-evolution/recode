use crate::Encoder;

impl<E> Encoder for Option<E>
where
    E: Encoder,
{
    type Error = E::Error;

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
        match self {
            | Some(ref e) => e.encode(buf),
            | None => Ok(()),
        }
    }
}
