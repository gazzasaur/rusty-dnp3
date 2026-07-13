use std::marker::PhantomData;

use rusty_dnp3_api::RustyDnp3Error;

use crate::{
    api::CrcCalculator,
    datalink::{
        DataLinkReqeustFunctionCode::{ConfirmedUserData, RequestLinkStatus, ResetLinkStates, TestLinkStates, UnconfirmedUserData},
        DataLinkResponseFunctionCode::{Ack, LinkStatus, Nack, NotSupported},
        Direction::{MasterToOutstation, OutstationToMaster},
        Primary::{PrimaryToSecondary, SecondaryToPrimary},
    },
};

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum DataLinkReqeustFunctionCode {
    ResetLinkStates,
    TestLinkStates,
    ConfirmedUserData,
    UnconfirmedUserData,
    RequestLinkStatus,
    Unknown(u8),
}

impl From<u8> for DataLinkReqeustFunctionCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ResetLinkStates,
            2 => TestLinkStates,
            3 => ConfirmedUserData,
            4 => UnconfirmedUserData,
            9 => RequestLinkStatus,
            x => DataLinkReqeustFunctionCode::Unknown(x),
        }
    }
}

impl From<&DataLinkReqeustFunctionCode> for u8 {
    fn from(value: &DataLinkReqeustFunctionCode) -> Self {
        match value {
            ResetLinkStates => 0,
            TestLinkStates => 2,
            ConfirmedUserData => 3,
            UnconfirmedUserData => 4,
            RequestLinkStatus => 9,
            DataLinkReqeustFunctionCode::Unknown(x) => *x,
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum DataLinkResponseFunctionCode {
    Ack,
    Nack,
    LinkStatus,
    NotSupported,
    Unknown(u8),
}

impl From<u8> for DataLinkResponseFunctionCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Ack,
            1 => Nack,
            11 => LinkStatus,
            15 => NotSupported,
            x => DataLinkResponseFunctionCode::Unknown(x),
        }
    }
}

impl From<&DataLinkResponseFunctionCode> for u8 {
    fn from(value: &DataLinkResponseFunctionCode) -> Self {
        match value {
            Ack => 0,
            Nack => 1,
            LinkStatus => 11,
            NotSupported => 15,
            DataLinkResponseFunctionCode::Unknown(x) => *x,
        }
    }
}

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
    PrimaryToSecondary { frame_cound_valid: FrameCountValid, function_code: DataLinkReqeustFunctionCode },
    SecondaryToPrimary { data_flow_control: bool, function_code: DataLinkResponseFunctionCode },
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

impl DataLinkFrame {
    pub fn new(direction: Direction, primary: Primary, source: u16, destination: u16, user_data: Vec<u8>) -> Result<Self, RustyDnp3Error> {
        if user_data.len() > 250 {
            return Err(RustyDnp3Error::ValidationError { reason: format!("data link user data length is greater than the allowed 250 bytes") });
        }
        let length = 5 + user_data.len() as u8;
        Ok(Self { length, direction, primary, source, destination, user_data })
    }
}

pub struct RustyDnp3Serialiser<T: CrcCalculator> {
    _crc_calculator: PhantomData<T>,
}

impl<T: CrcCalculator> RustyDnp3Serialiser<T> {
    pub fn new() -> Self {
        Self { _crc_calculator: PhantomData }
    }
}

