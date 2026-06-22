mod gridlink;

use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    process::ExitCode,
    thread,
};

use crate::gridlink::{
    DataFrameBody, EOM_FLAG_ON, Frame, FrameBody, FrameReadError, FrameType, VfsReadResponse,
    VfsRequest, VfsRequestBody, VfsRequestCode, VfsResponse, VfsSimpleResponse, VipcConnectHeader,
    VipcMessageBody, VipcMessageHeader,
};

const VFS_CLASS: u16 = 83;
const VFS_RESPONSE_FLAG: u16 = 0x8000;
const VFS_ERROR_OK: u16 = 0;
const VFS_ERROR_UNSUPPORTED: u16 = 1;

#[derive(Debug)]
#[allow(dead_code)]
struct VfsAttachment {
    server_conn_id: u16,
    requestors_conn_id: u16,
    vipc_local_path_id: u16,
    vipc_remote_path_id: u16,
    mode: u8,
    access: u8,
    password: [u8; 17],
    path: String,
    open: bool,
    num_buf: Option<u8>,
    read_offset: usize,
}

fn main() -> ExitCode {
    match server() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Fatal error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn server() -> io::Result<()> {
    let addr = std::env::var("LISTEN_ADDR").map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "env var LISTEN_ADDR not found")
    })?;

    let listener = TcpListener::bind(&addr)?;

    println!("Start GRiD Server at {addr}");
    loop {
        match listener.accept() {
            Ok((client, addr)) => {
                println!("Accepted client {addr}");
                thread::spawn(move || worker(client, addr));
            }
            Err(err) => {
                eprintln!("Failed to accept client: {err}");
            }
        };
    }
}

fn worker(client: TcpStream, addr: SocketAddr) {
    if let Err(err) = try_worker(client, addr) {
        eprintln!("worker({addr}): fatal error: {err}");
    }
}

