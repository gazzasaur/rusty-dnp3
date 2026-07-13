use thiserror::Error;

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

pub enum Dnp3Timestamp {
    None,
    Relative(u64),
    Absolute(u64),
}

pub enum DataPointValue {
    BinaryInputDataPoint(BinaryInputDataPointValue),
    BinaryInputEvent(BinaryInputEventValue),

    DoubleBitBinaryInputDataPoint(DoubleBitBinaryInputDataPointValue),
    DoubleBitBinaryInputEvenValue(DoubleBitBinaryInputEvent),
}

pub struct IndexedDataPoint<T> {
    pub index: u32,
    pub data_point: T
}

pub struct BinaryInputDataPointValue {
    pub state: bool,
    pub chatter_filter: bool,
    pub local_forced: bool,
    pub remote_forced: bool,
    pub comm_lost: bool,
    pub restart: bool,
    pub online: bool,
}

pub struct BinaryInputEventValue {
    pub data_point: BinaryInputDataPointValue,
    pub timestamp: Dnp3Timestamp,
}

pub enum DoubleBitBinaryState {
    Transition, // 0: Transition or Travel
    Close,      // 1: Close or ON
    Trip,       // 2: Trip, Open or OFF
    Abnormal,   // 3: Abnormal or Custom
}

pub struct DoubleBitBinaryInputDataPointValue {
    pub state: DoubleBitBinaryState,
    pub chatter_filter: bool,
    pub local_forced: bool,
    pub remote_forced: bool,
    pub comm_lost: bool,
    pub restart: bool,
    pub online: bool,
}

pub struct DoubleBitBinaryInputEvent {
    pub data_point: DoubleBitBinaryInputDataPointValue,
    pub timestamp: Dnp3Timestamp,
}
