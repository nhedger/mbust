# mbus-frame

**mbus-frame** is a pure Rust implementation of an M-Bus datagram encoder/decoder.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
mbus-frame = "0.1"
```

## Usage

The following examples demonstrate how to create and parse different types of 
M-Bus frames.

### Creating a Single Character frame

```rust
use mbus_frame::SingleCharacterFrame;

pub fn main() {
    /// Convert frame to bytes
    let bytes = SingleCharacterFrame::Ack.to_bytes();

    /// Create frame from bytes
    let frame = SingleCharacterFrame::from_bytes(&[0xE5]).unwrap();
}
```

### Creating a Short frame

```rust
use mbus_frame::ShortFrame;

pub fn main() {
    /// Convert frame to bytes
    let bytes = ShortFrame::new(
        Control::Initialize, 
        Address::Primary(0x01),
    ).to_bytes();

    /// Create frame from bytes
    let frame = ShortFrame::from_bytes(&[0x10, 0x40, 0x01, 0x41, 0x16]).unwrap();
}
```

### Creating a Long frame

```rust
use mbus_frame::LongFrame;

pub fn main() {
    /// Convert frame to bytes
    let bytes = LongFrame::new(
        Control::Send, 
        Address::Primary(0x01), 
        &[0x00, 0x01, 0x02, 0x03],
    ).to_bytes();

    /// Create frame from bytes
    let frame = LongFrame::from_bytes(&[
        0x68, 0x06, 0x06, 0x68,
        0x53, 0x01, 0x00, 0x01,
        0x02, 0x03, 0x5A, 0x16,
    ]).unwrap();
}
```

## References

The following references were used in the development of this library:

- EN 13757-2, _Communication systems for meters - Part 2: Wired M-Bus communication_
- EN 60870-5-1, _Telecontrol equipment and systems - Part 5: Transmission protocols - Section one: Transmission frame formats_
- EN 60870-5-2, _Telecontrol equipment and systems - Part 5: Transmission protocols - Section 2: Link transmission procedures_

## License

The **mbus-frame** library is open-source software licensed under the [MIT License](LICENSE).
