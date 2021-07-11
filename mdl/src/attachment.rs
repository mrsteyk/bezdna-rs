use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

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

        let name: String;
        cursor.seek(SeekFrom::Start(start_reading + snameindex as u64))?;
        unsafe {
            let mut tmp_name: [u8; 260] = std::mem::zeroed();
            cursor.read_exact(&mut tmp_name)?;
            name = util::str_from_u8_nul_utf8_unchecked(&tmp_name).to_owned();
        }

        Ok(StudioAttachment {
            start_pos: start_reading,

            name,
            flags,
            local_bone,
            local,
        })
    }
}
