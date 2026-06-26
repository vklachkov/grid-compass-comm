use crate::{gridlink::FrameError, gridlink::vipc::*, vfs::Vfs};

pub struct Vipc {
    vfs: Box<Vfs>,
}

impl Vipc {
    pub fn new(vfs: Box<Vfs>) -> Self {
        Self { vfs }
    }

    pub fn process_message(
        &mut self,
        payload: &[u8],
    ) -> Result<Option<OutgoingMessage>, FrameError> {
        let message = IncomingMessage::try_from_slice(payload)?;

        println!("session: received vipc message: {message:?}");

        let response = match message.body {
            IncomingMessageBody::Vfs(req) => Some(self.vfs.process_request(req)),
            IncomingMessageBody::Unsupported(_raw) => None,
        };

        Ok(response.map(|r| OutgoingMessage {
            note: message.note,
            body: r,
        }))
    }
}
