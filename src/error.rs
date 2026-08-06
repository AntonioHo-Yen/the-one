#[derive(Debug)]
pub enum ProtocolError {
    Unauthorized,
    UnauthorizedAccess,
    InvalidOriginKey,
    InvalidSignature,
    StalePayload,
    DuressTriggered,
    Custom(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::Unauthorized => write!(f, "Unauthorized operation"),
            ProtocolError::UnauthorizedAccess => write!(f, "Unauthorized access attempt"),
            ProtocolError::InvalidOriginKey => write!(f, "Invalid origin key"),
            ProtocolError::InvalidSignature => write!(f, "Invalid cryptographic signature"),
            ProtocolError::StalePayload => write!(f, "Stale payload timestamp"),
            ProtocolError::DuressTriggered => write!(f, "Duress state triggered"),
            ProtocolError::Custom(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}