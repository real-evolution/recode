use crate::Encoder;

impl<E> Encoder for Option<E>
where
    E: Encoder<Input = E>,
{
    type Error = E::Error;
    type Input = Option<E>;

    fn encode<B: bytes::BufMut>(
        input: &Option<E>,
        buf: &mut B,
    ) -> Result<(), Self::Error> {
        match input {
            | Some(ref e) => E::encode(e, buf),
            | None => Ok(()),
        }
    }
}
