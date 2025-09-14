use super::Encodable;
use thiserror::Error;

/// M-Bus Single Character Frame
///
/// An M-Bus single character frame is used for simple control commands
/// and acknowledgments. It consists of a single byte.
///
/// The format of a single character frame is defined in EN 60870-5-2 (§3.2)
/// as a single-character frame in the FT 1.2 format.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum SingleCharacterFrame {
    /// Positive Acknowledgment (ACK)
    Ack = 0xE5,

    /// Negative Acknowledgment (NACK)
    Nack = 0xA2,
}

impl Encodable for SingleCharacterFrame {
    type Error = SingleCharacterFrameDecodeError;

    /// Convert the single character frame to a byte vector
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    /// Create a single character frame from a byte slice
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 1 {
            return Err(SingleCharacterFrameDecodeError::InvalidSize(bytes.len()));
        }

        match bytes[0] {
            0xE5 => Ok(SingleCharacterFrame::Ack),
            0xA2 => Ok(SingleCharacterFrame::Nack),
            _ => Err(SingleCharacterFrameDecodeError::InvalidByte(bytes[0])),
        }
    }
}

/// Errors that can occur when decoding an M-Bus long frame
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum SingleCharacterFrameDecodeError {
    #[error("invalid frame size for single character frame, expected 1, got {0}")]
    InvalidSize(usize),
    #[error("invalid byte for single character frame, expected 0xE5 or 0xA2, got {0:#04x}")]
    InvalidByte(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_the_frame_to_a_byte_vector() {
        let frame = SingleCharacterFrame::Ack;
        let bytes = frame.to_bytes();
        assert_eq!(bytes, vec![0xE5]);
    }

    #[test]
    fn it_decodes_a_byte_slice_to_a_frame() {
        let bytes = vec![0xE5];
        let frame = SingleCharacterFrame::try_from_bytes(&bytes).unwrap();
        matches!(frame, SingleCharacterFrame::Ack);
    }

    #[test]
    fn it_fails_to_decode_a_byte_slice_with_invalid_size() {
        let bytes = vec![0xE5, 0x00];
        let err = SingleCharacterFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            SingleCharacterFrameDecodeError::InvalidSize(2)
        ));
    }

    #[test]
    fn it_fails_to_decode_a_byte_slice_with_invalid_byte() {
        let bytes = vec![0x00];
        let err = SingleCharacterFrame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            SingleCharacterFrameDecodeError::InvalidByte(0x00)
        ));
    }
}
