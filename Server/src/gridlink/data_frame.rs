use std::io::{self, Write};

use bstr::BStr;

use super::{
    error::FrameError,
    utils::{CursorExt, ReadExt, WriteExt},
};

#[derive(Clone, Copy, Debug, strum::FromRepr)]
#[repr(u16)]
enum DataFrameType {
    Msg = 0,                // VIPC_Px_Msg
    Connect = 1,            // VIPC_Px_Connect
    ConnectResponse = 2,    // VIPC_Px_ConnectResponse
    Disconnect = 3,         // VIPC_Px_Disconnect
    DisconnectResponse = 4, // VIPC_Px_DisconnectResponse
    SignOn = 19,            // VIPC_Px_Signon
    SignOnResponse = 6,     // VIPC_Px_SignonResponse
    SignOff = 7,            // VIPC_Px_Signoff
    Error = 100,            // VIPC_Px_Error
}

#[derive(Clone, Debug)]
pub enum DataFrameRequest<'a> {
    // VipcConnectHeaderType
    Connect {
        header: ConnectHeader, // VipcCommonPart
        path: &'a BStr,
    },

    // VipcDiscReqType
    Disconnect {
        header: ConnectHeader, // VipcCommonPart
        reason: u16,           // ReasonForDisconnect
    },

    // VipcSignonType
    SignOn {
        properties: Vec<SignOnProperty<'a>>, // PropertyList
    },

    // VipcSignoffType
    SignOff {
        // empty
    },

    // VipcMsgType
    Msg {
        header: ConnectHeader,
        payload: &'a [u8],
    },
}

#[derive(Clone, Copy, Debug)]
pub struct ConnectHeader {
    pub local_path_id: u16,  // localPathID
    pub remote_path_id: u16, // remotePathID
}

#[derive(Clone, Copy, Debug)]
pub struct SignOnProperty<'a> {
    pub ty: u8,          // propertyType
    pub value: &'a [u8], // len, value
}

#[derive(Clone, Debug)]
pub enum DataFrameResponse<'a> {
    // VipcConnectResponseType
    Connect {
        header: ConnectHeader, // VipcCommonPart
        status: u16,           // ConnectStatus
    },

    // VipcDiscRespType
    Disconnect {
        header: ConnectHeader, // VipcCommonPart
    },

    // VipcSignonResponseType
    SignOn {
        status: u16,           // signOnStatus
        server_name: &'a BStr, // serverNameStr
    },

    Msg {
        header: ConnectHeader,
        payload: &'a [u8],
    },
}

impl<'a> DataFrameRequest<'a> {
    pub fn try_from_slice(data: &'a [u8]) -> Result<Self, FrameError> {
        let mut cursor = io::Cursor::new(data);

        let ty = cursor.read_u16()?;
        let Some(ty) = DataFrameType::from_repr(ty) else {
            return Err(FrameError::Validation {
                reason: format!("unknown data frame type {ty}"),
            });
        };

        let body = match ty {
            DataFrameType::Connect => Self::Connect {
                header: Self::read_connect_header(&mut cursor)?,
                path: Self::read_small_slice(&mut cursor).map(BStr::new)?,
            },
            DataFrameType::Disconnect => Self::Disconnect {
                header: Self::read_connect_header(&mut cursor)?,
                reason: cursor.read_u16()?,
            },
            DataFrameType::SignOn => Self::SignOn {
                properties: Self::read_signon_properties(&mut cursor)?,
            },
            DataFrameType::SignOff => Self::SignOff {},
            DataFrameType::Msg => Self::Msg {
                header: Self::read_connect_header(&mut cursor)?,
                payload: cursor.read_remainder(),
            },
            _ => {
                return Err(FrameError::Validation {
                    reason: format!("unsupported data frame type {ty:?}"),
                });
            }
        };

        Ok(body)
    }

    fn read_connect_header(cursor: &mut io::Cursor<&[u8]>) -> Result<ConnectHeader, FrameError> {
        Ok(ConnectHeader {
            local_path_id: cursor.read_u16()?,
            remote_path_id: cursor.read_u16()?,
        })
    }

    fn read_signon_properties(
        cursor: &mut io::Cursor<&'a [u8]>,
    ) -> Result<Vec<SignOnProperty<'a>>, FrameError> {
        let mut properties = Vec::with_capacity(6);

        loop {
            let ty = match cursor.read_u8() {
                Ok(ty) => ty,
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err.into()),
            };

            let value = Self::read_small_slice(cursor)?;

            properties.push(SignOnProperty { ty, value });
        }

        Ok(properties)
    }

    fn read_small_slice(cursor: &mut io::Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
        let length = cursor.read_u8()?;
        cursor.read_slice(length as usize).map_err(Into::into)
    }
}

impl DataFrameResponse<'_> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = io::Cursor::new(Vec::with_capacity(6));

        match self {
            Self::Connect { header, status } => {
                _ = data.write_u16(DataFrameType::ConnectResponse as u16);
                Self::write_connect_header(&mut data, header);
                _ = data.write_u16(*status);
            }
            Self::Disconnect { header } => {
                data.write_u16(DataFrameType::DisconnectResponse as u16);
                Self::write_connect_header(&mut data, header);
            }
            Self::SignOn {
                status,
                server_name,
            } => {
                _ = data.write_u16(DataFrameType::SignOnResponse as u16);
                _ = data.write_u16(*status);
                Self::write_nslice(&mut data, server_name);
            }
            Self::Msg { header, payload } => {
                _ = data.write_u16(DataFrameType::Msg as u16);
                Self::write_connect_header(&mut data, header);
                _ = data.write_all(payload);
            }
        }

        data.into_inner()
    }

    fn write_connect_header(mut dst: impl WriteExt, header: &ConnectHeader) {
        _ = dst.write_u16(header.local_path_id);
        _ = dst.write_u16(header.remote_path_id);
    }

    fn write_nslice(mut dst: impl WriteExt, value: &[u8]) {
        _ = dst.write_u8(value.len() as u8);
        _ = dst.write_all(value);
    }
}
