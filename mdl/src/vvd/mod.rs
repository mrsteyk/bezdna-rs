use std::io::{Read, Seek};

use byteorder::ReadBytesExt;

#[derive(Debug, PartialEq, Clone)]
pub struct VvdFile {
    start_pos: u64,
    //
}

impl VvdFile {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<Self, std::io::Error> {
        let start_pos = cursor.stream_position()?;

        Ok(Self { start_pos })
    }
}
