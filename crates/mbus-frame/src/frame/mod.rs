mod long;
mod short;
mod single;

use thiserror::Error;
pub use long::LongFrame;
pub use short::ShortFrame;
pub use single::SingleCharacterFrame;

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

pub enum FrameType {
    Short,
    Long,
    Single,
}

impl Frame {
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
