use crate::{gridlink::FrameError, gridlink::vipc::*, vfs::Vfs};

pub struct Vipc {
    vfs: Box<Vfs>,
}

impl Vipc {
    pub fn new(vfs: Box<Vfs>) -> Self {
        Self { vfs }
    }

    pub fn process_message(&mut self, payload: &[u8]) -> Result<OutgoingMessage, FrameError> {
        let message = IncomingMessage::try_from_slice(payload)?;

        println!("session: received vipc message: {message:?}");

        let response = match message.body {
            IncomingMessageBody::Vfs(req) => self.vfs.process_request(req),
            IncomingMessageBody::Unsupported(_raw) => todo!(),
        };

        Ok(OutgoingMessage {
            note: message.note,
            body: response,
        })
    }
}
