use super::{Encodable, FrameWithControl};
use crate::address::Address;
use crate::control::Control;
use thiserror::Error;

/// M-Bus Long Frame
///
/// An M-Bus long frame is a variable-length frame used for transmitting user
/// data and control information.
///
/// The format of a long frame is defined in EN 60870-5-2 (§3.2) as a frame
/// with variable length in the FT 1.2 format.
#[derive(Debug, Clone)]
pub struct LongFrame {
    /// Start byte (0x68)
    ///
    /// The start byte marks the beginning of the long frame.
    start1: u8,

    /// Length of the user data
    length1: u8,

    /// Length of the user data, again
    length2: u8,

    /// Start byte, again (0x68)
    start2: u8,

    /// Control
    ///
    /// The control byte is used to indicate the type of frame.
    control: Control,

    /// Address
    ///
    /// The address byte specifies the address of the addressed slave.
    address: Address,

    /// User data
    data: Vec<u8>,

    /// Checksum
    ///
    /// The checksum byte is used for error detection.
    ///
    /// The checksum is calculated as the sum of all bytes in the frame,
    /// excluding the start and end bytes, modulo 256.
    checksum: u8,

    /// End byte (0x16)
    ///
    /// The end byte marks the end of the long frame.
    end: u8,
}

/// Start byte of an M-Bus long frame
const START_BYTE: u8 = 0x68;

/// End byte of an M-Bus long frame
const END_BYTE: u8 = 0x16;

/// Minimum length of an M-Bus long frame
///
/// The minimum length of an M-Bus long frame is 8 bytes, which
/// corresponds to a frame with no user data.
const MIN_LENGTH: usize = 8; // 6 + 0 + 2

/// Maximum length of an M-Bus long frame
///
/// The maximum length of an M-Bus long frame is 259 bytes, which
/// corresponds to a frame with 253 bytes of user data
const MAX_LENGTH: usize = 259; // 6 + 253 + 2

const START_1_INDEX: usize = 0;
const LENGTH_1_INDEX: usize = 1;
const LENGTH_2_INDEX: usize = 2;
const START_2_INDEX: usize = 3;
const CONTROL_INDEX: usize = 4;
const ADDRESS_INDEX: usize = 5;
const DATA_START_INDEX: usize = 6;

/// Implementation of the M-Bus long frame
impl LongFrame {
    pub fn new(control: Control, address: Address, data: &[u8]) -> Self {
        let length = 2 + data.len() as u8;

        Self {
            start1: START_BYTE,
            length1: length,
            length2: length,
            start2: START_BYTE,
            control: control.clone(),
            address: address.clone(),
            data: data.to_vec(),
            checksum: Self::compute_checksum(control, address, data),
            end: END_BYTE,
        }
    }

    /// Compute the checksum of a long frame
    fn compute_checksum(control: Control, address: Address, data: &[u8]) -> u8 {
        u8::from(control)
            .wrapping_add(address.into())
            .wrapping_add(data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)))
    }
}

impl Encodable for LongFrame {
    type Error = LongFrameDecodeError;

    /// Convert the long frame to a byte vector.
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.start1);
        bytes.push(self.length1);
        bytes.push(self.length2);
        bytes.push(self.start2);
        bytes.push(self.control.into());
        bytes.push(self.address.into());
        bytes.extend_from_slice(&self.data);
        bytes.push(self.checksum);
        bytes.push(self.end);

        bytes
    }

    /// Try decoding a byte slice into a short frame.
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        // Ensure that the size of the frame isn't too short, or too long
        if bytes.len() < MIN_LENGTH || bytes.len() > MAX_LENGTH {
            return Err(LongFrameDecodeError::InvalidSize(bytes.len()));
        }

        // Ensure that the size of the frame isn't too long
        if bytes.len() > 253 {
            return Err(LongFrameDecodeError::InvalidSize(bytes.len()));
        }

        // Ensure that the start byte is correct
        if bytes[START_1_INDEX] != START_BYTE {
            return Err(LongFrameDecodeError::InvalidStartByte(bytes[START_1_INDEX]));
        }

        // Ensure that the start bytes match
        if bytes[START_1_INDEX] != bytes[START_2_INDEX] {
            return Err(LongFrameDecodeError::StartByteMismatch(
                bytes[START_1_INDEX],
                bytes[START_2_INDEX],
            ));
        }

        // Ensure that the length fields match
        if bytes[LENGTH_1_INDEX] != bytes[LENGTH_2_INDEX] {
            return Err(LongFrameDecodeError::LengthMismatch(
                bytes[LENGTH_1_INDEX],
                bytes[LENGTH_2_INDEX],
            ));
        }

        // Ensure that the length field is correct
        let declared_length = bytes[LENGTH_1_INDEX];
        let expected_frame_size = 4 + declared_length + 2;
        if bytes.len() != expected_frame_size as usize {
            return Err(LongFrameDecodeError::InvalidLength(
                declared_length,
                (bytes.len() - 6) as u8,
            ));
        }

        // Ensure that the checksum is correct
        let checksum = Self::compute_checksum(
            bytes[CONTROL_INDEX].try_into()?,
            bytes[ADDRESS_INDEX].into(),
            &bytes[DATA_START_INDEX..DATA_START_INDEX + (bytes[LENGTH_1_INDEX] as usize - 2)],
        );

        let checksum_byte_index = bytes.len() - 2;
        if checksum != bytes[checksum_byte_index] {
            return Err(LongFrameDecodeError::InvalidChecksum(
                checksum,
                bytes[checksum_byte_index],
            ));
        }

        // Ensure that the end byte is correct
        let stop_byte_index = bytes.len() - 1;
        if bytes[stop_byte_index] != END_BYTE {
            return Err(LongFrameDecodeError::InvalidEndByte(bytes[stop_byte_index]));
        }

        Ok(Self {
            start1: bytes[START_1_INDEX],
            length1: bytes[LENGTH_1_INDEX],
            length2: bytes[LENGTH_2_INDEX],
            start2: bytes[START_2_INDEX],
            control: bytes[CONTROL_INDEX].try_into()?,
            address: bytes[ADDRESS_INDEX].into(),
            data: bytes[DATA_START_INDEX..DATA_START_INDEX + (bytes[LENGTH_1_INDEX] as usize - 2)]
                .to_vec(),
            checksum: bytes[checksum_byte_index],
            end: bytes[stop_byte_index],
        })
    }
}

