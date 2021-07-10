use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::util;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioBodyPartT {
    pub sznameindex: u32, // 0x0
    pub nummodels: u32,   // 0x4
    pub base: u32,        // 0x8
    pub modelindex: u32,  // 0xC - offset
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioBodyPart {
    start_pos: u64,

    pub name: String,
    pub base: u32,

    // TODO: change to models
    pub models: Vec<u32>,
}

impl StudioBodyPart {
    // TODO: better Error handling?...
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioBodyPart, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let name_index = cursor.read_u32::<LE>()?;
        let num_models = cursor.read_u32::<LE>()?;
        let base = cursor.read_u32::<LE>()?;
        let model_index = cursor.read_u32::<LE>()?;

        let models: Vec<u32> = Vec::new();

        for i in 0..num_models {
            // read shit?
            cursor.seek(SeekFrom::Start(
                start_reading + model_index as u64 + (i * 0x94) as u64,
            ))?;
        }

        let name: String;
        cursor.seek(SeekFrom::Start(start_reading + name_index as u64))?;
        unsafe {
            let mut tmp_name: [u8; 260] = std::mem::zeroed();
            cursor.read_exact(&mut tmp_name)?;
            name = util::str_from_u8_nul_utf8_unchecked(&tmp_name).to_owned();
        }

        Ok(StudioBodyPart {
            start_pos: start_reading,
            name,
            base,
            models,
        })
    }
}
