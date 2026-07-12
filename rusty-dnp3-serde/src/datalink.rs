use std::marker::PhantomData;

use crate::{
    api::CrcCalculator,
    datalink::{
        Direction::{MasterToOutstation, OutstationToMaster},
        Primary::{PrimaryToSecondary, SecondaryToPrimary},
    },
};

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Direction {
    MasterToOutstation,
    OutstationToMaster,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum FrameCountValid {
    Check { count: bool },
    Ignore,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Primary {
    PrimaryToSecondary { frame_cound_valid: FrameCountValid },
    SecondaryToPrimary { data_flow_control: bool },
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct DataLinkFrame {
    pub length: u8,
    pub direction: Direction,
    pub primary: Primary,
    pub source: u16,
    pub destination: u16,
    pub user_data: Vec<u8>,
}

pub enum DeserialisedResult {
    MoreDataRequired,
    JunkBytesDetected { reason: String },
    FrameDetected { data_link_frame: DataLinkFrame },
}

pub struct RustyDnp3Deserialiser<T: CrcCalculator> {
    buffer: Vec<u8>,
    header: Option<DataLinkFrame>,

    _crc_calculator: PhantomData<T>,
}

impl<T: CrcCalculator> RustyDnp3Deserialiser<T> {
    pub fn new() -> Self {
        return Self { header: None, buffer: Vec::new(), _crc_calculator: PhantomData::default() };
    }

    pub fn clear(&mut self) {
        self.header = None;
        self.buffer.clear();
    }

    // This deserialiser is very concervative. It only shaves off small chunks.
    // This is because it assume data can be transmitted over serial at some point and removing too many bytes may cause corrupt data that mimics DNP3 to eat away a valid trailing payload.
    pub fn digest<'a>(&mut self, received_data: &[u8]) -> DeserialisedResult {
        self.buffer.extend(received_data);

        if let Some(_) = Self::discard_bytes(&mut self.buffer, 0) {
            self.clear();
            return DeserialisedResult::JunkBytesDetected { reason: format!("start bytes not detected") };
        }

        let header = match self.header.as_mut() {
            Some(x) => x,
            None => {
                if self.buffer.len() < 3 {
                    return DeserialisedResult::MoreDataRequired;
                }
                let payload_length = self.buffer[2] as usize;

                if payload_length < 5 {
                    self.clear();
                    Self::discard_bytes(&mut self.buffer, 1);
                    return DeserialisedResult::JunkBytesDetected { reason: format!("invalid length {payload_length}") };
                } else if self.buffer.len() < 10 {
                    return DeserialisedResult::MoreDataRequired;
                }

                let received_checksum = u16::from_le_bytes([self.buffer[8], self.buffer[9]]);
                let expected_checksum = T::compute_crc(&self.buffer[0..8]);
                if received_checksum != expected_checksum {
                    self.clear();
                    Self::discard_bytes(&mut self.buffer, 1);
                    return DeserialisedResult::JunkBytesDetected { reason: format!("invalid dnp3 header checksum") };
                }

                let direction = if 0x80 & self.buffer[3] != 0 { MasterToOutstation } else { OutstationToMaster };
                let primary = match (self.buffer[3] & 0x40 != 0, self.buffer[3] & 0x20 != 0, self.buffer[3] & 0x10 != 0) {
                    (false, _, x) => SecondaryToPrimary { data_flow_control: x },
                    (true, true, x) => PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: x } },
                    (true, false, _) => PrimaryToSecondary { frame_cound_valid: FrameCountValid::Ignore },
                };
                let destination = u16::from_le_bytes([self.buffer[4], self.buffer[5]]);
                let source = u16::from_le_bytes([self.buffer[6], self.buffer[7]]);
                let length = self.buffer[2];
                self.buffer.drain(0..10);

                self.header.insert(DataLinkFrame { length, direction, primary, source, destination, user_data: Vec::new() })
            }
        };

        let payload_length = header.length as usize;

        loop {
            let remaining_user_data = payload_length - 5 - header.user_data.len();
            if remaining_user_data == 0 {
                return self
                    .header
                    .take()
                    .map(|x| DeserialisedResult::FrameDetected { data_link_frame: x })
                    .unwrap_or_else(|| DeserialisedResult::MoreDataRequired);
            }

            if remaining_user_data >= 16 && self.buffer.len() >= 18 {
                let received_checksum = u16::from_le_bytes([self.buffer[16], self.buffer[17]]);
                let expected_checksum = T::compute_crc(&self.buffer[0..16]);
                if received_checksum != expected_checksum {
                    self.clear();
                    Self::discard_bytes(&mut self.buffer, 1);
                    return DeserialisedResult::JunkBytesDetected { reason: format!("invalid dnp3 user data checksum") };
                }
                header.user_data.extend(self.buffer.drain(0..16));
                self.buffer.drain(0..2);
            } else if remaining_user_data < 16 && self.buffer.len() >= remaining_user_data + 2 {
                let received_checksum = u16::from_le_bytes([self.buffer[remaining_user_data], self.buffer[remaining_user_data + 1]]);
                let expected_checksum = T::compute_crc(&self.buffer[0..remaining_user_data]);
                if received_checksum != expected_checksum {
                    self.clear();
                    Self::discard_bytes(&mut self.buffer, 1);
                    return DeserialisedResult::JunkBytesDetected { reason: format!("invalid dnp3 user data checksum") };
                }
                header.user_data.extend(self.buffer.drain(0..remaining_user_data));
                self.buffer.drain(0..2);
            } else if remaining_user_data < 16 && self.buffer.len() < remaining_user_data + 2 {
                return DeserialisedResult::MoreDataRequired;
            }
        }
    }

    fn discard_bytes(buffer: &mut Vec<u8>, offset: usize) -> Option<usize> {
        if buffer.len() < offset {
            let drain_size = buffer.len();
            buffer.clear();
            return Some(drain_size);
        }
        if buffer.len() < 2 {
            return None;
        }

        match Self::calculate_discard_bytes(&buffer[offset..]) {
            Some(x) if x > 0 => {
                buffer.drain(0..(offset + x));
                return Some(x);
            }
            _ if offset > 0 => {
                buffer.drain(0..offset);
                return Some(offset);
            }
            _ => return None,
        }
    }

    fn calculate_discard_bytes(buffer: &[u8]) -> Option<usize> {
        if buffer.len() < 2 {
            return None;
        }

        for i in 0..buffer.len() - 1 {
            if buffer[i] == 0x05 && buffer[i + 1] == 0x64 {
                return Some(i);
            }
        }
        return Some(buffer.len() - 1);
    }
}

