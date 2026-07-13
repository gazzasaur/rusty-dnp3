use rusty_dnp3_api::RustyDnp3Error;

pub enum BareDnp3ObjectList {
    BinaryInput,
}

pub enum IndexedDnp3ObjectList {
    BinaryInput,
}

pub enum Dnp3DataPointValue {
    BinaryInputWithoutFlags {
        state: bool,
    },
    BinaryInputWithFlags {
        state: bool,
        chatter_filter: bool,
        local_forced: bool,
        remote_forced: bool,
        comm_lost: bool,
        restart: bool,
        online: bool,
    },

    BinaryInputEventWithoutTime {
        state: bool,
        chatter_filter: bool,
        local_forced: bool,
        remote_forced: bool,
        comm_lost: bool,
        restart: bool,
        online: bool,
    },
    BinaryInputEventWithTime {
        state: bool,
        chatter_filter: bool,
        local_forced: bool,
        remote_forced: bool,
        comm_lost: bool,
        restart: bool,
        online: bool,
        timestamp: u64,
    },
}

/// Virtual Addresses are not Supported as Group 102 is not supported or required under any class.
/// Variable format qualifiers are not supported as no types that use these are supported.
pub enum Dnp3ObjectList {
    NoPrefixOneOctetStartAndStop(u8, u8, Dnp3ObjectList),   // Preferred, regular object data
    NoPrefixTwoOctetStartAndStop(u16, u16, Dnp3ObjectList), // Preferred, regular object data
    NoPrefixFourOctetStartAndStop(u32, u32, Dnp3ObjectList),

    NoPrefixNoRange, // All Values, mMandatory for Polls

    NoPrefixOneOctetCount(u8),  // Preferred, single values like time and date
    NoPrefixTwoOctetCount(u16), // Preferred, single values like time and date
    NoPrefixFourOctetCount(u32),

    OneOctetIndexPrefixOneOctetCount(u8, u8, IndexedDnp3ObjectList), // Preferred, regular object data
    OneOctetIndexPrefixTwoOctetCount(u8, u16, IndexedDnp3ObjectList),
    OneOctetIndexPrefixFourOctetCount(u8, u32, IndexedDnp3ObjectList),

    TwoOctetIndexPrefixOneOctetCount(u16, u8, IndexedDnp3ObjectList),
    TwoOctetIndexPrefixTwoOctetCount(u16, u16, IndexedDnp3ObjectList), // Preferred, regular object data
    TwoOctetIndexPrefixFourOctetCount(u16, u32, IndexedDnp3ObjectList),

    FourOctetIndexPrefixOneOctetCount(u32, u8, IndexedDnp3ObjectList),
    FourOctetIndexPrefixTwoOctetCount(u32, u16, IndexedDnp3ObjectList),
    FourOctetIndexPrefixFourOctetCount(u32, u32, IndexedDnp3ObjectList),
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
