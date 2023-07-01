use crate::Encoder;

impl<B, E> Encoder<B> for Option<E>
where
    E: Encoder<B>,
{
    type Error = E::Error;

    fn encode(item: &Option<E>, buf: &mut B) -> Result<(), Self::Error> {
        match item {
            | Some(e) => E::encode(e, buf),
            | None => Ok(()),
        }
    }
}
