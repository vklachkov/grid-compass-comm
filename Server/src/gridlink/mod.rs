mod error;
mod raw_frame;

use std::{io, slice};

use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes, TryReadError, Unaligned,
};

pub use self::error::FrameError;
use self::raw_frame::RawFrame;

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
    Connect {
        header: VipcConnectHeader,
        path: String,
    },
    ConnectResponse {
        header: VipcConnectHeader,
        status: u16,
    },
    Disconnect {
        header: VipcConnectHeader,
        reason: u16,
    },
    DisconnectResponse {
        header: VipcConnectHeader,
    },
    SignOn {
        properties: Vec<DataProperty>,
    },
    SignOnResponse {
        status: u16,
        server_name: &'static str,
    },
    SignOff,
    Msg {
        header: VipcMessageHeader,
        body: VipcMessageBody,
    },
}

#[derive(Clone, Debug)]
pub struct DataProperty {
    pub ty: u8,
    pub length: u8,
    pub value: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct VipcConnectHeader {
    pub local_path_id: u16,
    pub remote_path_id: u16,
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct VipcMessageHeader {
    pub local_path_id: u16,
    pub remote_path_id: u16,
    pub class: u16,
    pub note: u16,
    pub data_length: u16,
}

#[derive(Clone, Debug)]
pub enum VipcMessageBody {
    VfsRequest(VfsRequest),
    VfsResponse(VfsResponse),
    Raw(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct VfsRequest {
    pub header: VfsRequestHeader,
    pub body: VfsRequestBody,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum VfsRequestCode {
    GetStatus = 1,
    Open = 2,
    Close = 3,
    Read = 4,
    Write = 5,
    Seek = 6,
    Attach = 8,
    Detach = 9,
    ReadDesc = 12,
    WriteDesc = 13,
    SetStatus = 20,
    ReadDirPage = 29,
    Unsupported = 0xFFFF,
}

impl VfsRequestCode {
    pub fn from_raw(raw: u16) -> Self {
        match raw {
            1 => Self::GetStatus,
            2 => Self::Open,
            3 => Self::Close,
            4 => Self::Read,
            5 => Self::Write,
            6 => Self::Seek,
            8 => Self::Attach,
            9 => Self::Detach,
            12 => Self::ReadDesc,
            13 => Self::WriteDesc,
            20 => Self::SetStatus,
            29 => Self::ReadDirPage,
            _ => Self::Unsupported,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

#[derive(Clone, Debug)]
pub enum VfsRequestBody {
    Attach(VfsAttachRequest),
    Open(VfsOpenRequest),
    Read(VfsReadRequest),
    Seek(VfsSeekRequest),
    Write(VfsWriteRequest),
    Simple,
    Raw(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct VfsAttachRequest {
    pub mode: u8,
    pub access: u8,
    pub password: [u8; 17],
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct VfsOpenRequest {
    pub num_buf: u8,
}

#[derive(Clone, Debug)]
pub struct VfsReadRequest {
    pub data_length: u16,
}

#[derive(Clone, Debug)]
pub struct VfsSeekRequest {
    pub mode: u8,
    pub position: u32,
}

#[derive(Clone, Debug)]
pub struct VfsWriteRequest {
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct VfsRequestHeader {
    pub request: u16,
    pub requestors_conn_id: u16,
    pub servers_conn_id: u16,
}

#[derive(Clone, Debug)]
pub enum VfsResponse {
    Simple(VfsSimpleResponse),
    Read(VfsReadResponse),
}

#[derive(Clone, Copy, Debug, Immutable, Unaligned, KnownLayout, FromBytes, IntoBytes)]
#[repr(packed)]
pub struct VfsSimpleResponse {
    pub response: u16,
    pub servers_conn_id: u16,
    pub requestors_conn_id: u16,
    pub error: u16,
}

#[derive(Clone, Debug)]
pub struct VfsReadResponse {
    pub common: VfsSimpleResponse,
    pub data_length: u16,
    pub data: Vec<u8>,
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

    pub fn read_from_io(src: impl io::Read) -> Result<Self, FrameError> {
        let raw = RawFrame::read_from_io(src)?;
        Self::deserialize(raw.data)
    }

    pub fn deserialize(buffer: Vec<u8>) -> Result<Self, FrameError> {
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

    fn read_entity<S>(mut src: impl io::Read) -> Result<S, FrameError>
    where
        S: KnownLayout + TryFromBytes,
    {
        let mut data = vec![0u8; size_of::<S>()];
        src.read_exact(&mut data)?;

        match S::try_read_from_bytes(&data) {
            Ok(s) => Ok(s),
            Err(TryReadError::Size(_)) => unreachable!(),
            Err(TryReadError::Validity(err)) => Err(FrameError::Validation {
                reason: err.to_string(),
            }),
        }
    }

    fn read_data(mut src: impl io::Read) -> Result<DataFrameBody, FrameError> {
        // FIXME: support BE machines.
        let code = Self::read_entity::<VipcProtocolFunctionCode>(&mut src)?;

        match code {
            VipcProtocolFunctionCode::Connect => Ok(DataFrameBody::Connect {
                header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
                path: Self::read_pascal_string(&mut src)?,
            }),
            VipcProtocolFunctionCode::Disconnect => Ok(DataFrameBody::Disconnect {
                header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
                reason: Self::read_entity::<u16>(&mut src)?,
            }),
            VipcProtocolFunctionCode::DisconnectResponse => Ok(DataFrameBody::DisconnectResponse {
                header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
            }),
            VipcProtocolFunctionCode::Msg => {
                let header = Self::read_entity::<VipcMessageHeader>(&mut src)?;
                let mut payload = vec![0; header.data_length as usize];
                src.read_exact(&mut payload)?;

                let body = if header.class == 83 {
                    let mut payload_reader = io::Cursor::new(payload);
                    // TODO: make protocol integer decoding explicit before supporting BE machines.
                    let request_header =
                        Self::read_entity::<VfsRequestHeader>(&mut payload_reader)?;
                    let mut request_payload = Vec::new();
                    io::Read::read_to_end(&mut payload_reader, &mut request_payload)?;

                    VipcMessageBody::VfsRequest(VfsRequest {
                        header: request_header,
                        body: Self::read_vfs_request_body(
                            VfsRequestCode::from_raw(request_header.request),
                            request_payload,
                        )?,
                    })
                } else {
                    VipcMessageBody::Raw(payload)
                };

                Ok(DataFrameBody::Msg { header, body })
            }
            VipcProtocolFunctionCode::SignOn => Ok(DataFrameBody::SignOn {
                properties: Self::read_signon_properties(&mut src)?,
            }),
            VipcProtocolFunctionCode::SignOff => Ok(DataFrameBody::SignOff),
            _ => unimplemented!("function {code:?}"),
        }
    }

    fn read_vfs_request_body(
        request: VfsRequestCode,
        payload: Vec<u8>,
    ) -> Result<VfsRequestBody, FrameError> {
        match request {
            VfsRequestCode::Attach => Self::read_vfs_attach_request(payload),
            VfsRequestCode::Open => Self::read_vfs_open_request(payload),
            VfsRequestCode::GetStatus
            | VfsRequestCode::Read
            | VfsRequestCode::ReadDesc
            | VfsRequestCode::ReadDirPage => Self::read_vfs_read_request(payload),
            VfsRequestCode::Seek => Self::read_vfs_seek_request(payload),

            VfsRequestCode::Write | VfsRequestCode::WriteDesc | VfsRequestCode::SetStatus => {
                Self::read_vfs_write_request(payload)
            }
            VfsRequestCode::Close | VfsRequestCode::Detach => Self::read_empty_vfs_request(payload),
            VfsRequestCode::Unsupported => Ok(VfsRequestBody::Raw(payload)),
        }
    }

    fn read_vfs_attach_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        const ATTACH_FIXED_PAYLOAD_SIZE: usize = 19;

        if payload.len() < ATTACH_FIXED_PAYLOAD_SIZE {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS attach payload: expected at least {ATTACH_FIXED_PAYLOAD_SIZE} bytes, found {}",
                    payload.len()
                ),
            });
        }

        let mode = payload[0];
        let access = payload[1];
        let mut password = [0u8; 17];
        password.copy_from_slice(&payload[2..ATTACH_FIXED_PAYLOAD_SIZE]);

        let mut path_reader = io::Cursor::new(&payload[ATTACH_FIXED_PAYLOAD_SIZE..]);
        let path = Self::read_pascal_string(&mut path_reader)?;

        Ok(VfsRequestBody::Attach(VfsAttachRequest {
            mode,
            access,
            password,
            path,
        }))
    }

    fn read_vfs_open_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        let Some((&num_buf, extra)) = payload.split_first() else {
            return Err(FrameError::Validation {
                reason: "invalid VFS open payload: expected num_buf byte".to_owned(),
            });
        };

        if !extra.is_empty() {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS open payload: expected 1 byte, found {}",
                    payload.len()
                ),
            });
        }

        Ok(VfsRequestBody::Open(VfsOpenRequest { num_buf }))
    }

    fn read_vfs_read_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        if payload.len() != 2 {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS read payload: expected 2 bytes, found {}",
                    payload.len()
                ),
            });
        }

        Ok(VfsRequestBody::Read(VfsReadRequest {
            data_length: u16::from_le_bytes([payload[0], payload[1]]),
        }))
    }

    fn read_vfs_seek_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        if payload.len() != 5 {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS seek payload: expected 5 bytes, found {}",
                    payload.len()
                ),
            });
        }

        Ok(VfsRequestBody::Seek(VfsSeekRequest {
            mode: payload[0],
            position: u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]),
        }))
    }

    fn read_vfs_write_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        if payload.len() < 2 {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS write payload: expected at least 2 bytes, found {}",
                    payload.len()
                ),
            });
        }

        let data_length = u16::from_le_bytes([payload[0], payload[1]]) as usize;
        let data = payload[2..].to_vec();
        if data.len() != data_length {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS write payload: declared {data_length} bytes, found {}",
                    data.len()
                ),
            });
        }

        Ok(VfsRequestBody::Write(VfsWriteRequest { data }))
    }

    fn read_empty_vfs_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
        if !payload.is_empty() {
            return Err(FrameError::Validation {
                reason: format!(
                    "invalid VFS simple payload: expected 0 bytes, found {}",
                    payload.len()
                ),
            });
        }

        Ok(VfsRequestBody::Simple)
    }

    fn read_signon_properties(mut src: impl io::Read) -> Result<Vec<DataProperty>, FrameError> {
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

    fn read_pascal_string(mut src: impl io::Read) -> Result<String, FrameError> {
        let mut length = 0;
        src.read_exact(slice::from_mut(&mut length))?;

        let mut raw = vec![0; length as usize];
        src.read_exact(&mut raw)?;

        String::from_utf8(raw).map_err(|err| FrameError::Validation {
            reason: format!("invalid string {:02x?}", err.as_bytes()),
        })
    }

    pub fn write_to_io(self, dst: impl io::Write) -> io::Result<usize> {
        RawFrame::new(self.serialize())
            .write_to_io(dst)
            .map_err(|err| match err {
                FrameError::Io(err) => err,
                err => io::Error::new(io::ErrorKind::InvalidData, err),
            })
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
                DataFrameBody::Connect { .. } => unimplemented!(),
                DataFrameBody::ConnectResponse { header, status } => {
                    response
                        .extend((VipcProtocolFunctionCode::ConnectResponse as u16).to_le_bytes());
                    response.extend(header.as_bytes());
                    response.extend(status.to_le_bytes());
                }
                DataFrameBody::Disconnect { .. } => unimplemented!(),
                DataFrameBody::DisconnectResponse { header } => {
                    response.extend(
                        (VipcProtocolFunctionCode::DisconnectResponse as u16).to_le_bytes(),
                    );
                    response.extend(header.as_bytes());
                }
                DataFrameBody::SignOn { .. } => unimplemented!(),
                DataFrameBody::SignOnResponse {
                    status,
                    server_name,
                } => {
                    response
                        .extend((VipcProtocolFunctionCode::SignOnResponse as u16).to_le_bytes());
                    response.extend(status.to_le_bytes());
                    response.push(server_name.len() as u8);
                    response.extend(server_name.as_bytes());
                }
                DataFrameBody::SignOff => {
                    response.extend((VipcProtocolFunctionCode::SignOff as u16).to_le_bytes());
                }
                DataFrameBody::Msg { header, body } => {
                    response.extend((VipcProtocolFunctionCode::Msg as u16).to_le_bytes());

                    let body = Self::serialize_vipc_message_body(body);
                    let header = VipcMessageHeader {
                        data_length: body.len() as u16,
                        ..header
                    };

                    response.extend(header.as_bytes());
                    response.extend(body);
                }
            },
        }

        response
    }

    fn serialize_vipc_message_body(body: VipcMessageBody) -> Vec<u8> {
        let mut response = Vec::with_capacity(64);

        match body {
            VipcMessageBody::VfsRequest(_) => unimplemented!(),
            VipcMessageBody::VfsResponse(vfs_response) => match vfs_response {
                VfsResponse::Simple(simple) => response.extend(simple.as_bytes()),
                VfsResponse::Read(read) => {
                    response.extend(read.common.as_bytes());
                    response.extend(read.data_length.to_le_bytes());
                    response.extend(read.data);
                }
            },
            VipcMessageBody::Raw(raw) => response.extend(raw),
        }

        response
    }
}
