use thiserror::Error;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Error, Debug)]
pub enum RustyDnp3Error {
    #[error("failed to calculate dnp3 checksum, {reason}")]
    ChecksumCalculation { reason: String },

    #[error("failed to serialise dnp3 payload, {reason}")]
    SerialisationError { reason: String },

    #[error("a dnp3 violation was detected, {reason}")]
    ValidationError { reason: String },

    #[error("unknown dnp3 error, {reason}")]
    Unknown { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
