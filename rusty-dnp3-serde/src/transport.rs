use rusty_dnp3_api::RustyDnp3Error;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TransportFrame<'a> {
    first: bool,
    finish: bool,
    sequence_number: u8,

    user_data: &'a [u8],
}

impl<'a> TransportFrame<'a> {
    pub fn new(first: bool, finish: bool, sequence_number: u8, user_data: &'a [u8]) -> Result<Self, RustyDnp3Error> {
        if sequence_number >= 64 {
            return Err(RustyDnp3Error::ValidationError { reason: format!("the transport function sequence number must be less than 64") });
        }
        if user_data.len() >= 250 {
            return Err(RustyDnp3Error::ValidationError { reason: format!("the transport function user data cannot exceed 249 bytes") });
        }
        Ok(Self { first, finish, sequence_number, user_data })
    }
}

impl<'a> From<TransportFrame<'a>> for Vec<u8> {
    fn from(value: TransportFrame<'a>) -> Self {
        Self::from(&value)
    }
}

impl<'a> From<&TransportFrame<'a>> for Vec<u8> {
    fn from(value: &TransportFrame<'a>) -> Self {
        let mut buffer = Self::with_capacity(value.user_data.len() + 1);
        buffer.push(
            match (value.finish, value.first) {
                (true, true) => 0xC0,
                (true, false) => 0x80,
                (false, true) => 0x40,
                (false, false) => 0x00,
            } | value.sequence_number,
        );
        buffer.extend_from_slice(value.user_data);
        buffer
    }
}

impl<'a> TryFrom<&'a [u8]> for TransportFrame<'a> {
    type Error = RustyDnp3Error;

    fn try_from(value: &'a [u8]) -> Result<Self, RustyDnp3Error> {
        if value.len() < 1 || value.len() > 250 {
            return Err(RustyDnp3Error::SerialisationError {
                reason: format!("dnp3 transport frames must be betwen 1 and 250 bytes inclusive but for {} bytes", value.len()),
            });
        }
        Ok(TransportFrame { first: value[0] & 0x40 != 0, finish: value[0] & 0x80 != 0, sequence_number: value[0] & 0x3F, user_data: &value[1..] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialises_and_deserialises() -> Result<(), anyhow::Error> {
        let user_data = "Hello".as_bytes();
        let frame = TransportFrame::new(true, true, 42, user_data)?;

        let frame_data = Vec::<u8>::from(&frame);
        let received_frame = TransportFrame::try_from(frame_data.as_slice())?;

        assert_eq!(frame, received_frame);

        Ok(())
    }
}
