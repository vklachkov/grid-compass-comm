#![allow(unused)]

mod data_frame;
mod error;
mod frame;
mod raw_frame;
mod utils;

pub mod vipc;

pub use data_frame::{ConnectHeader, DataFrameRequest, DataFrameResponse, SignOnProperty};
pub use error::FrameError;
pub use frame::{EOM_FLAG_ON, Frame, FrameBody, RfcFrameBody, ShortFrameBody};
pub use raw_frame::RawFrame;

// impl Frame {
//     fn read_data(mut src: impl io::Read) -> Result<DataFrameBody, FrameError> {
//         // FIXME: support BE machines.
//         let code = Self::read_entity::<DataFrameType>(&mut src)?;

//         match code {
//             DataFrameType::Connect => Ok(DataFrameBody::Connect {
//                 header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
//                 path: Self::read_pascal_string(&mut src)?,
//             }),
//             DataFrameType::Disconnect => Ok(DataFrameBody::Disconnect {
//                 header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
//                 reason: Self::read_entity::<u16>(&mut src)?,
//             }),
//             DataFrameType::DisconnectResponse => Ok(DataFrameBody::DisconnectResponse {
//                 header: Self::read_entity::<VipcConnectHeader>(&mut src)?,
//             }),
//             DataFrameType::Msg => {
//                 let header = Self::read_entity::<VipcMessageHeader>(&mut src)?;
//                 let mut payload = vec![0; header.data_length as usize];
//                 src.read_exact(&mut payload)?;

//                 let body = if header.class == 83 {
//                     let mut payload_reader = io::Cursor::new(payload);
//                     // TODO: make protocol integer decoding explicit before supporting BE machines.
//                     let request_header =
//                         Self::read_entity::<VfsRequestHeader>(&mut payload_reader)?;
//                     let mut request_payload = Vec::new();
//                     io::Read::read_to_end(&mut payload_reader, &mut request_payload)?;

//                     VipcMessageBody::VfsRequest(VfsRequest {
//                         header: request_header,
//                         body: Self::read_vfs_request_body(
//                             VfsRequestCode::from_raw(request_header.request),
//                             request_payload,
//                         )?,
//                     })
//                 } else {
//                     VipcMessageBody::Raw(payload)
//                 };

//                 Ok(DataFrameBody::Msg { header, body })
//             }
//             DataFrameType::SignOn => Ok(DataFrameBody::SignOn {
//                 properties: Self::read_signon_properties(&mut src)?,
//             }),
//             DataFrameType::SignOff => Ok(DataFrameBody::SignOff),
//             _ => unimplemented!("function {code:?}"),
//         }
//     }

//     fn read_vfs_request_body(
//         request: VfsRequestCode,
//         payload: Vec<u8>,
//     ) -> Result<VfsRequestBody, FrameError> {
//         match request {
//             VfsRequestCode::Attach => Self::read_vfs_attach_request(payload),
//             VfsRequestCode::Open => Self::read_vfs_open_request(payload),
//             VfsRequestCode::GetStatus
//             | VfsRequestCode::Read
//             | VfsRequestCode::ReadDesc
//             | VfsRequestCode::ReadDirPage => Self::read_vfs_read_request(payload),
//             VfsRequestCode::Seek => Self::read_vfs_seek_request(payload),

//             VfsRequestCode::Write | VfsRequestCode::WriteDesc | VfsRequestCode::SetStatus => {
//                 Self::read_vfs_write_request(payload)
//             }
//             VfsRequestCode::Close | VfsRequestCode::Detach => Self::read_empty_vfs_request(payload),
//             VfsRequestCode::Unsupported => Ok(VfsRequestBody::Raw(payload)),
//         }
//     }

//     fn read_vfs_attach_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         const ATTACH_FIXED_PAYLOAD_SIZE: usize = 19;

//         if payload.len() < ATTACH_FIXED_PAYLOAD_SIZE {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS attach payload: expected at least {ATTACH_FIXED_PAYLOAD_SIZE} bytes, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         let mode = payload[0];
//         let access = payload[1];
//         let mut password = [0u8; 17];
//         password.copy_from_slice(&payload[2..ATTACH_FIXED_PAYLOAD_SIZE]);

//         let mut path_reader = io::Cursor::new(&payload[ATTACH_FIXED_PAYLOAD_SIZE..]);
//         let path = Self::read_pascal_string(&mut path_reader)?;

//         Ok(VfsRequestBody::Attach(VfsAttachRequest {
//             mode,
//             access,
//             password,
//             path,
//         }))
//     }

//     fn read_vfs_open_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         let Some((&num_buf, extra)) = payload.split_first() else {
//             return Err(FrameError::Validation {
//                 reason: "invalid VFS open payload: expected num_buf byte".to_owned(),
//             });
//         };

//         if !extra.is_empty() {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS open payload: expected 1 byte, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         Ok(VfsRequestBody::Open(VfsOpenRequest { num_buf }))
//     }

//     fn read_vfs_read_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         if payload.len() != 2 {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS read payload: expected 2 bytes, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         Ok(VfsRequestBody::Read(VfsReadRequest {
//             data_length: u16::from_le_bytes([payload[0], payload[1]]),
//         }))
//     }

//     fn read_vfs_seek_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         if payload.len() != 5 {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS seek payload: expected 5 bytes, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         Ok(VfsRequestBody::Seek(VfsSeekRequest {
//             mode: payload[0],
//             position: u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]),
//         }))
//     }

