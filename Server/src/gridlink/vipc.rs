use bstr::BStr;

#[derive(Clone, Copy, Debug)]
pub struct VipcMessageHeader {
    pub local_path_id: u16,  // localPathID
    pub remote_path_id: u16, // remotePathID
    pub class: u16,          // class
    pub note: u16,           // note
    pub data_length: u16,    // vipcDataLength
}

#[derive(Clone, Debug)]
pub struct VfsRequest<'a> {
    pub header: VfsRequestHeader, // VfsRequestCommonPart
    pub body: VfsRequestBody<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum VfsRequestCode {
    GetStatus = 1,    // ddGetStatus
    Open = 2,         // ddOpen
    Close = 3,        // ddClose
    Read = 4,         // ddRead
    Write = 5,        // ddWrite
    Seek = 6,         // ddSeek
    Attach = 8,       // ddAttach
    Detach = 9,       // ddDetach
    ReadDesc = 12,    // ddReadDesc
    WriteDesc = 13,   // ddWriteDesc
    SetStatus = 20,   // ddSetStatus
    ReadDirPage = 29, // ddReadDirPage
}

#[derive(Clone, Debug)]
pub enum VfsRequestBody<'a> {
    // AttachReqType
    Attach(VfsAttachRequest<'a>),

    // OpenReqType
    Open(VfsOpenRequest),

    // ReadReqType
    Read(VfsReadRequest),

    // SeekReqType
    Seek(VfsSeekRequest),

    // WriteReqType
    Write(VfsWriteRequest<'a>),

    // SimpleReqType
    Simple,

    // raw
    Raw(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct VfsAttachRequest<'a> {
    pub mode: u8,           // mode
    pub access: u8,         // access
    pub password: [u8; 17], // password
    pub path: &'a BStr,
}

#[derive(Clone, Copy, Debug)]
pub struct VfsOpenRequest {
    pub num_buf: u8, // numBuf
}

#[derive(Clone, Copy, Debug)]
pub struct VfsReadRequest {
    pub data_length: u16, // vfsDatalength
}

#[derive(Clone, Copy, Debug)]
pub struct VfsSeekRequest {
    pub mode: u8,      // mode
    pub position: u32, // position
}

#[derive(Clone, Debug)]
pub struct VfsWriteRequest<'a> {
    pub data: &'a [u8], // buffer
}

#[derive(Clone, Copy, Debug)]
pub struct VfsRequestHeader {
    pub request: u16,            // vfsRequest
    pub requestors_conn_id: u16, // requestorsConnID
    pub servers_conn_id: u16,    // serversConnID
}

#[derive(Clone, Debug)]
pub enum VfsResponse<'a> {
    // SimpleRespType
    Simple(VfsSimpleResponse),

    // ReadRespType
    Read(VfsReadResponse<'a>),
}

#[derive(Clone, Copy, Debug)]
pub struct VfsSimpleResponse {
    pub response: u16,           // vfsResponse
    pub servers_conn_id: u16,    // serversConnID
    pub requestors_conn_id: u16, // requestorsConnID
    pub error: u16,              // Error
}

#[derive(Clone, Debug)]
pub struct VfsReadResponse<'a> {
    pub common: VfsSimpleResponse, // VfsRespCommonPart
    pub data_length: u16,          // vfsDatalength
    pub data: &'a [u8],            // buffer
}
