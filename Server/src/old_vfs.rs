// use crate::gridlink::{
//     DataFrameBody, EOM_FLAG_ON, Frame, FrameBody, FrameError, FrameType, VfsReadResponse,
//     VfsRequest, VfsRequestBody, VfsRequestCode, VfsResponse, VfsSimpleResponse, VipcConnectHeader,
//     VipcMessageBody, VipcMessageHeader,
// };

// const VFS_CLASS: u16 = 83;
// const GENERAL_BROADCAST_CLASS: u16 = 0x7000;
// const VFS_RESPONSE_FLAG: u16 = 0x8000;
// const VFS_ERROR_OK: u16 = 0;
// const VFS_ERROR_UNSUPPORTED: u16 = 1;
// const SHORT_DIRECTORY_ACCESS: u8 = 5;
// const SET_DIRECTION_ENTRY_ID: u8 = 0xfd;
// const SET_WILDCARD_ENTRY_ID: u8 = 0xfe;
// const WILDCARD_BYTE: u8 = 0xf7;
// const SERVER_NAME: &str = "vklachkov server";
// const FS_RESOURCE_NAME: &str = "Hard Disk";
// const NAME_DEVICE_RESOURCE_ENTRIES: &[&str] = &["Hard Disk~fs~", "Demo Device~fs~"];
// const ROOT_DIRECTORY_ENTRIES: &[&str] = &["Programs~Subject~", "Server Subjects~Subject~"];
// const PROGRAMS_DIRECTORY_ENTRIES: &[&str] = &["Message Of The Day~Text~"];

// #[derive(Debug)]
// #[allow(dead_code)]
// struct VfsAttachment {
//     server_conn_id: u16,
//     requestors_conn_id: u16,
//     vipc_local_path_id: u16,
//     vipc_remote_path_id: u16,
//     mode: u8,
//     access: u8,
//     password: [u8; 17],
//     path: String,
//     open: bool,
//     num_buf: Option<u8>,
//     read_offset: usize,
//     directory_offset: usize,
//     directory_wildcard: Option<Vec<u8>>,
// }
//
// fn handle_vfs_request(
//     addr: SocketAddr,
//     header: &VipcMessageHeader,
//     request: &VfsRequest,
//     last_vfs_conn_id: &mut u16,
//     vfs_attachments: &mut HashMap<u16, VfsAttachment>,
// ) -> Option<DataFrameBody> {
//     let error = match &request.body {
//         VfsRequestBody::Attach(attach) => {
//             *last_vfs_conn_id = last_vfs_conn_id.wrapping_add(1);
//             if *last_vfs_conn_id == 0 {
//                 *last_vfs_conn_id = 1;
//             }

//             vfs_attachments.insert(
//                 *last_vfs_conn_id,
//                 VfsAttachment {
//                     server_conn_id: *last_vfs_conn_id,
//                     requestors_conn_id: request.header.requestors_conn_id,
//                     vipc_local_path_id: header.remote_path_id,
//                     vipc_remote_path_id: header.local_path_id,
//                     mode: attach.mode,
//                     access: attach.access,
//                     password: attach.password,
//                     path: attach.path.clone(),
//                     open: false,
//                     num_buf: None,
//                     read_offset: 0,
//                     directory_offset: 0,
//                     directory_wildcard: None,
//                 },
//             );

//             println!(
//                 "worker({addr}): attached VFS conn {} to {:?}",
//                 *last_vfs_conn_id, attach.path
//             );

//             VFS_ERROR_OK
//         }
//         VfsRequestBody::Open(open) => {
//             let servers_conn_id = request.header.servers_conn_id;
//             match vfs_attachments.get_mut(&servers_conn_id) {
//                 Some(attachment) => {
//                     attachment.open = true;
//                     attachment.num_buf = Some(open.num_buf);

//                     println!(
//                         "worker({addr}): opened VFS conn {} path {:?}, num_buf {}",
//                         servers_conn_id, attachment.path, open.num_buf
//                     );

