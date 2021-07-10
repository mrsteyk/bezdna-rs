use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::util;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioBoneT {
    // I don't recall the actual name...
    pub sznameindex: i32,
    pub parent: i32, // -1 - none

    pub bone_controller: [i32; 6], // -1 - none

    pub pos: [f32; 3],
    pub quat: [f32; 4],
    pub rot: [f32; 3],

    pub pos_scale: [f32; 3],
    pub rot_scale: [f32; 3],
    // ???

    // size MUST be 0xf4 if to trust myself...
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioBone {
    start_pos: u64,

    pub name: String,
    pub parent: i32,
    // ???
}

impl StudioBone {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioBone, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let szhitboxnameindex = cursor.read_i32::<LE>()?;

        let parent = cursor.read_i32::<LE>()?;

        let name: String;
        cursor.seek(SeekFrom::Start(start_reading + szhitboxnameindex as u64))?;
        unsafe {
            let mut tmp_name: [u8; 260] = std::mem::zeroed();
            cursor.read_exact(&mut tmp_name)?;
            name = util::str_from_u8_nul_utf8_unchecked(&tmp_name).to_owned();
        }

        Ok(StudioBone {
            start_pos: start_reading,

            name,
            parent,
        })
    }
}