fn try_worker(mut client: TcpStream, addr: SocketAddr) -> io::Result<()> {
    let conn_id: u8 = 0x7B;
    let mut seq_number: u8 = 0x1C;

    let mut last_path_id: u16 = 1;
    let mut last_vfs_conn_id: u16 = 0;
    let mut vfs_attachments = HashMap::<u16, VfsAttachment>::new();

    loop {
        let frame = match gridlink::Frame::read_from_io(&mut client) {
            Ok(frame) => frame,
            Err(FrameReadError::Io(err)) if err.kind() == io::ErrorKind::UnexpectedEof => {
                println!("worker({addr}): connection closed");
                return Ok(());
            }
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
        };

        println!("worker({addr}): received {frame:#?}");

        let incoming_seq_number = frame.header().seq_number;

        match frame.header().ty {
            FrameType::Rfc => {
                Frame::rfc(conn_id, seq_number, EOM_FLAG_ON).write_to_io(&mut client)?;
            }
            FrameType::Ack => {
                // TODO
            }
            FrameType::Disc => {
                println!("worker({addr}): disconnect");
                return Ok(());
            }
            FrameType::Ping => {
                Frame::ack(conn_id, incoming_seq_number, EOM_FLAG_ON).write_to_io(&mut client)?;
            }
            FrameType::Data => {
                Frame::ack(conn_id, incoming_seq_number, EOM_FLAG_ON).write_to_io(&mut client)?;

                let data = match frame.body() {
                    FrameBody::Data(data) => data,
                    _ => unreachable!(),
                };

                match data {
                    DataFrameBody::Connect { header, path } => {
                        seq_number = seq_number.wrapping_add(1);
                        last_path_id = last_path_id.wrapping_add(1);
                        println!("worker({addr}): connected path {last_path_id} to {path:?}");
                        Frame::data(
                            seq_number,
                            EOM_FLAG_ON,
                            DataFrameBody::ConnectResponse {
                                header: VipcConnectHeader {
                                    local_path_id: last_path_id,
                                    remote_path_id: header.local_path_id,
                                },
                                status: 0, // OK
                            },
                        )
                        .write_to_io(&mut client)?;
                    }
                    DataFrameBody::Disconnect { header, reason } => {
                        let local_path_id = header.remote_path_id;
                        let remote_path_id = header.local_path_id;
                        println!(
                            "worker({addr}): VIPC disconnect local_path_id {}, remote_path_id {}, reason {}",
                            local_path_id, remote_path_id, reason
                        );

                        seq_number = seq_number.wrapping_add(1);
                        Frame::data(
                            seq_number,
                            EOM_FLAG_ON,
                            DataFrameBody::DisconnectResponse {
                                header: VipcConnectHeader {
                                    local_path_id,
                                    remote_path_id,
                                },
                            },
                        )
                        .write_to_io(&mut client)?;
                    }
                    DataFrameBody::Msg { header, body } => {
                        let response = match body {
                            VipcMessageBody::VfsRequest(request) => handle_vfs_request(
                                addr,
                                header,
                                request,
                                &mut last_vfs_conn_id,
                                &mut vfs_attachments,
                            ),
                            VipcMessageBody::Raw(raw) => {
                                let class = header.class;
                                println!(
                                    "worker({addr}): unsupported VIPC message class {}, {} bytes",
                                    class,
                                    raw.len()
                                );
                                None
                            }
                            VipcMessageBody::VfsResponse(_) => None,
                        };

                        if let Some(response) = response {
                            seq_number = seq_number.wrapping_add(1);
                            Frame::data(seq_number, EOM_FLAG_ON, response)
                                .write_to_io(&mut client)?;
                        }
                    }
                    DataFrameBody::SignOn { .. } => {
                        seq_number = seq_number.wrapping_add(1);
                        Frame::data(
                            seq_number,
                            EOM_FLAG_ON,
                            DataFrameBody::SignOnResponse {
                                status: 0, // OK
                                server_name: "vklachkov server",
                            },
                        )
                        .write_to_io(&mut client)?;
                    }
                    DataFrameBody::SignOff => {
                        println!("worker({addr}): sign off");
                        return Ok(());
                    }
                    DataFrameBody::ConnectResponse { .. }
                    | DataFrameBody::DisconnectResponse { .. }
                    | DataFrameBody::SignOnResponse { .. } => {
                        println!("worker({addr}): unexpected outbound-only data body");
                    }
                }
            }
        };
    }
}

