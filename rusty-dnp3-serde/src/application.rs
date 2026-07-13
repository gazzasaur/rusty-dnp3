use rusty_dnp3_api::RustyDnp3Error;

pub enum Dnp3Object {
    BinaryInput
}

pub enum Dnp3ObjectList {
    BinaryInput
}

/// Application Requests must be completely contained within a single fragment.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ApplicationFrame {
    first: bool,
    finish: bool,
    sequence_number: u8,
    function_code: u8, // Request or Response with IIN

    // DNP3 Objects
}
