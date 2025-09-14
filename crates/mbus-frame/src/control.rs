use thiserror::Error;

/// M-Bus Control Field
#[derive(Debug, Copy, Clone)]
pub enum Control {
    /// Initialize or Reset the slave device (SND-NKE)
    ///
    /// This command flows from the master to the slave and is used to
    /// initialize or reset a slave device.
    Initialize,

    /// Send user data to a slave device (SND-UD)
    ///
    /// This command flows from the master to the slave and is used to
    /// send user data to a slave device.
    Send {
        /// Frame count bit (FCB)
        ///
        /// Indicates if the frame count bit is set.
        fcb: bool,
    },

    /// Request time-critical user data from a slave device (REQ-UD1)
    ///
    /// This command flows from the master to the slave and is used to
    /// request time-critical user data from a slave device.
    PriorityRequest {
        /// Frame count bit (FCB)
        ///
        /// Indicates if the frame count bit is set.
        fcb: bool,
    },

    /// Request non-time-critical user data from a slave device (REQ-UD2)
    ///
    /// This command flows from the master to the slave and is used to
    /// request non-time-critical user data from a slave device.
    Request {
        /// Frame count bit (FCB)
        ///
        /// Indicates if the frame count bit is set.
        fcb: bool,
    },

    /// Respond with user data (RSP-UD)
    ///
    /// This command flows from the slave to the master and is used to
    /// respond with user data to a RequestData or RequestTimeCriticalData
    /// command.
    Response {
        /// Frame count bit (FCB)
        ///
        /// Indicates if the frame count bit is set.
        fcb: bool,

        /// Frame count valid bit (FCV)
        ///
        /// Indicates if the frame count bit should be considered.
        fcv: bool,
    },
}

#[derive(Debug, Error)]
pub enum ControlDecodeError {
    #[error("Unsupported communication type.")]
    UnsupportedCommunicationType,
}

/// Implement conversion from u8 to Control
impl TryFrom<u8> for Control {
    type Error = ControlDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x40 => Control::Initialize,
            0x53 | 0x73 => Control::Send { fcb: value == 0x73 },
            0x5A | 0x7A => Control::PriorityRequest { fcb: value == 0x7A },
            0x5B | 0x7B => Control::Request { fcb: value == 0x7B },
            0x08 | 0x18 | 0x28 | 0x38 => Control::Response {
                fcb: (value & 0x20) != 0,
                fcv: (value & 0x10) != 0,
            },
            _ => return Err(ControlDecodeError::UnsupportedCommunicationType),
        })
    }
}