fn handle_vfs_request(
    addr: SocketAddr,
    header: &VipcMessageHeader,
    request: &VfsRequest,
    last_vfs_conn_id: &mut u16,
    vfs_attachments: &mut HashMap<u16, VfsAttachment>,
) -> Option<DataFrameBody> {
    let error = match &request.body {
        VfsRequestBody::Attach(attach) => {
            *last_vfs_conn_id = last_vfs_conn_id.wrapping_add(1);
            if *last_vfs_conn_id == 0 {
                *last_vfs_conn_id = 1;
            }

            vfs_attachments.insert(
                *last_vfs_conn_id,
                VfsAttachment {
                    server_conn_id: *last_vfs_conn_id,
                    requestors_conn_id: request.header.requestors_conn_id,
                    vipc_local_path_id: header.remote_path_id,
                    vipc_remote_path_id: header.local_path_id,
                    mode: attach.mode,
                    access: attach.access,
                    password: attach.password,
                    path: attach.path.clone(),
                    open: false,
                    num_buf: None,
                    read_offset: 0,
                },
            );

            println!(
                "worker({addr}): attached VFS conn {} to {:?}",
                *last_vfs_conn_id, attach.path
            );

            VFS_ERROR_OK
        }
        VfsRequestBody::Open(open) => {
            let servers_conn_id = request.header.servers_conn_id;
            match vfs_attachments.get_mut(&servers_conn_id) {
                Some(attachment) => {
                    attachment.open = true;
                    attachment.num_buf = Some(open.num_buf);

                    println!(
                        "worker({addr}): opened VFS conn {} path {:?}, num_buf {}",
                        servers_conn_id, attachment.path, open.num_buf
                    );

                    VFS_ERROR_OK
                }
                None => {
                    println!(
                        "worker({addr}): open for unknown VFS conn {}",
                        servers_conn_id
                    );
                    VFS_ERROR_UNSUPPORTED
                }
            }
        }
        VfsRequestBody::Read(read) if request.header.request == VfsRequestCode::Read.as_u16() => {
            let servers_conn_id = request.header.servers_conn_id;
            let Some(attachment) = vfs_attachments.get_mut(&servers_conn_id) else {
                println!(
                    "worker({addr}): read for unknown VFS conn {}",
                    servers_conn_id
                );

                return Some(vfs_read_response(
                    header,
                    request.header.request,
                    servers_conn_id,
                    request.header.requestors_conn_id,
                    VFS_ERROR_UNSUPPORTED,
                    Vec::new(),
                ));
            };

            let data = read_virtual_file(attachment, read.data_length as usize);
            println!(
                "worker({addr}): read VFS conn {} path {:?}, requested {} bytes, sent {} bytes",
                servers_conn_id,
                attachment.path,
                read.data_length,
                data.len()
            );

            return Some(vfs_read_response(
                header,
                request.header.request,
                servers_conn_id,
                request.header.requestors_conn_id,
                VFS_ERROR_OK,
                data,
            ));
        }
        VfsRequestBody::Read(read)
            if request.header.request == VfsRequestCode::ReadDesc.as_u16() =>
        {
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): read descriptor for VFS conn {}, requested {} bytes",
                servers_conn_id, read.data_length
            );

            return Some(vfs_read_response(
                header,
                request.header.request,
                servers_conn_id,
                request.header.requestors_conn_id,
                VFS_ERROR_OK,
                Vec::new(),
            ));
        }
        VfsRequestBody::Read(read)
            if request.header.request == VfsRequestCode::ReadDirPage.as_u16() =>
        {
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): read directory page for VFS conn {}, requested {} objects",
                servers_conn_id, read.data_length
            );

            return Some(vfs_read_response(
                header,
                request.header.request,
                servers_conn_id,
                request.header.requestors_conn_id,
                VFS_ERROR_OK,
                Vec::new(),
            ));
        }
        VfsRequestBody::Seek(seek) if request.header.request == VfsRequestCode::Seek.as_u16() => {
            let servers_conn_id = request.header.servers_conn_id;
            match vfs_attachments.get_mut(&servers_conn_id) {
                Some(attachment) => {
                    apply_seek(attachment, seek.mode, seek.position);

                    println!(
                        "worker({addr}): seek VFS conn {} path {:?}, mode {}, position {}, offset {}",
                        servers_conn_id,
                        attachment.path,
                        seek.mode,
                        seek.position,
                        attachment.read_offset
                    );

                    VFS_ERROR_OK
                }
                None => {
                    println!(
                        "worker({addr}): seek for unknown VFS conn {}, mode {}, position {}",
                        servers_conn_id, seek.mode, seek.position
                    );

                    VFS_ERROR_UNSUPPORTED
                }
            }
        }
        VfsRequestBody::Write(write)
            if request.header.request == VfsRequestCode::SetStatus.as_u16() =>
        {
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): set status for VFS conn {}, {} bytes",
                servers_conn_id,
                write.data.len()
            );

            VFS_ERROR_OK
        }
        VfsRequestBody::Simple if request.header.request == VfsRequestCode::Detach.as_u16() => {
            let servers_conn_id = request.header.servers_conn_id;
            let detached = vfs_attachments.remove(&servers_conn_id);
            println!(
                "worker({addr}): detached VFS conn {}, existed={}",
                servers_conn_id,
                detached.is_some()
            );

            VFS_ERROR_OK
        }
        VfsRequestBody::Raw(payload) => {
            let request_code = request.header.request;
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): unsupported VFS request {}, servers_conn_id {}, {} payload bytes",
                request_code,
                servers_conn_id,
                payload.len()
            );
            VFS_ERROR_UNSUPPORTED
        }
        VfsRequestBody::Read(read) => {
            let request_code = request.header.request;
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): unsupported VFS read request {}, servers_conn_id {}, requested {} bytes",
                request_code, servers_conn_id, read.data_length
            );
            VFS_ERROR_UNSUPPORTED
        }
        VfsRequestBody::Seek(seek) => {
            let request_code = request.header.request;
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): unsupported VFS seek request {}, servers_conn_id {}, mode {}, position {}",
                request_code, servers_conn_id, seek.mode, seek.position
            );
            VFS_ERROR_UNSUPPORTED
        }
        VfsRequestBody::Write(write) => {
            let request_code = request.header.request;
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): unsupported VFS write request {}, servers_conn_id {}, {} bytes",
                request_code,
                servers_conn_id,
                write.data.len()
            );
            VFS_ERROR_UNSUPPORTED
        }
        VfsRequestBody::Simple => {
            let request_code = request.header.request;
            let servers_conn_id = request.header.servers_conn_id;
            println!(
                "worker({addr}): unsupported VFS simple request {}, servers_conn_id {}",
                request_code, servers_conn_id
            );
            VFS_ERROR_UNSUPPORTED
        }
    };

    Some(vfs_response(
        header,
        request.header.request,
        if matches!(&request.body, VfsRequestBody::Attach(_)) && error == VFS_ERROR_OK {
            *last_vfs_conn_id
        } else {
            request.header.servers_conn_id
        },
        request.header.requestors_conn_id,
        error,
    ))
}

