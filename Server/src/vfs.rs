use std::{collections::HashMap, num::NonZeroU16};

use crate::gridlink::vipc::*;

const RESOURCES: &[&str] = &["Hard Disk~FS~"];
const HARD_DISK: &[&str] = &[
    "Folder 1~Subject~",
    "Folder 3~Subject~",
    "Folder 2~Subject~",
];
const HARD_DISK_FILES: &[&str] = &["Demo file~Text~"];

pub struct Vfs {
    connection_id: NonZeroU16,
    files: HashMap<NonZeroU16, VfsFileDescriptor>,
}

struct VfsFileDescriptor {
    resource: VfsResource,
    read_dir_page_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VfsResource {
    Resources,
    HardDisk,
    HardDiskFiles,
    Unknown,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            connection_id: NonZeroU16::MIN,
            files: HashMap::new(),
        }
    }

    pub fn process_request(&mut self, req: VfsRequest) -> OutgoingMessageBody {
        match req.body {
            VfsRequestBody::GetStatus(body) => self.get_status(&req.header, body),
            VfsRequestBody::Open(body) => self.open(&req.header, body),
            VfsRequestBody::Read(body) => self.read(&req.header, body),
            VfsRequestBody::ReadDesc(body) => self.read_desc(&req.header, body),
            VfsRequestBody::ReadDirPage(body) => self.read_dir_page(&req.header, body),
            VfsRequestBody::Write(body) => self.write(&req.header, body),
            VfsRequestBody::WriteDesc(body) => self.write_desc(&req.header, body),
            VfsRequestBody::SetStatus(body) => self.set_status(&req.header, body),
            VfsRequestBody::Seek(body) => self.seek(&req.header, body),
            VfsRequestBody::Attach(body) => self.attach(&req.header, body),
            VfsRequestBody::Detach => self.detach(&req.header),
            VfsRequestBody::Close => self.close(&req.header),
            VfsRequestBody::Unknown(body) => self.unknown(body),
        }
    }

    fn get_status(
        &mut self,
        header: &VfsRequestHeader,
        _body: VfsReadRequest,
    ) -> OutgoingMessageBody {
        OutgoingMessageBody::VfsGetStatus(VfsGetStatusResponse {
            header: VfsResponseHeader {
                response: 0x8000 | VfsRequestCode::GetStatus as u16,
                servers_conn_id: header.servers_conn_id,
                requestors_conn_id: header.requestors_conn_id,
                error: 0, // Ok
            },
            // FIXME: replace with real data
            open: true,
            access: VfsAccessMode::Read,
            seek: true,
            file_position: 0,
            file_length: 0,
            num_pages: 0,
            num_pages_alloc: 0,
        })
    }

    fn open(&mut self, header: &VfsRequestHeader, _body: VfsOpenRequest) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Open as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn read(&mut self, header: &VfsRequestHeader, _body: VfsReadRequest) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsRead(VfsReadResponse {
            header: VfsResponseHeader {
                response: 0x8000 | VfsRequestCode::Read as u16,
                servers_conn_id: header.servers_conn_id,
                requestors_conn_id: header.requestors_conn_id,
                error: 0, // Ok
            },
            data: b"Read stub".to_vec(),
        })
    }

    fn read_desc(
        &mut self,
        header: &VfsRequestHeader,
        _body: VfsReadRequest,
    ) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsRead(VfsReadResponse {
            header: VfsResponseHeader {
                response: 0x8000 | VfsRequestCode::ReadDesc as u16,
                servers_conn_id: header.servers_conn_id,
                requestors_conn_id: header.requestors_conn_id,
                error: 0, // Ok
            },
            data: Vec::new(),
        })
    }

    fn read_dir_page(
        &mut self,
        header: &VfsRequestHeader,
        body: VfsReadRequest,
    ) -> OutgoingMessageBody {
        let entries = NonZeroU16::new(header.servers_conn_id)
            .and_then(|conn_id| self.files.get_mut(&conn_id))
            .map(|file| file.read_dir_page(body.data_length as usize))
            .unwrap_or_default();

        OutgoingMessageBody::VfsReadDirPage(VfsReadDirPageResponse {
            header: VfsResponseHeader {
                response: 0x8000 | VfsRequestCode::ReadDirPage as u16,
                servers_conn_id: header.servers_conn_id,
                requestors_conn_id: header.requestors_conn_id,
                error: 0, // Ok
            },
            entries,
        })
    }

    fn write(
        &mut self,
        header: &VfsRequestHeader,
        _body: VfsWriteRequest<'_>,
    ) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Write as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn write_desc(
        &mut self,
        header: &VfsRequestHeader,
        _body: VfsWriteRequest<'_>,
    ) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::WriteDesc as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn set_status(
        &mut self,
        header: &VfsRequestHeader,
        _body: VfsSetStatusRequest<'_>,
    ) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::SetStatus as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn seek(&mut self, header: &VfsRequestHeader, _body: VfsSeekRequest) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Seek as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
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
                resource: VfsResource::from_components(&body.path.components),
                read_dir_page_offset: 0,
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

    fn detach(&mut self, header: &VfsRequestHeader) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Detach as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn close(&mut self, header: &VfsRequestHeader) -> OutgoingMessageBody {
        // TODO

        OutgoingMessageBody::VfsSimple(VfsResponseHeader {
            response: 0x8000 | VfsRequestCode::Close as u16,
            servers_conn_id: header.servers_conn_id,
            requestors_conn_id: header.requestors_conn_id,
            error: 0, // Ok
        })
    }

    fn unknown(&mut self, _body: &[u8]) -> OutgoingMessageBody {
        panic!("unsupported VFS request")
    }
}

impl VfsFileDescriptor {
    fn read_dir_page(&mut self, max_entries: usize) -> Vec<VfsShortDirEntry> {
        let entries = match self.resource {
            VfsResource::Resources => RESOURCES,
            VfsResource::HardDisk => HARD_DISK,
            VfsResource::HardDiskFiles => HARD_DISK_FILES,
            VfsResource::Unknown => return Vec::new(),
        };

        let mut page = Vec::new();

        for name in entries
            .iter()
            .skip(self.read_dir_page_offset)
            .take(max_entries)
        {
            page.push(VfsShortDirEntry {
                name: name.as_bytes().to_vec(),
            });
            self.read_dir_page_offset += 1;
        }

        page
    }
}

impl VfsResource {
    fn from_components(components: &[&bstr::BStr]) -> Self {
        if components.len() == 2
            && components[0] == b"Name Device"
            && components[1] == b"Resources~Subject~"
        {
            Self::Resources
        } else if components.len() == 1 && components[0] == b"Hard Disk" {
            Self::HardDisk
        } else if components.len() == 2
            && components[0] == b"Hard Disk"
            && components[1] == b"Folder 3~Subject~"
        {
            Self::HardDiskFiles
        } else {
            Self::Unknown
        }
    }
}
