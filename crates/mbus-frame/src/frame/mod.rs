mod short;
mod long;
mod single;

pub use short::ShortFrame;
pub use long::LongFrame;
pub use single::SingleCharacterFrame;

/// Trait for M-Bus frames
pub trait Frame: Sized {
    /// Error type for frame parsing
    type Error;

    /// Convert the frame to a byte vector
    fn to_bytes(&self) -> Vec<u8>;

    /// Create a frame from a byte slice
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
}