fn read_virtual_file(attachment: &mut VfsAttachment, max_len: usize) -> Vec<u8> {
    let content = virtual_file_content(&attachment.path);
    let remaining = &content[attachment.read_offset.min(content.len())..];
    let read_len = max_len.min(remaining.len());
    let data = remaining[..read_len].to_vec();
    attachment.read_offset += read_len;
    data
}

fn apply_seek(attachment: &mut VfsAttachment, mode: u8, position: u32) {
    let len = virtual_file_content(&attachment.path).len();
    let position = position as usize;

    attachment.read_offset = match mode {
        1 => attachment.read_offset.saturating_sub(position),
        2 => position.min(len),
        3 => attachment.read_offset.saturating_add(position).min(len),
        4 => len.saturating_sub(position),
        _ => attachment.read_offset,
    };
}

fn virtual_file_content(path: &str) -> &'static [u8] {
    if path.ends_with("Message Of The Day~Text~") {
        b"Hello World from GRiD Server!\r\n"
    } else {
        b""
    }
}

fn vfs_response(
    request_header: &VipcMessageHeader,
    request: u16,
    servers_conn_id: u16,
    requestors_conn_id: u16,
    error: u16,
) -> DataFrameBody {
    DataFrameBody::Msg {
        header: VipcMessageHeader {
            local_path_id: request_header.remote_path_id,
            remote_path_id: request_header.local_path_id,
            class: VFS_CLASS,
            note: request_header.note,
            data_length: 0,
        },
        body: VipcMessageBody::VfsResponse(VfsResponse::Simple(VfsSimpleResponse {
            response: request | VFS_RESPONSE_FLAG,
            servers_conn_id,
            requestors_conn_id,
            error,
        })),
    }
}

fn vfs_read_response(
    request_header: &VipcMessageHeader,
    request: u16,
    servers_conn_id: u16,
    requestors_conn_id: u16,
    error: u16,
    data: Vec<u8>,
) -> DataFrameBody {
    DataFrameBody::Msg {
        header: VipcMessageHeader {
            local_path_id: request_header.remote_path_id,
            remote_path_id: request_header.local_path_id,
            class: VFS_CLASS,
            note: request_header.note,
            data_length: 0,
        },
        body: VipcMessageBody::VfsResponse(VfsResponse::Read(VfsReadResponse {
            common: VfsSimpleResponse {
                response: request | VFS_RESPONSE_FLAG,
                servers_conn_id,
                requestors_conn_id,
                error,
            },
            data,
        })),
    }
}
