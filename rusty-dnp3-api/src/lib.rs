use std::fmt::Debug;

use thiserror::Error;

use crate::DataPointObjectList::{NoRange, StartAndStop};

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

fn a() {
    let b = Dnp3ObjectList::BinaryInputDataPointObject(StartAndStop(RangeSpecifier::OneOctet(0), vec![]));
    match &b {
        Dnp3ObjectList::BinaryInputDataPointObject(data_point_object_list) => c(data_point_object_list),
    };
}

fn c<'a, T: DataPointObject>(f: &'a DataPointObjectList<T>) -> Option<&'a Vec<IndexedDataPointObject<u8, T>>> {
    match f {
        NoRange => return None,
        StartAndStop(range_specifier, items) => return None,
        DataPointObjectList::OneOctetIndex(range_specifier_size, indexed_data_point_objects) => return Some(indexed_data_point_objects),
        DataPointObjectList::TwoOctetIndex(range_specifier_size, indexed_data_point_objects) => return None,
        DataPointObjectList::FourOctetIndex(range_specifier_size, indexed_data_point_objects) => return None,
    };
}

pub enum Dnp3ObjectList {
    BinaryInputDataPointObject(DataPointObjectList<BinaryInputDataPointObject>),
}

pub enum RangeSpecifier {
    OneOctet(u8),
    TwoOctets(u16),
    FourOctets(u32),
}

pub enum RangeSpecifierSize {
    OneOctet,
    TwoOctets,
    FourOctets,
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