//                     VFS_ERROR_OK
//                 }
//                 None => {
//                     println!(
//                         "worker({addr}): open for unknown VFS conn {}",
//                         servers_conn_id
//                     );
//                     VFS_ERROR_UNSUPPORTED
//                 }
//             }
//         }
//         VfsRequestBody::Read(read)
//             if request.header.request == VfsRequestCode::GetStatus.as_u16() =>
//         {
//             let servers_conn_id = request.header.servers_conn_id;
//             let data = vec![0; read.data_length as usize];
//             println!(
//                 "worker({addr}): get status for VFS conn {}, requested {} bytes",
//                 servers_conn_id, read.data_length
//             );

//             return Some(vfs_read_response(
//                 header,
//                 request.header.request,
//                 servers_conn_id,
//                 request.header.requestors_conn_id,
//                 VFS_ERROR_OK,
//                 data,
//             ));
//         }
//         VfsRequestBody::Read(read) if request.header.request == VfsRequestCode::Read.as_u16() => {
//             let servers_conn_id = request.header.servers_conn_id;
//             let Some(attachment) = vfs_attachments.get_mut(&servers_conn_id) else {
//                 println!(
//                     "worker({addr}): read for unknown VFS conn {}",
//                     servers_conn_id
//                 );

//                 return Some(vfs_read_response(
//                     header,
//                     request.header.request,
//                     servers_conn_id,
//                     request.header.requestors_conn_id,
//                     VFS_ERROR_UNSUPPORTED,
//                     Vec::new(),
//                 ));
//             };

//             let data = read_virtual_file(attachment, read.data_length as usize);
//             println!(
//                 "worker({addr}): read VFS conn {} path {:?}, requested {} bytes, sent {} bytes",
//                 servers_conn_id,
//                 attachment.path,
//                 read.data_length,
//                 data.len()
//             );

//             return Some(vfs_read_response(
//                 header,
//                 request.header.request,
//                 servers_conn_id,
//                 request.header.requestors_conn_id,
//                 VFS_ERROR_OK,
//                 data,
//             ));
//         }
//         VfsRequestBody::Read(read)
//             if request.header.request == VfsRequestCode::ReadDesc.as_u16() =>
//         {
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): read descriptor for VFS conn {}, requested {} bytes",
//                 servers_conn_id, read.data_length
//             );

//             return Some(vfs_read_response(
//                 header,
//                 request.header.request,
//                 servers_conn_id,
//                 request.header.requestors_conn_id,
//                 VFS_ERROR_OK,
//                 vec![0; read.data_length as usize],
//             ));
//         }
//         VfsRequestBody::Read(read)
//             if request.header.request == VfsRequestCode::ReadDirPage.as_u16() =>
//         {
//             let servers_conn_id = request.header.servers_conn_id;
//             let (data, object_count) = match vfs_attachments.get_mut(&servers_conn_id) {
//                 Some(attachment) => read_virtual_directory_page(attachment, read.data_length),
//                 None => (Vec::new(), 0),
//             };

//             println!(
//                 "worker({addr}): read directory page for VFS conn {}, requested {} objects, sent {} objects, entries {:?}",
//                 servers_conn_id,
//                 read.data_length,
//                 object_count,
//                 decode_short_directory_entry_names(&data)
//             );

//             return Some(vfs_read_response(
//                 header,
//                 request.header.request,
//                 servers_conn_id,
//                 request.header.requestors_conn_id,
//                 VFS_ERROR_OK,
//                 data,
//             ));
//         }
//         VfsRequestBody::Seek(seek) if request.header.request == VfsRequestCode::Seek.as_u16() => {
//             let servers_conn_id = request.header.servers_conn_id;
//             match vfs_attachments.get_mut(&servers_conn_id) {
//                 Some(attachment) => {
//                     apply_seek(attachment, seek.mode, seek.position);

//                     println!(
//                         "worker({addr}): seek VFS conn {} path {:?}, mode {}, position {}, offset {}",
//                         servers_conn_id,
//                         attachment.path,
//                         seek.mode,
//                         seek.position,
//                         attachment.read_offset
//                     );

//                     VFS_ERROR_OK
//                 }
//                 None => {
//                     println!(
//                         "worker({addr}): seek for unknown VFS conn {}, mode {}, position {}",
//                         servers_conn_id, seek.mode, seek.position
//                     );

