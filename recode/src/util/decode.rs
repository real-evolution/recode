use crate::Decoder;

impl<D> Decoder for Option<D>
where
    D: Decoder<Output = D>,
{
    type Error = D::Error;
    type Output = Option<D::Output>;

    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self::Output, Self::Error> {
        D::decode(buf).map(Some)
    }
}