#[cfg(test)]
mod tests {
    use crate::crc::LutCrcCalculator;

    use super::*;

    #[test]
    fn it_deserialises_a_complete_short_packet() -> Result<(), anyhow::Error> {
        let mut subject = RustyDnp3Deserialiser::<LutCrcCalculator>::new();
        match subject.digest(&[0x05, 0x64, 0x05, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 0xBF, 0xB5]) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome"),
            DeserialisedResult::JunkBytesDetected { reason: _ } => assert!(false, "Unexpected Outcome"),
            DeserialisedResult::FrameDetected { data_link_frame } => assert_eq!(
                data_link_frame,
                DataLinkFrame {
                    length: 5,
                    direction: MasterToOutstation,
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true } },
                    source: 65519,
                    destination: 1,
                    user_data: Vec::new()
                }
            ),
        };

        Ok(())
    }

    #[test]
    fn it_deserialises_a_complete_short_packet_drip_in() -> Result<(), anyhow::Error> {
        let mut subject = RustyDnp3Deserialiser::<LutCrcCalculator>::new();

        for i in 0..9 {
            match subject.digest(&[0x05, 0x64, 0x05, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 0xBF, 0xB5][i..=i]) {
                DeserialisedResult::MoreDataRequired => (),
                DeserialisedResult::JunkBytesDetected { reason: _ } => assert!(false, "Unexpected Outcome"),
                DeserialisedResult::FrameDetected { data_link_frame: _ } => assert!(false, "Unexpected Outcome"),
            }
        }
        match subject.digest(&[0xB5]) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome"),
            DeserialisedResult::JunkBytesDetected { reason: _ } => assert!(false, "Unexpected Outcome"),
            DeserialisedResult::FrameDetected { data_link_frame } => assert_eq!(
                data_link_frame,
                DataLinkFrame {
                    length: 5,
                    direction: MasterToOutstation,
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true } },
                    source: 65519,
                    destination: 1,
                    user_data: Vec::new()
                },
            ),
        }

        Ok(())
    }
}