impl<T: CrcCalculator> RustyDnp3Serialiser<T> {
    pub fn serialise(&self, frame: &DataLinkFrame) -> Result<Vec<u8>, RustyDnp3Error> {
        if frame.user_data.len() > 250 {
            return Err(RustyDnp3Error::SerialisationError { reason: format!("data link user data length is greater than the allowed 250 bytes") });
        }
        if frame.length < 5 {
            return Err(RustyDnp3Error::SerialisationError { reason: format!("dnp3 payload length cannot be less than 5 but was {}", frame.length) });
        }
        if frame.length as usize != frame.user_data.len() + 5 {
            return Err(RustyDnp3Error::SerialisationError { reason: format!("dnp3 payload length does not match user data length") });
        }

        let number_of_user_data_frames = (frame.user_data.len() / 16) + (if frame.user_data.len() % 16 != 0 { 2 } else { 0 });

        let mut buffer = Vec::with_capacity(frame.length as usize + 5 + 2 * number_of_user_data_frames);
        buffer.extend_from_slice(&[0x05, 0x64, frame.length]);
        buffer.push(if matches!(frame.direction, MasterToOutstation) { 0x80 } else { 0x00 });
        buffer[3] |= match &frame.primary {
            PrimaryToSecondary { frame_cound_valid: FrameCountValid::Ignore, function_code } => 0x40 | Into::<u8>::into(function_code) & 0x0F,
            PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code } => 0x70 | Into::<u8>::into(function_code) & 0x0F,
            PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: false }, function_code } => 0x60 | Into::<u8>::into(function_code) & 0x0F,
            SecondaryToPrimary { data_flow_control: true, function_code } => 0x10 | Into::<u8>::into(function_code) & 0x0F,
            SecondaryToPrimary { data_flow_control: false, function_code } => 0x00 | Into::<u8>::into(function_code) & 0x0F,
        };
        buffer.extend_from_slice(&frame.destination.to_le_bytes());
        buffer.extend_from_slice(&frame.source.to_le_bytes());
        let data_chunk = buffer[0..8].to_vec();

        buffer.extend_from_slice(&T::compute_crc(&data_chunk).to_le_bytes());

        for i in 0..number_of_user_data_frames {
            let start_user_data_index = i * 16;
            let finish_user_data_index = match (i + 1) * 16 {
                x if x <= frame.user_data.len() => x,
                _ => start_user_data_index + frame.user_data.len() % 16,
            };
            buffer.extend_from_slice(&frame.user_data[start_user_data_index..finish_user_data_index]);
            let crc_value = T::compute_crc(&frame.user_data[start_user_data_index..finish_user_data_index]);
            buffer.extend_from_slice(&crc_value.to_le_bytes());
        }

        return Ok(buffer);
    }
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
                    (false, _, x) => SecondaryToPrimary { data_flow_control: x, function_code: (self.buffer[3] & 0x0F).into() },
                    (true, true, x) => {
                        PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: x }, function_code: (self.buffer[3] & 0x0F).into() }
                    }
                    (true, false, _) => PrimaryToSecondary { frame_cound_valid: FrameCountValid::Ignore, function_code: (self.buffer[3] & 0x0F).into() },
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
    use crate::
        crc::{LutCrcCalculator, compute_checksum_lut}
    ;

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
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
                    source: 65519,
                    destination: 1,
                    user_data: Vec::new()
                }
            ),
        };
        Ok(())
    }

    #[test]
    fn it_fails_on_an_invlid_header_checksum() -> Result<(), anyhow::Error> {
        let mut subject = RustyDnp3Deserialiser::<LutCrcCalculator>::new();
        match subject.digest(&[0x05, 0x64, 0x05, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 0xBF, 0xB4]) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome: More Data Required"),
            DeserialisedResult::JunkBytesDetected { reason } => assert_eq!(reason, "invalid dnp3 header checksum"),
            DeserialisedResult::FrameDetected { data_link_frame: _ } => assert!(false, "Unexpected Outcome: Frame Detected"),
        };
        Ok(())
    }

    #[test]
    fn it_fails_on_an_invlid_data_checksum() -> Result<(), anyhow::Error> {
        let mut subject = RustyDnp3Deserialiser::<LutCrcCalculator>::new();
        match subject.digest(&[0x05, 0x64, 0x06, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 0xEF, 0x26, 0x00, 0x01, 0x02]) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome: More Data Required"),
            DeserialisedResult::JunkBytesDetected { reason } => assert_eq!(reason, "invalid dnp3 user data checksum"),
            DeserialisedResult::FrameDetected { data_link_frame: _ } => assert!(false, "Unexpected Outcome: Frame Detected"),
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
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
                    source: 65519,
                    destination: 1,
                    user_data: Vec::new()
                },
            ),
        }
        Ok(())
    }

    #[test]
    fn it_deserialises_a_complete_short_packet_with_user_data() -> Result<(), anyhow::Error> {
        let mut subject = RustyDnp3Deserialiser::<LutCrcCalculator>::new();

        let mut data = vec![0x05, 0x64, 0x05 + 16, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 126, 205];
        let mut user_data: Vec<u8> = (0u8..16u8).collect();
        user_data.extend_from_slice(&compute_checksum_lut(&user_data).to_le_bytes());
        data.extend_from_slice(&user_data);

        match subject.digest(&data) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome: More Data Required"),
            DeserialisedResult::JunkBytesDetected { reason } => assert!(false, "Unexpected Outcome: {reason}"),
            DeserialisedResult::FrameDetected { data_link_frame } => assert_eq!(
                data_link_frame,
                DataLinkFrame {
                    length: 21,
                    direction: MasterToOutstation,
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
                    source: 65519,
                    destination: 1,
                    user_data: (0..16).collect()
                },
            ),
        }

        let mut data = vec![0x05, 0x64, 0x05 + 32, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 61, 68];
        user_data.extend_from_slice(&(16u8..32u8).collect::<Vec<u8>>());
        user_data.extend_from_slice(&compute_checksum_lut(&(16u8..32u8).collect::<Vec<u8>>()).to_le_bytes());
        data.extend_from_slice(&user_data);

        match subject.digest(&data) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome: More Data Required"),
            DeserialisedResult::JunkBytesDetected { reason } => assert!(false, "Unexpected Outcome: {reason}"),
            DeserialisedResult::FrameDetected { data_link_frame } => assert_eq!(
                data_link_frame,
                DataLinkFrame {
                    length: 37,
                    direction: MasterToOutstation,
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
                    source: 65519,
                    destination: 1,
                    user_data: (0..32).collect()
                },
            ),
        }

        let mut data = vec![0x05, 0x64, 0x05 + 39, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 6, 107];
        user_data.extend_from_slice(&(32u8..39u8).collect::<Vec<u8>>());
        user_data.extend_from_slice(&compute_checksum_lut(&(32u8..39u8).collect::<Vec<u8>>()).to_le_bytes());
        data.extend_from_slice(&user_data);

        match subject.digest(&data) {
            DeserialisedResult::MoreDataRequired => assert!(false, "Unexpected Outcome: More Data Required"),
            DeserialisedResult::JunkBytesDetected { reason } => assert!(false, "Unexpected Outcome: {reason}"),
            DeserialisedResult::FrameDetected { data_link_frame } => assert_eq!(
                data_link_frame,
                DataLinkFrame {
                    length: 44,
                    direction: MasterToOutstation,
                    primary: PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
                    source: 65519,
                    destination: 1,
                    user_data: (0..39).collect()
                },
            ),
        }
        Ok(())
    }

    #[test]
    fn it_serialises_a_short_packet() -> Result<(), anyhow::Error> {
        let subject = RustyDnp3Serialiser::<LutCrcCalculator>::new();
        let buffer = subject.serialise(&DataLinkFrame::new(
            MasterToOutstation,
            PrimaryToSecondary { frame_cound_valid: FrameCountValid::Check { count: true }, function_code: TestLinkStates },
            0xFFEF,
            0x0001,
            vec![],
        )?)?;
        assert_eq!(buffer, &[0x05, 0x64, 0x05, 0xF2, 0x01, 0x00, 0xEF, 0xFF, 0xBF, 0xB5]);

        Ok(())
    }
}
