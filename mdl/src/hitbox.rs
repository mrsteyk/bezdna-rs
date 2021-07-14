use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::util;

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct StudioHitboxT {
    pub bone_id: u32, // Logic left the server...
    pub group: u32,
    pub bbmin: [f32; 3],        // 0x8
    pub bbmax: [f32; 3],        // 0x14
    pub szhitboxnameindex: i32, // 0x20
    pub unused: u64,            // 0x24
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioHitboxSetT {
    pub sznameindex: i32,
    pub numhitboxes: u32,
    pub hitboxindex: i32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioHitbox {
    start_pos: u64,

    pub bone_id: u32,
    pub group: u32,
    pub bb: [[f32; 3]; 2], // min, max
    pub hitbox_name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioHitboxSet {
    start_pos: u64,

    pub name: String,
    pub hitboxes: Vec<StudioHitbox>,
}

impl StudioHitbox {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioHitbox, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let bone_id = cursor.read_u32::<LE>()?;
        let group = cursor.read_u32::<LE>()?;

        let mut bb_min = [0.0f32; 3];
        let mut bb_max = [0.0f32; 3];
        cursor.read_f32_into::<LE>(&mut bb_min)?;
        cursor.read_f32_into::<LE>(&mut bb_max)?;

        let szhitboxnameindex = cursor.read_i32::<LE>()?;

        //println!("brih: {}", szhitboxnameindex);

        let name: String;
        cursor.seek(SeekFrom::Start(start_reading + szhitboxnameindex as u64))?;
        unsafe {
            let mut tmp_name: [u8; 260] = std::mem::zeroed();
            cursor.read_exact(&mut tmp_name)?;
            name = util::str_from_u8_nul_utf8_unchecked(&tmp_name).to_owned();
        }

        Ok(StudioHitbox {
            start_pos: start_reading,
            bone_id,
            group,
            bb: [bb_min, bb_max],
            hitbox_name: name,
        })
    }
}

impl StudioHitboxSet {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioHitboxSet, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let szhitboxnameindex = cursor.read_i32::<LE>()?;
        let num_hitboxes = cursor.read_u32::<LE>()?;
        let hitboxindex = cursor.read_i32::<LE>()?;

        let mut hitboxes: Vec<StudioHitbox> = Vec::new();

        let name: String;
        cursor.seek(SeekFrom::Start(start_reading + szhitboxnameindex as u64))?;
        unsafe {
            let mut tmp_name: [u8; 260] = std::mem::zeroed();
            cursor.read_exact(&mut tmp_name)?;
            name = util::str_from_u8_nul_utf8_unchecked(&tmp_name).to_owned();
        }

        //println!("hbsx: {}", num_hitboxes);

        for i in 0..num_hitboxes {
            cursor.seek(SeekFrom::Start(
                start_reading + hitboxindex as u64 + (0x44 * i) as u64,
            ))?;
            hitboxes.push(StudioHitbox::read(cursor)?);
        }

        Ok(StudioHitboxSet {
            start_pos: start_reading,

            name,
            hitboxes,
        })
    }
}
