use rusty_dnp3_api::RustyDnp3Error;

pub trait PointObjectSerialiser<T> {
    fn serialise(object: &T) -> Result<Vec<u8>, RustyDnp3Error>;
}

pub trait PointObjectDeserialiser<T> {
    fn deserialise<'a>(data: &'a[u8]) -> Result<(T, &'a[u8]), RustyDnp3Error>;
}
