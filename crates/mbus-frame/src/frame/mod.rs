mod long;
mod short;
mod single;

use thiserror::Error;
pub use long::LongFrame;
pub use short::ShortFrame;
pub use single::SingleCharacterFrame;
use crate::address::Address;
use crate::control::Control;

/// Trait for M-Bus frames
pub trait Encodable: Sized {
    /// Error type for frame parsing
    type Error;

    /// Convert the frame to a byte vector
    fn to_bytes(&self) -> Vec<u8>;

    /// Create a frame from a byte slice
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub trait FrameWithControl: Sized {
    /// Set the frame count bit (FCB) of the frame's control field
    fn with_frame_count_bit(&self, fcb: bool) -> Self;
}

/// Generic M-Bus frame
#[derive(Debug, Clone)]
pub enum Frame {
    Short(ShortFrame),
    Long(LongFrame),
    Single(SingleCharacterFrame),
}

#[derive(Debug)]
pub enum FrameType {
    Short,
    Long,
    Single,
}

impl Frame {
    pub fn new_single(frame: SingleCharacterFrame) -> Self {
        Frame::Single(frame)
    }

    pub fn new_short(control: Control, address: Address) -> Self {
        Frame::Short(ShortFrame::new(control, address))
    }

    pub fn new_long(control: Control, address: Address, data: Vec<u8>) -> Self {
        Frame::Long(LongFrame::new(control, address, &data))
    }

    /// Parse an M-Bus frame from a byte slice
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, FrameError> {
        match Self::detect_type_from_bytes(bytes)? {
            FrameType::Short => Ok(Frame::Short(ShortFrame::try_from_bytes(bytes)?)),
            FrameType::Long => Ok(Frame::Long(LongFrame::try_from_bytes(bytes)?)),
            FrameType::Single => Ok(Frame::Single(SingleCharacterFrame::try_from_bytes(bytes)?)),
        }
    }

    /// Convert the frame to a byte vector
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Frame::Short(frame) => frame.to_bytes(),
            Frame::Long(frame) => frame.to_bytes(),
            Frame::Single(frame) => frame.to_bytes(),
        }
    }

    pub fn get_type(&self) -> FrameType {
        match self {
            Frame::Short(_) => FrameType::Short,
            Frame::Long(_) => FrameType::Long,
            Frame::Single(_) => FrameType::Single,
        }
    }

    fn detect_type_from_bytes(bytes: &[u8]) -> Result<FrameType, FrameDetectionError> {
        if bytes.is_empty() {
            return Err(FrameDetectionError::Empty);
        }

        match bytes[0] {
            0x10 => Ok(FrameType::Short),
            0x68 => Ok(FrameType::Long),
            0xE5 | 0xA2 => Ok(FrameType::Single),
            _ => Err(FrameDetectionError::UnknownFrameType(bytes[0])),
        }
    }
}

impl FrameWithControl for Frame {
    fn with_frame_count_bit(&self, fcb: bool) -> Self {
        match self {
            Frame::Short(frame) => Frame::Short(frame.with_frame_count_bit(fcb)),
            Frame::Long(frame) => Frame::Long(frame.with_frame_count_bit(fcb)),
            Frame::Single(_) => self.clone(),
        }
    }
}

#[derive(Error, Debug)]
pub enum FrameDetectionError {
    #[error("input byte slice is empty")]
    Empty,
    #[error("unknown frame type {0:#04x}")]
    UnknownFrameType(u8),
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("frame detection failed: {0}")]
    Detection(#[from] FrameDetectionError),
    #[error("short frame parsing failed: {0}")]
    ShortFrame(#[from] short::ShortFrameDecodeError),
    #[error("long frame parsing failed: {0}")]
    LongFrame(#[from] long::LongFrameDecodeError),
    #[error("single character frame parsing failed: {0}")]
    SingleCharacterFrame(#[from] single::SingleCharacterFrameDecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_detects_a_short_frame() {
        let bytes = vec![0x10, 0x7B, 0x00, 0x7B, 0x10];
        let frame_type = Frame::detect_type_from_bytes(&bytes).unwrap();
        matches!(frame_type, FrameType::Short);
    }

    #[test]
    fn it_detects_a_long_frame() {
        let bytes = vec![0x68, 0x06, 0x06, 0x68, 0x53, 0x01, 0x00, 0x01, 0x02, 0x03, 0x5A, 0x16];
        let frame_type = Frame::detect_type_from_bytes(&bytes).unwrap();
        matches!(frame_type, FrameType::Long);
    }

    #[test]
    fn it_detects_a_single_character_frame() {
        let bytes = vec![0xE5];
        let frame_type = Frame::detect_type_from_bytes(&bytes).unwrap();
        matches!(frame_type, FrameType::Single);
    }

    #[test]
    fn it_fails_to_detect_an_empty_byte_slice() {
        let bytes = vec![];
        let err = Frame::detect_type_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, FrameDetectionError::Empty));
    }

    #[test]
    fn it_fails_to_detect_an_unknown_frame_type() {
        let bytes = vec![0x00];
        let err = Frame::detect_type_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, FrameDetectionError::UnknownFrameType(0x00)));
    }

    #[test]
    fn it_fails_to_parse_an_invalid_short_frame() {
        let bytes = vec![0x10, 0x00, 0x00, 0x00];
        let err = Frame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, FrameError::ShortFrame(_)));
    }

    #[test]
    fn it_fails_to_parse_an_invalid_long_frame() {
        let bytes = vec![0x68, 0x00, 0x00, 0x68];
        let err = Frame::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, FrameError::LongFrame(_)));
    }

    #[test]
    fn it_creates_a_new_short_frame() {
        let frame = Frame::new_short(Control::Request, Address::Primary(1));
        assert!(matches!(frame, Frame::Short(_)));
    }

    #[test]
    fn it_creates_a_new_long_frame() {
        let frame = Frame::new_long(Control::Request, Address::Primary(1), vec![0x01, 0x02, 0x03]);
        assert!(matches!(frame, Frame::Long(_)));
    }

    #[test]
    fn it_creates_a_new_single_character_frame() {
        let frame = Frame::new_single(SingleCharacterFrame::Ack);
        assert!(matches!(frame, Frame::Single(_)));
    }
}