impl From<Control> for u8 {
    fn from(control: Control) -> Self {
        match control {
            Control::Initialize => 0x40,
            Control::Send { fcb } => if fcb { 0x73 } else { 0x53 },
            Control::PriorityRequest { fcb } => if fcb { 0x7A } else { 0x5A },
            Control::Request { fcb } => if fcb { 0x7B } else { 0x5B },
            Control::Response { fcb, fcv } => {
                let mut value = 0x08;
                if fcv {
                    value |= 0x10;
                }
                if fcb {
                    value |= 0x20;
                }
                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes_an_initialize_control() {
        let control: Control = 0x40.try_into().unwrap();
        assert!(matches!(control, Control::Initialize));
    }

    #[test]
    fn it_decodes_a_send_control_with_fcb() {
        let control: Control = 0x73.try_into().unwrap();
        assert!(matches!(control, Control::Send { fcb: true }));
    }

    #[test]
    fn it_decodes_a_send_control_without_fcb() {
        let control: Control = 0x53.try_into().unwrap();
        assert!(matches!(control, Control::Send { fcb: false }));
    }

    #[test]
    fn it_decodes_a_priority_request_control_with_fcb() {
        let control: Control = 0x7A.try_into().unwrap();
        assert!(matches!(control, Control::PriorityRequest { fcb: true }));
    }

    #[test]
    fn it_decodes_a_priority_request_control_without_fcb() {
        let control: Control = 0x5A.try_into().unwrap();
        assert!(matches!(control, Control::PriorityRequest { fcb: false }));
    }

    #[test]
    fn it_decodes_a_request_control_with_fcb() {
        let control: Control = 0x7B.try_into().unwrap();
        assert!(matches!(control, Control::Request { fcb: true }));
    }

    #[test]
    fn it_decodes_a_request_control_without_fcb() {
        let control: Control = 0x5B.try_into().unwrap();
        assert!(matches!(control, Control::Request { fcb: false }));
    }

    #[test]
    fn it_decodes_a_response_control_with_fcb_and_fcv() {
        let control: Control = 0x38.try_into().unwrap();
        assert!(matches!(
            control,
            Control::Response {
                fcb: true,
                fcv: true
            }
        ));
    }

    #[test]
    fn it_decodes_a_response_control_with_fcb_without_fcv() {
        let control: Control = 0x28.try_into().unwrap();
        assert!(matches!(
            control,
            Control::Response {
                fcb: true,
                fcv: false
            }
        ));
    }

    #[test]
    fn it_decodes_a_response_control_without_fcb_with_fcv() {
        let control: Control = 0x18.try_into().unwrap();
        assert!(matches!(
            control,
            Control::Response {
                fcb: false,
                fcv: true
            }
        ));
    }

    #[test]
    fn it_decodes_a_response_control_without_fcb_and_fcv() {
        let control: Control = 0x08.try_into().unwrap();
        assert!(matches!(
            control,
            Control::Response {
                fcb: false,
                fcv: false
            }
        ));
    }

    #[test]
    fn it_fails_to_decode_an_unsupported_control() {
        let result: Result<Control, ControlDecodeError> = 0x99.try_into();
        assert!(matches!(result, Err(ControlDecodeError::UnsupportedCommunicationType)));
    }

    #[test]
    fn it_encodes_an_initialize_control() {
        let control = Control::Initialize;
        let value: u8 = control.into();
        assert_eq!(value, 0x40);
    }

    #[test]
    fn it_encodes_a_send_control_with_fcb() {
        let control = Control::Send { fcb: true };
        let value: u8 = control.into();
        assert_eq!(value, 0x73);
    }

    #[test]
    fn it_encodes_a_send_control_without_fcb() {
        let control = Control::Send { fcb: false };
        let value: u8 = control.into();
        assert_eq!(value, 0x53);
    }

    #[test]
    fn it_encodes_a_priority_request_control_with_fcb() {
        let control = Control::PriorityRequest { fcb: true };
        let value: u8 = control.into();
        assert_eq!(value, 0x7A);
    }

    #[test]
    fn it_encodes_a_priority_request_control_without_fcb() {
        let control = Control::PriorityRequest { fcb: false };
        let value: u8 = control.into();
        assert_eq!(value, 0x5A);
    }

    #[test]
    fn it_encodes_a_request_control_with_fcb() {
        let control = Control::Request { fcb: true };
        let value: u8 = control.into();
        assert_eq!(value, 0x7B);
    }

    #[test]
    fn it_encodes_a_request_control_without_fcb() {
        let control = Control::Request { fcb: false };
        let value: u8 = control.into();
        assert_eq!(value, 0x5B);
    }

    #[test]
    fn it_encodes_a_response_control_with_fcb_and_fcv() {
        let control = Control::Response { fcb: true, fcv: true };
        let value: u8 = control.into();
        assert_eq!(value, 0x38);
    }

    #[test]
    fn it_encodes_a_response_control_with_fcb_without_fcv() {
        let control = Control::Response { fcb: true, fcv: false };
        let value: u8 = control.into();
        assert_eq!(value, 0x28);
    }

    #[test]
    fn it_encodes_a_response_control_without_fcb_with_fcv() {
        let control = Control::Response { fcb: false, fcv: true };
        let value: u8 = control.into();
        assert_eq!(value, 0x18);
    }

    #[test]
    fn it_encodes_a_response_control_without_fcb_and_fcv() {
        let control = Control::Response { fcb: false, fcv: false };
        let value: u8 = control.into();
        assert_eq!(value, 0x08);
    }
}
