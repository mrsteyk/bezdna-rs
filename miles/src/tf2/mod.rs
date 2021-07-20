use std::io::{Read, Seek, SeekFrom};

use byteorder::{LE, ReadBytesExt};

use crate::MilesError;

pub mod mbnk;

#[derive(Debug)]
pub struct Controller { // 0x1C exactly...
    pub name_offset: u32, // string table(@0x10) offset
    pub unk4: f32,
    pub unk8: f32,
    pub unkC: f32,
    
    pub unk10: u8,
    pub unk11: u8,

    pub unk12: u16,
    
    // -- idk
    
    pub unk14: u32,
    
    pub unk18: u16,
    
    pub unk1a: u8,
    pub unk1b: u8,
}

#[derive(Debug)]
pub struct Unk20 { // total size 0xB0
    pub unk0: u32,
    pub unk4: u32, // or u8?

    pub unk8: u64,
    //0x10 here

    // ---

    pub unk24: f32,
    pub unk28: u16, // gets set to 0x70
    // 0x2A here

    pub unk40: u16,
    // 0x42 here

    pub unk50: f32, // db

    pub unk70: u16,
    // 0x72 here
    
    pub unk74: f32, // db
    pub unk78: f32, // db
    pub unk7c: f32,
    // 0x80 here

    pub unk8c: f32, // ampl of 0x78

    pub unk90: f32, // ampl of unk50, epxf(unk50 * 0.115129) - DbToAmpl
    pub unk94: u32, // gets set to unk98
    pub unk98: u32,
    // 0xA0 here

    pub unkAD: u8,

}

#[derive(Debug)]
pub struct MilesProject {
    // seeks, padded so you can do pointer replacement for easier access...
    pub controllers_seek: u32, // structs of size 0x1c
    pub string_table_offset: u32, // String table...
    pub unk18: u32,
    pub unk20: u32, // structs of size 0xB0
    pub unk28: u32,
    pub unk30: u32,
    pub unk38: u32,
    pub unk40: u32,
    pub unk48: u32,
    pub unk50: u32,
    pub unk58: u32,
    pub unk60: u32,
    pub unk68: u32, // has strings...
    pub unk70: u32, // has strings...
    pub unk78: u32,
    pub unk80: u32,

    // --- unk pads
    pub unk88: u32, // or 64???
    pub unk8c: u32,
    pub unk90: u32,

    // ---
    pub controllers_num: u32, // count of some sort (struct of size 56 aka 0x38)
    pub unk98: u32, // count of some sort (prolly unk20)

    // --- Parsed ---
    pub controllers_parsed: Vec<(Controller, String)>,
}

impl super::MilesProject for MilesProject {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_version(&self) -> crate::MilesVersion {
        crate::MilesVersion::TF2
    }
}

impl MilesProject {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, MilesError> {
        let header = cursor.read_u32::<LE>()?;
        if header != 0x43_50_52_4A {
            return Err(MilesError::InvalidHeader(header))
        }

        let version = cursor.read_u32::<LE>()?;
        if version != 0xD {
            return Err(MilesError::UnknownVersion(version))
        }

        let controllers_seek = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let string_table_offset = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk18 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk20 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk28 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk30 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk38 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk40 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk48 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk50 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk58 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk60 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk68 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk70 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk78 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk80 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;

        let unk88 = cursor.read_u32::<LE>()?;
        let unk8c = cursor.read_u32::<LE>()?;
        let unk90 = cursor.read_u32::<LE>()?;
        let controllers_num = cursor.read_u32::<LE>()?;
        let unk98 = cursor.read_u32::<LE>()?;
        // unk9c here...
        // probably up to 0xa8...

        // ---
        let controllers_parsed = {
            let mut ret = Vec::<(Controller, String)>::new();

            for i in 0..controllers_num {
                cursor.seek(SeekFrom::Start(controllers_seek as u64 + i as u64 * 0x1c))?;
                let unk8 = Controller {
                    name_offset: cursor.read_u32::<LE>()?,
                    unk4: cursor.read_f32::<LE>()?,
                    unk8: cursor.read_f32::<LE>()?,
                    unkC: cursor.read_f32::<LE>()?,
    
                    unk10: cursor.read_u8()?,
                    unk11: cursor.read_u8()?,

                    unk12: cursor.read_u16::<LE>()?,
                    
                    unk14: cursor.read_u32::<LE>()?,
                    
                    unk18: cursor.read_u16::<LE>()?,
                    
                    unk1a: cursor.read_u8()?,
                    unk1b: cursor.read_u8()?,
                };
                let string_seek = string_table_offset + unk8.name_offset;
                cursor.seek(SeekFrom::Start(string_seek as u64))?;
                let str = crate::util::string_from_buf(cursor);

                ret.push((unk8, str));
            }

            ret
        };

        Ok(Self {
            controllers_seek,
            string_table_offset,
            unk18,
            unk20,
            unk28,
            unk30,
            unk38,
            unk40,
            unk48,
            unk50,
            unk58,
            unk60,
            unk68,
            unk70,
            unk78,
            unk80,

            unk88,
            unk8c,
            unk90,
            controllers_num,
            unk98,

            controllers_parsed,
        })
    }
}