use crate::Decoder;

impl<B, D> Decoder<B> for Option<D>
where
    D: Decoder<B>,
{
    type Error = D::Error;

    fn decode(buf: &mut B) -> Result<Option<D>, Self::Error> {
        D::decode(buf).map(Some)
    }
}
