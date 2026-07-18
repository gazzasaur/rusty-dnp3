use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustyDnp3Error {
    #[error("failed to calculate dnp3 checksum, {reason}")]
    ChecksumCalculation { reason: String },

    #[error("failed to serialise dnp3 payload, {reason}")]
    SerialisationError { reason: String },

    #[error("failed to deserialise dnp3 payload, {reason}")]
    DeserialisationError { reason: String },

    #[error("a dnp3 violation was detected, {reason}")]
    ValidationError { reason: String },

    #[error("unknown dnp3 error, {reason}")]
    Unknown { reason: String },
}

pub enum Dnp3ObjectList {
    BinaryInputDataPointFlags(BinaryInputDataPointFlags),
    BinaryInputDataPointObject(DataPointObjectList<BinaryInputDataPointObject>),
}

pub enum RangeSpecifier {
    OneOctet(u8),
    TwoOctets(u16),
    FourOctets(u32),
}

impl RangeSpecifier {
    pub fn range_qualifier_code(&self) -> Vec<u8> {
        match self {
            RangeSpecifier::OneOctet(_) => vec![0x00],
            RangeSpecifier::TwoOctets(_) => vec![0x01],
            RangeSpecifier::FourOctets(_) => vec![0x02],
        }
    }

    pub fn octet_size(&self) -> u8 {
        match self {
            RangeSpecifier::OneOctet(_) => 1,
            RangeSpecifier::TwoOctets(_) => 2,
            RangeSpecifier::FourOctets(_) => 4,
        }
    }

    pub fn add(&self, value: usize) -> Result<RangeSpecifier, RustyDnp3Error> {
        let error = || RustyDnp3Error::SerialisationError { reason: format!("start stop range exceeds octet size bounds") };
        match self {
            RangeSpecifier::OneOctet(x) => (value + *x as usize).try_into().map(|y| RangeSpecifier::OneOctet(y)).map_err(|_| error()),
            RangeSpecifier::TwoOctets(x) => (value + *x as usize).try_into().map(|y| RangeSpecifier::TwoOctets(y)).map_err(|_| error()),
            RangeSpecifier::FourOctets(x) => (value + *x as usize).try_into().map(|y| RangeSpecifier::FourOctets(y)).map_err(|_| error()),
        }
    }
}

impl From<RangeSpecifier> for Vec<u8> {
    fn from(value: RangeSpecifier) -> Self {
        (&value).into()
    }
}

impl From<&RangeSpecifier> for Vec<u8> {
    fn from(value: &RangeSpecifier) -> Self {
        match value {
            RangeSpecifier::OneOctet(x) => x.to_le_bytes().to_vec(),
            RangeSpecifier::TwoOctets(x) => x.to_le_bytes().to_vec(),
            RangeSpecifier::FourOctets(x) => x.to_le_bytes().to_vec(),
        }
    }
}

pub enum RangeSpecifierSize {
    OneOctet,
    TwoOctets,
    FourOctets,
}

impl From<&RangeSpecifierSize> for RangeSpecifier {
    fn from(value: &RangeSpecifierSize) -> Self {
        match value {
            RangeSpecifierSize::OneOctet => RangeSpecifier::OneOctet(0),
            RangeSpecifierSize::TwoOctets => RangeSpecifier::TwoOctets(0),
            RangeSpecifierSize::FourOctets => RangeSpecifier::FourOctets(0),
        }
    }
}

impl From<RangeSpecifierSize> for RangeSpecifier {
    fn from(value: RangeSpecifierSize) -> Self {
        match value {
            RangeSpecifierSize::OneOctet => RangeSpecifier::OneOctet(0),
            RangeSpecifierSize::TwoOctets => RangeSpecifier::TwoOctets(0),
            RangeSpecifierSize::FourOctets => RangeSpecifier::FourOctets(0),
        }
    }
}

pub enum DataPointObjectList<T: DataPointObject> {
    NoRange,
    StartAndStop(RangeSpecifier, Vec<T>),

    OneOctetIndex(RangeSpecifierSize, Vec<IndexedDataPointObject<u8, T>>),
    TwoOctetIndex(RangeSpecifierSize, Vec<IndexedDataPointObject<u16, T>>),
    FourOctetIndex(RangeSpecifierSize, Vec<IndexedDataPointObject<u32, T>>),
}

pub trait DataPointObject {}
impl DataPointObject for BinaryInputDataPointObject {}
impl DataPointObject for BinaryInputEventDataPointObject {}

pub trait IndexSize {}
impl IndexSize for u8 {}
impl IndexSize for u16 {}
impl IndexSize for u32 {}

pub struct IndexedDataPointObject<U: IndexSize, T: DataPointObject> {
    pub index: U,
    pub data_point: T,
}

pub enum Dnp3Timestamp {
    None,
    Relative(u64),
    Absolute(u64),
}

pub struct BinaryInputDataPointFlags {
    pub start_index: RangeSpecifier,
    pub flags: Vec<bool>,
}

pub struct BinaryInputDataPointObject {
    pub state: bool,
    pub chatter_filter: bool,
    pub local_forced: bool,
    pub remote_forced: bool,
    pub comm_lost: bool,
    pub restart: bool,
    pub online: bool,
}

pub struct BinaryInputEventDataPointObject {
    pub data_point: BinaryInputDataPointObject,
    pub timestamp: Dnp3Timestamp,
}

pub enum DoubleBitBinaryState {
    Transition, // 0: Transition or Travel
    Close,      // 1: Close or ON
    Trip,       // 2: Trip, Open or OFF
    Abnormal,   // 3: Abnormal or Custom
}

pub struct DoubleBitBinaryInputDataPointObject {
    pub state: DoubleBitBinaryState,
    pub chatter_filter: bool,
    pub local_forced: bool,
    pub remote_forced: bool,
    pub comm_lost: bool,
    pub restart: bool,
    pub online: bool,
}

pub struct DoubleBitBinaryInputEventDataPointObject {
    pub data_point: DoubleBitBinaryInputDataPointObject,
    pub timestamp: Dnp3Timestamp,
}
