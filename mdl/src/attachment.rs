use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::util;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioAttachmentT {
    pub snameindex: i32,
    pub flags: u32,
    pub local_bone: u32,
    pub local: [f32; 12],
    pub unused: [u32; 8],
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioAttachment {
    start_pos: u64,

    pub name: String,
    pub flags: u32,
    pub local_bone: u32,
    pub local: [f32; 12],
}

impl StudioAttachment {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioAttachment, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let snameindex = cursor.read_i32::<LE>()?;

        let flags = cursor.read_u32::<LE>()?;
        let local_bone = cursor.read_u32::<LE>()?;

        let mut local = [0.0f32; 12];
        cursor.read_f32_into::<LE>(&mut local)?;

        cursor.seek(SeekFrom::Start(start_reading + snameindex as u64))?;
        let name = util::string_from_buf(cursor);

        Ok(StudioAttachment {
            start_pos: start_reading,

            name,
            flags,
            local_bone,
            local,
        })
    }
}