//                     VFS_ERROR_UNSUPPORTED
//                 }
//             }
//         }
//         VfsRequestBody::Write(write)
//             if request.header.request == VfsRequestCode::SetStatus.as_u16() =>
//         {
//             let servers_conn_id = request.header.servers_conn_id;
//             match vfs_attachments.get_mut(&servers_conn_id) {
//                 Some(attachment) => {
//                     apply_set_status(attachment, &write.data);
//                     println!(
//                         "worker({addr}): set status for VFS conn {}, {} bytes, wildcard {:?}",
//                         servers_conn_id,
//                         write.data.len(),
//                         attachment
//                             .directory_wildcard
//                             .as_ref()
//                             .map(|wildcard| String::from_utf8_lossy(wildcard))
//                     );

//                     VFS_ERROR_OK
//                 }
//                 None => {
//                     println!(
//                         "worker({addr}): set status for unknown VFS conn {}, {} bytes",
//                         servers_conn_id,
//                         write.data.len()
//                     );

//                     VFS_ERROR_UNSUPPORTED
//                 }
//             }
//         }
//         VfsRequestBody::Simple if request.header.request == VfsRequestCode::Close.as_u16() => {
//             let servers_conn_id = request.header.servers_conn_id;
//             if let Some(attachment) = vfs_attachments.get_mut(&servers_conn_id) {
//                 attachment.open = false;
//             }
//             println!("worker({addr}): closed VFS conn {}", servers_conn_id);

//             VFS_ERROR_OK
//         }
//         VfsRequestBody::Simple if request.header.request == VfsRequestCode::Detach.as_u16() => {
//             let servers_conn_id = request.header.servers_conn_id;
//             let detached = vfs_attachments.remove(&servers_conn_id);
//             println!(
//                 "worker({addr}): detached VFS conn {}, existed={}",
//                 servers_conn_id,
//                 detached.is_some()
//             );

//             VFS_ERROR_OK
//         }
//         VfsRequestBody::Raw(payload) => {
//             let request_code = request.header.request;
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): unsupported VFS request {}, servers_conn_id {}, {} payload bytes",
//                 request_code,
//                 servers_conn_id,
//                 payload.len()
//             );
//             VFS_ERROR_UNSUPPORTED
//         }
//         VfsRequestBody::Read(read) => {
//             let request_code = request.header.request;
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): unsupported VFS read request {}, servers_conn_id {}, requested {} bytes",
//                 request_code, servers_conn_id, read.data_length
//             );
//             VFS_ERROR_UNSUPPORTED
//         }
//         VfsRequestBody::Seek(seek) => {
//             let request_code = request.header.request;
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): unsupported VFS seek request {}, servers_conn_id {}, mode {}, position {}",
//                 request_code, servers_conn_id, seek.mode, seek.position
//             );
//             VFS_ERROR_UNSUPPORTED
//         }
//         VfsRequestBody::Write(write) => {
//             let request_code = request.header.request;
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): unsupported VFS write request {}, servers_conn_id {}, {} bytes",
//                 request_code,
//                 servers_conn_id,
//                 write.data.len()
//             );
//             VFS_ERROR_UNSUPPORTED
//         }
//         VfsRequestBody::Simple => {
//             let request_code = request.header.request;
//             let servers_conn_id = request.header.servers_conn_id;
//             println!(
//                 "worker({addr}): unsupported VFS simple request {}, servers_conn_id {}",
//                 request_code, servers_conn_id
//             );
//             VFS_ERROR_UNSUPPORTED
//         }
//     };

//     Some(vfs_response(
//         header,
//         request.header.request,
//         if matches!(&request.body, VfsRequestBody::Attach(_)) && error == VFS_ERROR_OK {
//             *last_vfs_conn_id
//         } else {
//             request.header.servers_conn_id
//         },
//         request.header.requestors_conn_id,
//         error,
//     ))
// }

// fn read_virtual_file(attachment: &mut VfsAttachment, max_len: usize) -> Vec<u8> {
//     let content = virtual_file_content(&attachment.path);
//     let remaining = &content[attachment.read_offset.min(content.len())..];
//     let read_len = max_len.min(remaining.len());
//     let data = remaining[..read_len].to_vec();
//     attachment.read_offset += read_len;
//     data
// }

