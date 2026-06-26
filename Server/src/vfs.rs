use std::{collections::HashMap, num::NonZeroU16};

use crate::gridlink::vipc::*;

pub struct Vfs {
    connection_id: NonZeroU16,
    files: HashMap<NonZeroU16, VfsFileDescriptor>,
}

struct VfsFileDescriptor {}

impl Vfs {
    pub fn new() -> Self {
        Self {
            connection_id: NonZeroU16::MIN,
            files: HashMap::new(),
        }
    }

    pub fn process_request(&mut self, req: VfsRequest) -> OutgoingMessageBody {
        match req.body {
            VfsRequestBody::Open(..) => todo!(),
            VfsRequestBody::Read(..) => todo!(),
            VfsRequestBody::Write(..) => todo!(),
            VfsRequestBody::Seek(..) => todo!(),
            VfsRequestBody::Attach(body) => self.attach(&req.header, body),
            _ => unimplemented!(),
        }
    }

    fn attach(
        &mut self,
        header: &VfsRequestHeader,
        body: VfsAttachRequest<'_>,
    ) -> OutgoingMessageBody {
        let conn_id = self.connection_id;

        if self.files.contains_key(&conn_id) {
            unimplemented!();
        }

        self.files.insert(
            conn_id,
            VfsFileDescriptor {
                // TODO
            },
        );

        // no wrapping_add() for NonZero types.
        self.connection_id = conn_id.checked_add(1).unwrap_or(NonZeroU16::MIN);

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Attach as u16,
            servers_conn_id: conn_id.get(),
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }
}