impl FrameWithControl for LongFrame {
    fn with_frame_count_bit(&self, fcb: bool) -> Self {
        let mut new = self.clone();
        new.control = self.control.with_frame_count_bit(fcb);
        new
    }
}

/// Errors that can occur when decoding an M-Bus long frame
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LongFrameDecodeError {
    #[error("invalid frame size for long frame, expected >=8, got {0}")]
    InvalidSize(usize),
    #[error("invalid length for long frame, expected {0}, got {1}")]
    InvalidLength(u8, u8),
    #[error("invalid start byte for long frame, expected 0x10, got {0:#04x}")]
    InvalidStartByte(u8),
    #[error("mismatched start bytes for long frame, expected 0x68, got {0:#04x} and {1:#04x}")]
    StartByteMismatch(u8, u8),
    #[error("mismatched length fields for long frame, expected {0}, got {1}")]
    LengthMismatch(u8, u8),
    #[error("invalid checksum for long frame, expected {0:#04x}, got {1:#04x}")]
    InvalidChecksum(u8, u8),
    #[error("invalid end byte for long frame, expected 0x16, got {0:#04x}")]
    InvalidEndByte(u8),
    #[error("failed to decode control field: {0}")]
    ControlDecodeError(#[from] crate::control::ControlDecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_the_frame_to_a_byte_vector() {
        let frame = LongFrame::new(
            Control::Send,
            Address::Primary(0x01),
            &[0x00, 0x01, 0x02, 0x03],
        );
        let bytes = frame.to_bytes();
        assert_eq!(
            bytes,
            vec![
                0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16
            ]
        );
    }

    #[test]
    fn it_decodes_a_byte_vector_to_a_frame() {
        let bytes = vec![
            0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let frame = LongFrame::try_from_bytes(&bytes).unwrap();
        assert_eq!(frame.start1, 0x68);
        assert_eq!(frame.length1, 0x06);
        assert_eq!(frame.length2, 0x06);
        assert_eq!(frame.start2, 0x68);
        matches!(frame.control, Control::Send);
        matches!(frame.address, Address::Primary(0x01));
        assert_eq!(frame.data, vec![0x00, 0x01, 0x02, 0x03]);
        assert_eq!(frame.checksum, 0x5A);
        assert_eq!(frame.end, 0x16);
    }

    #[test]
    fn it_fails_to_decode_a_frame_shorter_than_8_bytes() {
        let bytes = vec![0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, LongFrameDecodeError::InvalidSize(7)));
    }

    #[test]
    fn it_fails_to_decode_a_frame_longer_than_261_bytes() {
        let mut bytes = vec![0x68, 0xFF, 0xFF, 0x68, 0x53, 0x01];
        bytes.extend(vec![0x00; 255]);
        bytes.push(0x00);
        bytes.push(0x16);
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, LongFrameDecodeError::InvalidSize(263)));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_invalid_start_byte() {
        let bytes = vec![
            0x69, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, LongFrameDecodeError::InvalidStartByte(0x69)));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_mismatched_start_bytes() {
        let bytes = vec![
            0x68, 0x06, 0x06, 0x69, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            LongFrameDecodeError::StartByteMismatch(0x68, 0x69)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_mismatched_length_fields() {
        let bytes = vec![
            0x68, 0x06, 0x07, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            LongFrameDecodeError::LengthMismatch(0x06, 0x07)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_invalid_length() {
        let bytes = vec![
            0x68, 0x07, 0x07, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            LongFrameDecodeError::InvalidLength(0x07, 0x06)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_invalid_checksum() {
        let bytes = vec![
            0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5B, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            LongFrameDecodeError::InvalidChecksum(0x5A, 0x5B)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_invalid_end_byte() {
        let bytes = vec![
            0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x15,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, LongFrameDecodeError::InvalidEndByte(0x15)));
    }

    #[test]
    fn it_fails_to_decode_a_frame_with_invalid_control_byte() {
        let bytes = vec![
            0x68, 0x06, 0x06, 0x68, 0x54, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16,
        ];
        let err = LongFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, LongFrameDecodeError::ControlDecodeError(_)));
    }
}