// fn apply_seek(attachment: &mut VfsAttachment, mode: u8, position: u32) {
//     let file_len = virtual_file_content(&attachment.path).len();
//     let directory_len = virtual_directory_entries(&attachment.path).len();
//     let position = position as usize;

//     attachment.read_offset = match mode {
//         1 => attachment.read_offset.saturating_sub(position),
//         2 => position.min(file_len),
//         3 => attachment
//             .read_offset
//             .saturating_add(position)
//             .min(file_len),
//         4 => file_len.saturating_sub(position),
//         _ => attachment.read_offset,
//     };

//     attachment.directory_offset = match mode {
//         1 => attachment.directory_offset.saturating_sub(position),
//         2 => position.min(directory_len),
//         3 => attachment
//             .directory_offset
//             .saturating_add(position)
//             .min(directory_len),
//         4 => directory_len.saturating_sub(position),
//         _ => attachment.directory_offset,
//     };
// }

// fn virtual_file_content(path: &str) -> &'static [u8] {
//     if path
//         .to_ascii_lowercase()
//         .ends_with("message of the day~text~")
//     {
//         b"Hello World from GRiD Server!\r\n"
//     } else {
//         b""
//     }
// }

// fn apply_set_status(attachment: &mut VfsAttachment, data: &[u8]) {
//     let mut offset = 0;

//     while offset + 3 <= data.len() {
//         let entry_id = data[offset];
//         let length = u16::from_le_bytes([data[offset + 1], data[offset + 2]]) as usize;
//         offset += 3;

//         if offset + length > data.len() {
//             break;
//         }

//         let status_data = &data[offset..offset + length];
//         match entry_id {
//             SET_DIRECTION_ENTRY_ID => {
//                 attachment.directory_offset = 0;
//             }
//             SET_WILDCARD_ENTRY_ID => {
//                 attachment.directory_wildcard = Some(status_data.to_vec());
//                 attachment.directory_offset = 0;
//             }
//             _ => {}
//         }

//         offset += length;
//     }
// }

// fn read_virtual_directory_page(attachment: &mut VfsAttachment, max_objects: u16) -> (Vec<u8>, u16) {
//     let entries: Vec<&str> = virtual_directory_entries(&attachment.path)
//         .iter()
//         .copied()
//         .filter(|entry| matches_directory_wildcard(entry, attachment.directory_wildcard.as_deref()))
//         .collect();
//     let start = attachment.directory_offset.min(entries.len());
//     let object_count = max_objects as usize;
//     let end = start.saturating_add(object_count).min(entries.len());
//     let page = &entries[start..end];

//     attachment.directory_offset = end;

//     match attachment.access {
//         SHORT_DIRECTORY_ACCESS => encode_short_directory_entries(page),
//         // TODO: implement CompleteDirEntryType for longDirectory access once maxFileNameLen
//         // and tSz are recovered from the GRiD OS headers.
//         _ => (Vec::new(), 0),
//     }
// }

// fn virtual_directory_entries(path: &str) -> &'static [&'static str] {
//     let normalized = path.to_ascii_lowercase();
//     if normalized.ends_with("name device`resources~subject~") {
//         NAME_DEVICE_RESOURCE_ENTRIES
//     } else if normalized == format!("`{SERVER_NAME}:{FS_RESOURCE_NAME}").to_ascii_lowercase() {
//         ROOT_DIRECTORY_ENTRIES
//     } else if normalized.ends_with(":server subjects`programs")
//         || normalized.ends_with("`server subjects`programs")
//     {
//         PROGRAMS_DIRECTORY_ENTRIES
//     } else {
//         &[]
//     }
// }

// fn matches_directory_wildcard(name: &str, wildcard: Option<&[u8]>) -> bool {
//     let Some(wildcard) = wildcard else {
//         return true;
//     };

//     let name = ascii_lowercase_bytes(name.as_bytes());
//     let pattern = ascii_lowercase_bytes(wildcard);
//     matches_wildcard(&name, &pattern)
// }

// fn ascii_lowercase_bytes(bytes: &[u8]) -> Vec<u8> {
//     bytes.iter().map(u8::to_ascii_lowercase).collect()
// }

// fn matches_wildcard(mut value: &[u8], mut pattern: &[u8]) -> bool {
//     let mut retry_value = None;
//     let mut retry_pattern = None;

