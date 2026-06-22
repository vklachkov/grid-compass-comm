use std::{io, slice};

use super::error::FrameError;

/// Data Link Escape. Used to prefix special commands or escape data bytes.
const DLE: u8 = 0x10;
/// Start of Text. Marks the beginning of the payload.
const STX: u8 = 0x02;
/// End of Text. Marks the end of the payload.
const ETX: u8 = 0x03;

/// Size of the preallocated buffer for a frame.
const AVERAGE_FRAME_SIZE: usize = 64;

/// Number of bytes after which the reader returns an error.
const MAX_FRAME_SIZE: usize = 512;

#[derive(Clone, Debug)]
pub struct RawFrame {
    pub data: Vec<u8>,
}

impl RawFrame {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Reads and unstuffs frame data from an I/O source.
    pub fn read_from_io(mut src: impl io::Read) -> Result<Self, FrameError> {
        let buffer = Self::read_unstuffed(&mut src)?;
        let buffer_crc = crc16(&buffer);

        let mut crc = [0u8; 2];
        src.read_exact(&mut crc)?;

        let crc = u16::from_le_bytes(crc);
        if crc != buffer_crc {
            return Err(FrameError::InvalidCrc {
                expected: buffer_crc,
                found: crc,
            });
        }

        Ok(Self::new(buffer))
    }

    fn read_unstuffed(mut src: impl io::Read) -> Result<Vec<u8>, FrameError> {
        let mut byte = 0u8;
        let mut buffer = Vec::with_capacity(AVERAGE_FRAME_SIZE);

        loop {
            if buffer.len() >= MAX_FRAME_SIZE {
                return Err(FrameError::FrameTooLarge {
                    max: MAX_FRAME_SIZE,
                });
            }

            src.read_exact(slice::from_mut(&mut byte))?;
            if byte != DLE {
                buffer.push(byte);
                continue;
            }

            src.read_exact(slice::from_mut(&mut byte))?;
            match byte {
                DLE => buffer.push(DLE),
                STX => buffer.clear(),
                ETX => break,
                _ => {
                    return Err(FrameError::MalformedFrameMarker { marker: byte });
                }
            }
        }

        Ok(buffer)
    }

    /// Stuffs and writes frame data to an I/O destination.
    pub fn write_to_io(&self, dst: impl io::Write) -> Result<usize, FrameError> {
        let crc = crc16(&self.data);

        let count_of_dle = self.data.iter().filter(|&&b| b == DLE).count();
        if count_of_dle == 0 {
            return Self::write_stuffed(dst, &self.data, crc);
        }

        let mut stuffed_frame_data = Vec::with_capacity(self.data.len() + count_of_dle);
        for &b in self.data.iter() {
            stuffed_frame_data.push(b);
            if b == DLE {
                stuffed_frame_data.push(DLE);
            }
        }

        Self::write_stuffed(dst, &stuffed_frame_data, crc)
    }

    fn write_stuffed(mut dst: impl io::Write, data: &[u8], crc: u16) -> Result<usize, FrameError> {
        let mut total = 0;

        dst.write_all(&[DLE, STX])?;
        total += 2;

        dst.write_all(data)?;
        total += data.len();

        dst.write_all(&[DLE, ETX])?;
        total += 2;

        dst.write_all(&crc.to_le_bytes())?;
        total += 2;

        Ok(total)
    }
}

/// Calculates the CRC16 ARC checksum for the given data.
fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0;

    for byte in data {
        crc ^= *byte as u16;

        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }

    crc
}
