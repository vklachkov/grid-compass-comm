mod gridlink;

use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    process::ExitCode,
    thread,
};

use crate::gridlink::{DataFrameBody, EOM_FLAG_ON, Frame, FrameBody, FrameType};

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

    loop {
        let frame = gridlink::Frame::read_from_io(&mut client)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

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
                    DataFrameBody::SignOn { .. } => {
                        seq_number = seq_number.wrapping_add(1);
                        Frame::data(
                            seq_number,
                            EOM_FLAG_ON,
                            DataFrameBody::SignOnResponse {
                                sign_on_status: 0, // OK
                                server_name: "vklachkov server",
                            },
                        )
                        .write_to_io(&mut client)?;
                    }
                    _ => unreachable!(),
                }
            }
        };
    }
}
