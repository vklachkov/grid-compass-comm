use std::{io, slice};

use thiserror::Error;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, LE, TryFromBytes, TryReadError, U16, Unaligned,
};

const DLE: u8 = 0x10;
const STX: u8 = 0x02;
const ETX: u8 = 0x03;

pub const EOM_FLAG_ON: u8 = 1;

pub const PDL_VERSION: [u8; 2] = [b'0', b'2'];

#[derive(Clone, Debug)]
pub struct Frame {
    header: FrameHeader,
    body: FrameBody,
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, TryFromBytes, IntoBytes)]
#[repr(packed)]
pub struct FrameHeader {
    pub ty: FrameType,
    pub flags: u8,
    pub window_size: u8,
    pub seq_number: u8,
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, TryFromBytes, IntoBytes)]
#[repr(u8)]
pub enum FrameType {
    Rfc = 1,
    Ack = 2,
    Disc = 3,
    Ping = 4,
    Data = 5,
}

#[derive(Clone, Debug)]
pub enum FrameBody {
    Rfc(RfcFrameBody),
    Ack(ShortFrameBody),
    Disc(ShortFrameBody),
    Ping(ShortFrameBody),
    Data(DataFrameBody),
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct RfcFrameBody {
    pub pdl_connection_id: u8,
    pub pdl_version: [u8; 2],
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct ShortFrameBody {
    pub pdl_connection_id: u8,
}

#[derive(Clone, Copy, Debug, Immutable, KnownLayout, TryFromBytes, IntoBytes)]
#[repr(u16)]
pub enum VipcProtocolFunctionCode {
    Msg = 0,
    Connect = 1,
    ConnectResponse = 2,
    Disconnect = 3,
    DisconnectResponse = 4,
    SignOn = 19,
    SignOnResponse = 6,
    SignOff = 7,
    Error = 100,
}

#[derive(Clone, Debug)]
pub enum DataFrameBody {
    SignOn {
        properties: Vec<DataProperty>,
    },
    SignOnResponse {
        sign_on_status: u16,
        server_name: &'static str,
    },
}

#[derive(Clone, Debug)]
pub struct DataProperty {
    pub ty: u8,
    pub length: u8,
    pub value: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum FrameReadError {
    #[error("malformed frame marker {marker:#04x}")]
    MalformedFrameMarker { marker: u8 },

    #[error("invalid frame CRC: expected {expected:#06x}, found {found:#06x}")]
    InvalidCrc { expected: u16, found: u16 },

    #[error("validation error: {reason}")]
    Validation { reason: String },

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl Frame {
    pub fn rfc(conn_id: u8, seq_number: u8, flags: u8) -> Self {
        Frame {
            header: FrameHeader {
                ty: FrameType::Rfc,
                flags: flags,
                window_size: EOM_FLAG_ON,
                seq_number: seq_number,
            },
            body: FrameBody::Rfc(RfcFrameBody {
                pdl_connection_id: conn_id,
                pdl_version: PDL_VERSION,
            }),
        }
    }

    pub fn ack(conn_id: u8, seq_number: u8, flags: u8) -> Self {
        Self {
            header: FrameHeader {
                ty: FrameType::Ack,
                flags: flags,
                window_size: b'A',
                seq_number: seq_number,
            },
            body: FrameBody::Ack(ShortFrameBody {
                pdl_connection_id: conn_id,
            }),
        }
    }

    pub fn data(seq_number: u8, flags: u8, body: DataFrameBody) -> Self {
        Self {
            header: FrameHeader {
                ty: FrameType::Data,
                flags: flags,
                window_size: b'D',
                seq_number: seq_number,
            },
            body: FrameBody::Data(body),
        }
    }

    pub fn header(&self) -> &FrameHeader {
        &self.header
    }

    pub fn body(&self) -> &FrameBody {
        &self.body
    }

    pub fn read_from_io(mut src: impl io::Read) -> Result<Self, FrameReadError> {
        let raw = Self::read_raw_frame(&mut src)?;
        Self::deserialize(raw)
    }

    pub fn read_raw_frame(mut src: impl io::Read) -> Result<Vec<u8>, FrameReadError> {
        // Skip all bytes until start sequence.
        let mut pair = [0u8; 2];
        while pair != [DLE, STX] {
            pair[0] = pair[1];
            src.read_exact(&mut pair[1..=1])?;
        }

        // Read all bytes and unstuff them until end sequence.
        let mut buffer = Vec::with_capacity(64);
        loop {
            let mut byte = 0u8;
            src.read_exact(slice::from_mut(&mut byte))?;

            if byte != DLE {
                buffer.push(byte);
                continue;
            }

            src.read_exact(slice::from_mut(&mut byte))?;
            match byte {
                DLE => buffer.push(DLE),
                STX => buffer.clear(),
                ETX => break,
                marker => {
                    return Err(FrameReadError::MalformedFrameMarker { marker });
                }
            }
        }

        let expected_crc = Self::crc16_arc(&buffer);

        // Check frame checksum after.
        let mut crc = [0u8; 2];
        src.read_exact(&mut crc)?;

        let crc = u16::from_le_bytes(crc);
        if crc != expected_crc {
            return Err(FrameReadError::InvalidCrc {
                expected: expected_crc,
                found: crc,
            });
        }

        Ok(buffer)
    }

    pub fn deserialize(buffer: Vec<u8>) -> Result<Self, FrameReadError> {
        let mut r = io::Cursor::new(buffer);

        let header = Self::read_entity::<FrameHeader>(&mut r)?;

        let body = match header.ty {
            FrameType::Rfc => Self::read_entity(&mut r).map(FrameBody::Rfc),
            FrameType::Ack => Self::read_entity(&mut r).map(FrameBody::Ack),
            FrameType::Disc => Self::read_entity(&mut r).map(FrameBody::Disc),
            FrameType::Ping => Self::read_entity(&mut r).map(FrameBody::Ping),
            FrameType::Data => Self::read_data(&mut r).map(FrameBody::Data),
        }?;

        // TODO: check cursor for unused bytes.

        Ok(Frame { header, body })
    }

    fn crc16_arc(data: &[u8]) -> u16 {
        let mut crc = 0;

        for byte in data {
            crc ^= *byte as u16;

            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc
    }

    fn read_entity<S>(mut src: impl io::Read) -> Result<S, FrameReadError>
    where
        S: KnownLayout + TryFromBytes,
    {
        let mut data = vec![0u8; size_of::<S>()];
        src.read_exact(&mut data)?;

        match S::try_read_from_bytes(&data) {
            Ok(s) => Ok(s),
            Err(TryReadError::Size(_)) => unreachable!(),
            Err(TryReadError::Validity(err)) => Err(FrameReadError::Validation {
                reason: err.to_string(),
            }),
        }
    }

    fn read_data(mut src: impl io::Read) -> Result<DataFrameBody, FrameReadError> {
        // FIXME: support BE machines.
        let code = Self::read_entity::<VipcProtocolFunctionCode>(&mut src)?;

        match code {
            VipcProtocolFunctionCode::SignOn => Ok(DataFrameBody::SignOn {
                properties: Self::read_properties(&mut src)?,
            }),
            _ => unimplemented!("function {code:?}"),
        }
    }

    fn read_properties(mut src: impl io::Read) -> Result<Vec<DataProperty>, FrameReadError> {
        let mut properties = Vec::<DataProperty>::with_capacity(6);

        loop {
            let mut ty = 0;

            match src.read_exact(slice::from_mut(&mut ty)) {
                Ok(_) => {}
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err.into()),
            }

            let mut length = 0;
            src.read_exact(slice::from_mut(&mut length))?;

            let mut value = vec![0; length as usize];
            src.read_exact(&mut value)?;

            properties.push(DataProperty { ty, length, value });
        }

        Ok(properties)
    }

    pub fn write_to_io(self, mut dst: impl io::Write) -> io::Result<()> {
        let frame_data = self.serialize();
        let crc = Self::crc16_arc(&frame_data);

        let count_of_dle = frame_data.iter().filter(|&&b| b == DLE).count();
        let frame_data = if count_of_dle == 0 {
            frame_data
        } else {
            let mut stuffed_frame_data = Vec::with_capacity(frame_data.len() + count_of_dle);
            for b in frame_data.into_iter() {
                stuffed_frame_data.push(b);
                if b == DLE {
                    stuffed_frame_data.push(DLE);
                }
            }
            stuffed_frame_data
        };

        dst.write_all(&[DLE, STX])?;
        dst.write_all(&frame_data)?;
        dst.write_all(&[DLE, ETX])?;
        dst.write_all(&crc.to_le_bytes())?;

        Ok(())
    }

    fn serialize(self) -> Vec<u8> {
        let mut response = Vec::with_capacity(128);

        response.extend(self.header.as_bytes());
        match self.body {
            FrameBody::Rfc(b) => response.extend(b.as_bytes()),
            FrameBody::Ack(b) => response.extend(b.as_bytes()),
            FrameBody::Disc(b) => response.extend(b.as_bytes()),
            FrameBody::Ping(b) => response.extend(b.as_bytes()),
            FrameBody::Data(b) => match b {
                DataFrameBody::SignOn { .. } => unimplemented!(),
                DataFrameBody::SignOnResponse {
                    sign_on_status,
                    server_name,
                } => {
                    response
                        .extend((VipcProtocolFunctionCode::SignOnResponse as u16).to_le_bytes());
                    response.extend(sign_on_status.to_le_bytes());
                    response.push(server_name.len() as u8);
                    response.extend(server_name.as_bytes());
                }
            },
        }

        response
    }
}
