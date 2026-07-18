use rusty_dnp3_api::{BinaryInputDataPointFlags, BinaryInputDataPointObject, RustyDnp3Error};

use crate::application::common::{PointObjectDeserialiser, PointObjectSerialiser};

impl PointObjectSerialiser<BinaryInputDataPointObject> for BinaryInputDataPointObject {
    fn serialise(object: &BinaryInputDataPointObject) -> Result<Vec<u8>, RustyDnp3Error> {
        let mut value = 0;
        value |= if object.state { 0x80 } else { 0x00 };
        value |= if object.chatter_filter { 0x20 } else { 0x00 };
        value |= if object.local_forced { 0x10 } else { 0x00 };
        value |= if object.remote_forced { 0x08 } else { 0x00 };
        value |= if object.comm_lost { 0x04 } else { 0x00 };
        value |= if object.restart { 0x02 } else { 0x00 };
        value |= if object.online { 0x01 } else { 0x00 };
        Ok(vec![value])
    }
}

impl PointObjectDeserialiser<BinaryInputDataPointObject> for BinaryInputDataPointObject {
    fn deserialise<'a>(data: &'a [u8]) -> Result<(BinaryInputDataPointObject, &'a [u8]), RustyDnp3Error> {
        if data.len() < 3 {
            return Err(RustyDnp3Error::DeserialisationError { reason: format!("No data while parsing binary input with flags") });
        }
        // Ignore group and variation
        Ok((
            BinaryInputDataPointObject {
                state: data[2] & 0x80 != 0,
                chatter_filter: data[2] & 0x20 != 0,
                local_forced: data[2] & 0x10 != 0,
                remote_forced: data[2] & 0x08 != 0,
                comm_lost: data[2] & 0x04 != 0,
                restart: data[2] & 0x02 != 0,
                online: data[2] & 0x01 != 0,
            },
            &data[3..],
        ))
    }
}

pub fn calculate_object_size_binary_input_data_point_flags_objects(
    remaining_bytes: usize,
    object: &BinaryInputDataPointFlags,
) -> Result<usize, RustyDnp3Error> {
    let header_size = (2 * object.start_index.octet_size() + 3) as usize; // 3 = Group + Variation + Qualifier
    let all_payload_bytes = object.flags.len() / 8 + if object.flags.len() % 8 != 0 { 1 } else { 0 };
    match (remaining_bytes, all_payload_bytes) {
        _ if remaining_bytes <= header_size => Ok(0),
        _ if remaining_bytes >= header_size + all_payload_bytes => Ok(object.flags.len()),
        _ => Ok(8 * (remaining_bytes - header_size)),
    }
}

pub fn serialise_binary_input_data_point_flags_objects(object: &BinaryInputDataPointFlags) -> Result<Vec<u8>, RustyDnp3Error> {
    let prefix_code = object.start_index.range_qualifier_code();
    let stop_index = object.start_index.add(object.flags.len())?;
    let start_index_bytes: Vec<u8> = (&object.start_index).into();
    let stop_index_bytes: Vec<u8> = stop_index.into();

    let mut buffer = vec![0x01, 0x01];
    buffer.extend_from_slice(&prefix_code);
    buffer.extend_from_slice(&start_index_bytes);
    buffer.extend_from_slice(&stop_index_bytes);

    let mut flags_buffer = Vec::with_capacity(object.flags.len() / 8 + if object.flags.len() % 8 != 0 { 1 } else { 0 });
    for i in 0..object.flags.len() {
        let byte_index = i / 8;
        if byte_index >= flags_buffer.len() {
            flags_buffer.push(0);
        }
        if object.flags[i] {
            flags_buffer[byte_index] |= 1 << (i % 8)
        }
    }
    buffer.extend_from_slice(&flags_buffer);
    Ok(buffer)
}

pub fn deserialise_binary_input_data_point_flags_objects<'a>(data: &'a[u8]) -> Result<(BinaryInputDataPointFlags, &'a[u8]), RustyDnp3Error> {
    if data.len() < 3 {
        return Err(RustyDnp3Error::DeserialisationError { reason: format!("Not enough data while parsing binary input packed") });
    }
    // Ignore group and variation
    
    let prefix_code = object.start_index.range_qualifier_code();
    let stop_index = object.start_index.add(object.flags.len())?;
    let start_index_bytes: Vec<u8> = (&object.start_index).into();
    let stop_index_bytes: Vec<u8> = stop_index.into();

    let mut buffer = vec![0x01, 0x01];
    buffer.extend_from_slice(&prefix_code);
    buffer.extend_from_slice(&start_index_bytes);
    buffer.extend_from_slice(&stop_index_bytes);

    let mut flags_buffer = Vec::with_capacity(object.flags.len() / 8 + if object.flags.len() % 8 != 0 { 1 } else { 0 });
    for i in 0..object.flags.len() {
        let byte_index = i / 8;
        if byte_index >= flags_buffer.len() {
            flags_buffer.push(0);
        }
        if object.flags[i] {
            flags_buffer[byte_index] |= 1 << (i % 8)
        }
    }
    buffer.extend_from_slice(&flags_buffer);
    Ok(buffer)
}
