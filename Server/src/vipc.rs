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
        // match header.class {
        //     MessageClass::Vfs => self.process_vfs_req(&header, data),
        //     MessageClass::Unsupported(class) => todo!(),
        // }

        Ok(())
    }

    fn process_vfs_req(&mut self /* header: &MessageHeader, data: &[u8] */) {
        // let mut payload_reader = io::Cursor::new(payload);
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
        // self.vfs.handle_vfs_request(
        //                                 addr,
        //                                 header,
        //                                 request,
        //                                 &mut last_vfs_conn_id,
        //                                 &mut vfs_attachments,
        //                             )
    }
}
