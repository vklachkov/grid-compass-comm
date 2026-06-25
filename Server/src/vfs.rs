use crate::gridlink::*;

pub struct Vfs {}

impl Vfs {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_request(&mut self, req: VfsRequest) {
        match req.body {
            VfsRequestBody::Open(..) => {}
            VfsRequestBody::Read(..) => {}
            VfsRequestBody::Write(..) => {}
            VfsRequestBody::Seek(..) => {}
            VfsRequestBody::Attach(..) => {}
            _ => unimplemented!(),
        }
    }
}
