use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("validation error: {reason}")]
    Validation {
        reason: String,
    },

    #[error("frame is too large: max {max} bytes")]
    FrameTooLarge {
        max: usize,
    },

    #[error("unexpected end of frame")]
    UnexpectedEof,

    #[error("malformed frame marker {marker:#04x}")]
    MalformedFrameMarker {
        marker: u8,
    },

    #[error("invalid frame CRC: expected {expected:#06x}, found {found:#06x}")]
    InvalidCrc {
        expected: u16,
        found: u16,
    },

    #[error(transparent)]
    Io(io::Error),
}

impl From<io::Error> for FrameError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            FrameError::UnexpectedEof
        } else {
            FrameError::Io(err)
        }
    }
}
