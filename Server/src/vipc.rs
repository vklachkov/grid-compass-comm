use std::rc::Rc;

use crate::{gridlink::*, vfs::Vfs};

pub struct Vipc {
    vfs: Rc<Box<Vfs>>,
}

impl Vipc {
    pub fn new(vfs: Rc<Box<Vfs>>) -> Self {
        Self { vfs }
    }

    pub fn process_message(
        &mut self,
        header: ConnectHeader,
        payload: &[u8],
    ) -> Result<(), FrameError> {
        let message = IpcMessage::try_from_slice(payload)?;

        match message.body {
            IpcMessageBody::Vfs(req) => self.process_vfs_req(req),
            IpcMessageBody::Unsupported(raw) => todo!(),
        }

        Ok(())
    }

    fn process_vfs_req(&mut self, req: VfsRequest) {}
}
