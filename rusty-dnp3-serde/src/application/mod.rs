pub mod bi;
pub mod common;

use rusty_dnp3_api::{BinaryInputDataPointObject, DataPointObject, DataPointObjectList, Dnp3ObjectList, RangeSpecifier, RustyDnp3Error};

use crate::application::{
    bi::{calculate_object_size_binary_input_data_point_flags_objects, serialise_binary_input_data_point_flags_objects},
    common::PointObjectSerialiser,
};

pub fn calculate_object_size(remaining_bytes: usize, objects: &Dnp3ObjectList) -> Result<usize, RustyDnp3Error> {
    match objects {
        Dnp3ObjectList::BinaryInputDataPointFlags(object) => calculate_object_size_binary_input_data_point_flags_objects(remaining_bytes, &object),
        Dnp3ObjectList::BinaryInputDataPointObject(point_objects) => {
            calculate_object_size_data_point_objects::<BinaryInputDataPointObject>(remaining_bytes, point_objects)
        }
    }
}

pub fn serialise(objects: &Dnp3ObjectList) -> Result<Vec<u8>, RustyDnp3Error> {
    match objects {
        Dnp3ObjectList::BinaryInputDataPointFlags(object) => serialise_binary_input_data_point_flags_objects(object),
        Dnp3ObjectList::BinaryInputDataPointObject(point_objects) => serialise_data_point_objects::<BinaryInputDataPointObject>(point_objects),
    }
}

pub fn calculate_object_size_data_point_objects<T: DataPointObject + PointObjectSerialiser<T>>(
    remaining_bytes: usize,
    data_point_objects: &DataPointObjectList<T>,
) -> Result<usize, RustyDnp3Error> {
    Ok(0)
}

pub fn serialise_data_point_objects<T: DataPointObject + PointObjectSerialiser<T>>(
    data_point_objects: &DataPointObjectList<T>,
) -> Result<Vec<u8>, RustyDnp3Error> {
    match data_point_objects {
        DataPointObjectList::NoRange => Ok(vec![0x06]),

        DataPointObjectList::StartAndStop(_, items) if items.len() == 0 => {
            return Err(RustyDnp3Error::SerialisationError { reason: format!("attempted to serialise an empty list") });
        }
        DataPointObjectList::StartAndStop(range_specifier, items) => {
            let start_index_bytes: Vec<u8> = range_specifier.add(items.len() - 1)?.into();
            let stop_index_bytes: Vec<u8> = range_specifier.add(items.len() - 1)?.into();

            let mut buffer = range_specifier.range_qualifier_code();
            buffer.extend_from_slice(&start_index_bytes);
            buffer.extend_from_slice(&stop_index_bytes);
            for item in items {
                buffer.extend_from_slice(T::serialise(item)?.as_slice());
            }
            Ok(buffer)
        }
        DataPointObjectList::OneOctetIndex(range_specifier_size, items) => {
            let range_specifier: RangeSpecifier = range_specifier_size.into();
            let mut buffer = range_specifier.range_qualifier_code();
            buffer[0] |= 0x10;

            let count_bytes: Vec<u8> = range_specifier.add(items.len())?.into();
            buffer.extend_from_slice(count_bytes.as_slice());
            for item in items {
                buffer.extend_from_slice(&vec![item.index]);
                buffer.extend_from_slice(T::serialise(&item.data_point)?.as_slice());
            }
            Ok(buffer)
        }
        DataPointObjectList::TwoOctetIndex(range_specifier_size, items) => {
            let range_specifier: RangeSpecifier = range_specifier_size.into();
            let mut buffer = range_specifier.range_qualifier_code();
            buffer[0] |= 0x20;

            let count_bytes: Vec<u8> = range_specifier.add(items.len())?.into();
            buffer.extend_from_slice(count_bytes.as_slice());
            for item in items {
                buffer.extend_from_slice(&item.index.to_le_bytes());
                buffer.extend_from_slice(T::serialise(&item.data_point)?.as_slice());
            }
            Ok(buffer)
        }
        DataPointObjectList::FourOctetIndex(range_specifier_size, items) => {
            let range_specifier: RangeSpecifier = range_specifier_size.into();
            let mut buffer = range_specifier.range_qualifier_code();
            buffer[0] |= 0x30;

            let count_bytes: Vec<u8> = range_specifier.add(items.len())?.into();
            buffer.extend_from_slice(count_bytes.as_slice());
            for item in items {
                buffer.extend_from_slice(&item.index.to_le_bytes());
                buffer.extend_from_slice(T::serialise(&item.data_point)?.as_slice());
            }
            Ok(buffer)
        }
    }
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

#[cfg(test)]
mod tests {
    use rusty_dnp3_api::BinaryInputDataPointFlags;

    use super::*;

    #[test]
    fn it_creates_a_g1v1_payload() -> Result<(), anyhow::Error> {
        let subject_data = Dnp3ObjectList::BinaryInputDataPointFlags(BinaryInputDataPointFlags {
            start_index: RangeSpecifier::OneOctet(0),
            flags: vec![true, false, true, true],
        });
        let serialised_data = serialise(&subject_data)?;
        assert_eq!(serialised_data, vec![0x01, 0x01, 0x00, 0x00, 0x04, 0x0D]);
        assert_eq!(calculate_object_size(100, &subject_data)?, 4);
        assert_eq!(calculate_object_size(6, &subject_data)?, 4);
        assert_eq!(calculate_object_size(5, &subject_data)?, 0);
        assert_eq!(calculate_object_size(4, &subject_data)?, 0);
        assert_eq!(calculate_object_size(3, &subject_data)?, 0);
        assert_eq!(calculate_object_size(2, &subject_data)?, 0);
        assert_eq!(calculate_object_size(1, &subject_data)?, 0);
        assert_eq!(calculate_object_size(0, &subject_data)?, 0);
        Ok(())
    }

    #[test]
    fn it_creates_a_long_g1v1_payload() -> Result<(), anyhow::Error> {
        let mut subject_inner_data = BinaryInputDataPointFlags { start_index: RangeSpecifier::OneOctet(0), flags: vec![true, false, true, true] };
        for _ in 0..23 {
            subject_inner_data.flags.extend_from_slice(&[false, true, true]);
        }

        let subject_data = Dnp3ObjectList::BinaryInputDataPointFlags(subject_inner_data);
        let serialised_data = serialise(&subject_data)?;
        assert_eq!(serialised_data, vec![0x01, 0x01, 0x00, 0x00, 0x49, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0xB6, 1]);
        assert_eq!(calculate_object_size(100, &subject_data)?, 73);
        assert_eq!(calculate_object_size(6, &subject_data)?, 8);
        assert_eq!(calculate_object_size(5, &subject_data)?, 0);

        Ok(())
    }
}
