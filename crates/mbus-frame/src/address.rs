/// M-Bus Address
#[derive(Debug, Copy, Clone)]
pub enum Address {
    /// The address for unconfigured devices (0)
    Unconfigured,

    /// The primary address for devices (1-250)
    Primary(u8),

    /// The address for link layer management (251)
    Management,

    /// Reserved address (252)
    Reserved,

    /// The address for secondary addressing (253)
    Secondary,

    /// The address for tests and diagnosis (254)
    Diagnosis,

    /// The address to broadcast messages to all slaves (255)
    Broadcast,
}

/// Implement conversion from u8 to Address
impl From<u8> for Address {
    fn from(value: u8) -> Self {
        match value {
            0 => Address::Unconfigured,
            1..=250 => Address::Primary(value),
            251 => Address::Management,
            252 => Address::Reserved,
            253 => Address::Secondary,
            254 => Address::Diagnosis,
            255 => Address::Broadcast,
        }
    }
}

/// Implement conversion from Address to u8
impl From<Address> for u8 {
    fn from(address: Address) -> Self {
        match address {
            Address::Unconfigured => 0,
            Address::Primary(value) => value,
            Address::Management => 251,
            Address::Reserved => 252,
            Address::Secondary => 253,
            Address::Diagnosis => 254,
            Address::Broadcast => 255,
        }
    }
}