//     fn read_vfs_write_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         if payload.len() < 2 {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS write payload: expected at least 2 bytes, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         let data_length = u16::from_le_bytes([payload[0], payload[1]]) as usize;
//         let data = payload[2..].to_vec();
//         if data.len() != data_length {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS write payload: declared {data_length} bytes, found {}",
//                     data.len()
//                 ),
//             });
//         }

//         Ok(VfsRequestBody::Write(VfsWriteRequest { data }))
//     }

//     fn read_empty_vfs_request(payload: Vec<u8>) -> Result<VfsRequestBody, FrameError> {
//         if !payload.is_empty() {
//             return Err(FrameError::Validation {
//                 reason: format!(
//                     "invalid VFS simple payload: expected 0 bytes, found {}",
//                     payload.len()
//                 ),
//             });
//         }

//         Ok(VfsRequestBody::Simple)
//     }

//     fn read_signon_properties(mut src: impl io::Read) -> Result<Vec<SignOnProperty>, FrameError> {
//         let mut properties = Vec::<SignOnProperty>::with_capacity(6);

//         loop {
//             let mut ty = 0;

//             match src.read_exact(slice::from_mut(&mut ty)) {
//                 Ok(_) => {}
//                 Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
//                 Err(err) => return Err(err.into()),
//             }

//             let mut length = 0;
//             src.read_exact(slice::from_mut(&mut length))?;

//             let mut value = vec![0; length as usize];
//             src.read_exact(&mut value)?;

//             properties.push(SignOnProperty { ty, length, value });
//         }

//         Ok(properties)
//     }

//     fn read_pascal_string(mut src: impl io::Read) -> Result<String, FrameError> {
//         let mut length = 0;
//         src.read_exact(slice::from_mut(&mut length))?;

//         let mut raw = vec![0; length as usize];
//         src.read_exact(&mut raw)?;

//         String::from_utf8(raw).map_err(|err| FrameError::Validation {
//             reason: format!("invalid string {:02x?}", err.as_bytes()),
//         })
//     }

//     pub fn write_to_io(self, dst: impl io::Write) -> io::Result<usize> {
//         let raw = self
//             .serialize()
//             .and_then(|data| RawFrame::new(data).write_to_io(dst));
//         raw.map_err(|err| match err {
//             FrameError::Io(err) => err,
//             err => io::Error::new(io::ErrorKind::InvalidData, err),
//         })
//     }

//     fn serialize(self) -> Result<Vec<u8>, FrameError> {
//         let mut response = Vec::with_capacity(128);

//         response.extend(self.header.to_bytes()?);
//         match self.body {
//             FrameBody::Rfc(b) => response.extend(b.as_bytes()),
//             FrameBody::Ack(b) => response.extend(b.as_bytes()),
//             FrameBody::Disc(b) => response.extend(b.as_bytes()),
//             FrameBody::Ping(b) => response.extend(b.as_bytes()),
//             FrameBody::Data(b) => match b {
//                 DataFrameBody::Connect { .. } => unimplemented!(),
//                 DataFrameBody::ConnectResponse { header, status } => {
//                     response.extend((DataFrameType::ConnectResponse as u16).to_le_bytes());
//                     response.extend(header.as_bytes());
//                     response.extend(status.to_le_bytes());
//                 }
//                 DataFrameBody::Disconnect { .. } => unimplemented!(),
//                 DataFrameBody::DisconnectResponse { header } => {
//                     response.extend((DataFrameType::DisconnectResponse as u16).to_le_bytes());
//                     response.extend(header.as_bytes());
//                 }
//                 DataFrameBody::SignOn { .. } => unimplemented!(),
//                 DataFrameBody::SignOnResponse {
//                     status,
//                     server_name,
//                 } => {
//                     response.extend((DataFrameType::SignOnResponse as u16).to_le_bytes());
//                     response.extend(status.to_le_bytes());
//                     response.push(server_name.len() as u8);
//                     response.extend(server_name.as_bytes());
//                 }
//                 DataFrameBody::SignOff => {
//                     response.extend((DataFrameType::SignOff as u16).to_le_bytes());
//                 }
//                 DataFrameBody::Msg { header, body } => {
//                     response.extend((DataFrameType::Msg as u16).to_le_bytes());

//                     let body = Self::serialize_vipc_message_body(body);
//                     let header = VipcMessageHeader {
//                         data_length: body.len() as u16,
//                         ..header
//                     };

//                     response.extend(header.as_bytes());
//                     response.extend(body);
//                 }
//             },
//         }

//         Ok(response)
//     }

//     fn serialize_vipc_message_body(body: VipcMessageBody) -> Vec<u8> {
//         let mut response = Vec::with_capacity(64);

//         match body {
//             VipcMessageBody::VfsRequest(_) => unimplemented!(),
//             VipcMessageBody::VfsResponse(vfs_response) => match vfs_response {
//                 VfsResponse::Simple(simple) => response.extend(simple.as_bytes()),
//                 VfsResponse::Read(read) => {
//                     response.extend(read.common.as_bytes());
//                     response.extend(read.data_length.to_le_bytes());
//                     response.extend(read.data);
//                 }
//             },
//             VipcMessageBody::Raw(raw) => response.extend(raw),
//         }

//         response
//     }
// }
