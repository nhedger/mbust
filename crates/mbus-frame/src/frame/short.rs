use super::{Encodable, FrameWithControl};
use crate::address::Address;
use thiserror::Error;
use crate::control::Control;

/// M-Bus Short Frame
///
/// An M-Bus short frame is fixed-length frame used for simple communication
/// that doesn't transmit user data.
///
/// The format of a short frame is defined in EN 60870-5-2 (§3.2) as a frame
/// with fixed length in the FT 1.2 format.
#[derive(Debug)]
#[derive(Clone)]
pub struct ShortFrame {
    /// Start byte (0x10)
    ///
    /// The start byte marks the beginning of the short frame.
    start: u8,

    /// Control
    ///
    /// The control byte is used to indicate the type of frame.
    control: Control,

    /// Address
    ///
    /// The address byte specifies the address of the addressed slave.
    address: Address,

    /// Checksum
    ///
    /// The checksum byte is used for error detection.
    ///
    /// The checksum is calculated as the sum of all bytes in the frame,
    /// excluding the start and end bytes, modulo 256.
    checksum: u8,

    /// End byte (0x16)
    ///
    /// The end byte marks the end of the short frame.
    end: u8,
}

/// Start byte of an M-Bus short frame
const START_BYTE: u8 = 0x10;

/// End byte of an M-Bus short frame
const END_BYTE: u8 = 0x16;

/// Length of an M-Bus short frame
const LENGTH: usize = 5;

const START_INDEX: usize = 0;
const CONTROL_INDEX: usize = 1;
const ADDRESS_INDEX: usize = 2;
const CHECKSUM_INDEX: usize = 3;
const END_INDEX: usize = 4;

/// Implementation of the M-Bus short frame
impl ShortFrame {
    /// Create a new M-Bus short frame
    pub fn new(control: Control, address: Address) -> Self {
        Self {
            start: START_BYTE,
            control: control.clone(),
            address: address.clone(),
            checksum: Self::compute_checksum(control, address),
            end: END_BYTE,
        }
    }

    /// Compute the checksum of a long frame
    fn compute_checksum(control: Control, address: Address) -> u8 {
        u8::from(control)
            .wrapping_add(address.into())
    }
}

impl Encodable for ShortFrame {
    type Error = ShortFrameDecodeError;

    /// Convert the short frame to a byte vector.
    fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.start,
            self.control.into(),
            self.address.into(),
            self.checksum,
            self.end,
        ]
    }

    /// Try decoding a byte slice into a short frame.
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        // Ensure that the length is correct
        if bytes.len() != LENGTH {
            return Err(ShortFrameDecodeError::InvalidLength(bytes.len()));
        }

        // Ensure that the start byte is correct
        if bytes[START_INDEX] != START_BYTE {
            return Err(ShortFrameDecodeError::InvalidStartByte(bytes[START_INDEX]));
        }

        // Ensure that the checksum is correct
        let checksum = bytes[CONTROL_INDEX].wrapping_add(bytes[ADDRESS_INDEX]);
        if checksum != bytes[CHECKSUM_INDEX] {
            return Err(ShortFrameDecodeError::InvalidChecksum(
                checksum,
                bytes[CHECKSUM_INDEX],
            ));
        }

        // Ensure that the end byte is correct
        if bytes[END_INDEX] != END_BYTE {
            return Err(ShortFrameDecodeError::InvalidEndByte(bytes[END_INDEX]));
        }

        Ok(Self {
            start: bytes[START_INDEX],
            control: bytes[CONTROL_INDEX].try_into()?,
            address: bytes[ADDRESS_INDEX].into(),
            checksum: bytes[CHECKSUM_INDEX],
            end: bytes[END_INDEX],
        })
    }
}

impl FrameWithControl for ShortFrame {
    fn with_frame_count_bit(&self, fcb: bool) -> Self {
        let mut new = self.clone();
        new.control = self.control.with_frame_count_bit(fcb);
        new
    }
}


/// Errors that can occur when decoding an M-Bus short frame
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ShortFrameDecodeError {
    #[error("invalid length for short frame, expected 5, got {0}")]
    InvalidLength(usize),
    #[error("invalid start byte for short frame, expected 0x10, got {0:#04x}")]
    InvalidStartByte(u8),
    #[error("invalid checksum for short frame, expected {0:#04x}, got {1:#04x}")]
    InvalidChecksum(u8, u8),
    #[error("invalid end byte for short frame, expected 0x16, got {0:#04x}")]
    InvalidEndByte(u8),
    #[error("failed to decode control field: {0}")]
    ControlDecodeError(#[from] crate::control::ControlDecodeError),

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_the_frame_to_a_byte_vector() {
        let frame = ShortFrame::new(Control::Initialize, Address::Primary(0x01));
        let bytes = frame.to_bytes();
        assert_eq!(bytes, vec![0x10, 0x40, 0x01, 0x41, 0x16]);
    }

    #[test]
    fn it_decodes_a_byte_vector_into_a_frame() {
        let bytes = vec![0x10, 0x40, 0x01, 0x41, 0x16];
        let frame = ShortFrame::try_from_bytes(&bytes).unwrap();
        assert_eq!(frame.start, 0x10);
        matches!(frame.control, Control::Initialize);
        matches!(frame.address, Address::Primary(0x01));
        assert_eq!(frame.checksum, 0x41);
        assert_eq!(frame.end, 0x16);
    }

    #[test]
    fn it_fails_to_decode_a_byte_vector_shorter_than_5_bytes() {
        let bytes = vec![0x10, 0x40, 0x01, 0x41];
        let err = ShortFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ShortFrameDecodeError::InvalidLength(4)));
    }

    #[test]
    fn it_fails_to_decode_a_byte_vector_longer_than_5_bytes() {
        let bytes = vec![0x10, 0x40, 0x01, 0x41, 0x16, 0x00];
        let err = ShortFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ShortFrameDecodeError::InvalidLength(6)));
    }

    #[test]
    fn it_fails_to_decode_a_byte_vector_with_invalid_start_byte() {
        let bytes = vec![0x11, 0x40, 0x01, 0x41, 0x16];
        let err = ShortFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ShortFrameDecodeError::InvalidStartByte(0x11)));
    }

    #[test]
    fn it_fails_to_decode_a_byte_vector_with_invalid_checksum() {
        let bytes = vec![0x10, 0x40, 0x01, 0x42, 0x16];
        let err = ShortFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            ShortFrameDecodeError::InvalidChecksum(0x41, 0x42)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_byte_vector_with_invalid_end_byte() {
        let bytes = vec![0x10, 0x40, 0x01, 0x41, 0x15];
        let err = ShortFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ShortFrameDecodeError::InvalidEndByte(0x15)));
    }
}
