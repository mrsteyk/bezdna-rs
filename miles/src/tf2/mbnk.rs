use std::io::{Read, Seek, SeekFrom};

use byteorder::{LE, ReadBytesExt};

use crate::{MilesError, util};

#[derive(Debug)]
pub struct Event {
    //pub name_off: u32,
    pub name: String,
    pub unk4: u32, // some offset of struct array of elem size 0x98?...
}

#[derive(Debug)]
pub struct MilesBank {
    // header shit
    pub unk8: u32,

    pub unk10: u32, // seek
   
    // pad shit
    // unk18: u64
    // unk20: u64
    // unk28: u64

    pub unk30: u32, // seek
    pub unk38: u32, // seek
    pub unk40: u32, // seek
    pub unk48: u32, // seek
    
    // pad shit
    // unk50: u64
    
    pub unk58: u32, // seek
    pub event_table_seek: u32, // seek, structs of size 8
    pub unk68: u32, // seek
    pub string_table_seek: u32, // seek
    pub unk78: u32, // seek
    pub unk80: u32, // seek
    pub unk88: u32, // seek
    pub unk90: u32, // seek

    pub unk98: u32,
    pub unk9C: u32,

    pub unkA0: u32,
    pub unkA4: u32,
    pub event_count: u32,
    pub unkAC: u32,

    pub events: Vec<Event>,
}

impl MilesBank {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, MilesError> {
        let header = cursor.read_u32::<LE>()?;
        if header != 0x43_42_4E_4B { // CBNK
            return Err(MilesError::InvalidHeader(header))
        }

        let version = cursor.read_u32::<LE>()?;
        if version != 0xD {
            return Err(MilesError::UnknownVersion(version))
        }

        let unk8 = cursor.read_u32::<LE>()?;

        let header = cursor.read_u32::<LE>()?;
        if header != 0x42_41_4E_4B { // BANK
            return Err(MilesError::InvalidHeader(header))
        }

        let unk10 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        
        cursor.read_u64::<LE>()?; // 0x18
        cursor.read_u64::<LE>()?; // 0x20
        cursor.read_u64::<LE>()?; // 0x28
        
        let unk30 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk38 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk40 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk48 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;

        cursor.read_u64::<LE>()?; // 0x50

        let unk58 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let event_table_seek = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32; // 0x60
        let unk68 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let string_table_seek = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32; // 0x70
        let unk78 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk80 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk88 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;
        let unk90 = (cursor.read_u64::<LE>()? & 0xFF_FF_FF_FF) as u32;

        let unk98 = cursor.read_u32::<LE>()?;
        let unk9C = cursor.read_u32::<LE>()?;

        let unkA0 = cursor.read_u32::<LE>()?;
        let unkA4 = cursor.read_u32::<LE>()?;
        let event_count = cursor.read_u32::<LE>()?;
        let unkAC = cursor.read_u32::<LE>()?;

        // ---
        let events = {
            let mut ret = Vec::<Event>::with_capacity(event_count as usize);

            for i in 0..event_count as u64 {
                cursor.seek(SeekFrom::Start(event_table_seek as u64 + i * 8))?;

                let name_off = cursor.read_u32::<LE>()?;
                let unk4 = cursor.read_u32::<LE>()?;

                cursor.seek(SeekFrom::Start(string_table_seek as u64 + name_off as u64))?;
                let name = util::string_from_buf(cursor);

                ret.push(Event {
                    name,
                    unk4,
                })
            }

            ret
        };
        
        Ok(Self {
            unk8,

            unk10,

            unk30,
            unk38,
            unk40,
            unk48,
            
            // pad shit
            // unk50: u64
            
            unk58,
            event_table_seek, 
            unk68, 
            string_table_seek, 
            unk78, 
            unk80, 
            unk88, 
            unk90,

            unk98,
            unk9C,

            unkA0,
            unkA4,
            event_count,
            unkAC,

            events,
        })
    }
}