//     while !value.is_empty() {
//         if let Some((&WILDCARD_BYTE, rest)) = pattern.split_first() {
//             retry_value = Some(value);
//             retry_pattern = Some(rest);
//             pattern = rest;
//         } else if pattern.first() == value.first() {
//             value = &value[1..];
//             pattern = &pattern[1..];
//         } else if let (Some(next_value), Some(next_pattern)) = (retry_value, retry_pattern) {
//             if next_value.is_empty() {
//                 return false;
//             }
//             retry_value = Some(&next_value[1..]);
//             value = &next_value[1..];
//             pattern = next_pattern;
//         } else {
//             return false;
//         }
//     }

//     while pattern.first() == Some(&WILDCARD_BYTE) {
//         pattern = &pattern[1..];
//     }

//     pattern.is_empty()
// }

// fn encode_short_directory_entries(entries: &[&str]) -> (Vec<u8>, u16) {
//     let mut data = Vec::new();

//     for name in entries {
//         data.extend([0; 8]);
//         data.push(name.len() as u8);
//         data.extend(name.as_bytes());
//     }

//     (data, entries.len() as u16)
// }

// fn decode_short_directory_entry_names(mut data: &[u8]) -> Vec<String> {
//     let mut names = Vec::new();

//     while data.len() >= 9 {
//         let name_len = data[8] as usize;
//         if data.len() < 9 + name_len {
//             break;
//         }

//         names.push(String::from_utf8_lossy(&data[9..9 + name_len]).into_owned());
//         data = &data[9 + name_len..];
//     }

//     names
// }

// fn vipc_raw_message_response(request_header: &VipcMessageHeader, data: Vec<u8>) -> DataFrameBody {
//     DataFrameBody::Msg {
//         header: VipcMessageHeader {
//             local_path_id: request_header.remote_path_id,
//             remote_path_id: request_header.local_path_id,
//             class: request_header.class,
//             note: request_header.note,
//             data_length: 0,
//         },
//         body: VipcMessageBody::Raw(data),
//     }
// }

// fn vfs_response(
//     request_header: &VipcMessageHeader,
//     request: u16,
//     servers_conn_id: u16,
//     requestors_conn_id: u16,
//     error: u16,
// ) -> DataFrameBody {
//     DataFrameBody::Msg {
//         header: VipcMessageHeader {
//             local_path_id: request_header.remote_path_id,
//             remote_path_id: request_header.local_path_id,
//             class: VFS_CLASS,
//             note: request_header.note,
//             data_length: 0,
//         },
//         body: VipcMessageBody::VfsResponse(VfsResponse::Simple(VfsSimpleResponse {
//             response: request | VFS_RESPONSE_FLAG,
//             servers_conn_id,
//             requestors_conn_id,
//             error,
//         })),
//     }
// }

// fn vfs_read_response(
//     request_header: &VipcMessageHeader,
//     request: u16,
//     servers_conn_id: u16,
//     requestors_conn_id: u16,
//     error: u16,
//     data: Vec<u8>,
// ) -> DataFrameBody {
//     let data_length = data.len() as u16;
//     vfs_read_response_with_data_length(
//         request_header,
//         request,
//         servers_conn_id,
//         requestors_conn_id,
//         error,
//         data_length,
//         data,
//     )
// }

// fn vfs_read_response_with_data_length(
//     request_header: &VipcMessageHeader,
//     request: u16,
//     servers_conn_id: u16,
//     requestors_conn_id: u16,
//     error: u16,
//     data_length: u16,
//     data: Vec<u8>,
// ) -> DataFrameBody {
//     DataFrameBody::Msg {
//         header: VipcMessageHeader {
//             local_path_id: request_header.remote_path_id,
//             remote_path_id: request_header.local_path_id,
//             class: VFS_CLASS,
//             note: request_header.note,
//             data_length: 0,
//         },
//         body: VipcMessageBody::VfsResponse(VfsResponse::Read(VfsReadResponse {
//             common: VfsSimpleResponse {
//                 response: request | VFS_RESPONSE_FLAG,
//                 servers_conn_id,
//                 requestors_conn_id,
//                 error,
//             },
//             data_length,
//             data,
//         })),
//     }
// }
