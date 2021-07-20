use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use crate::{util, MilesError};

#[derive(Debug)]
// or sample, idk how to call it...
// size - 0x58
pub struct Sound {
    //pub name_offset: u32,
    pub name: String,

    pub sample_rate: u16,
    // 6 here
    pub unk6: u16,

    pub channels: u8, // channels or some shit...
    // 9 here
    pub unk9: u8,
    pub unkA: u32,
    pub unkE: u32,
    pub unk12: u32,
    pub unk16: u16,
    pub unk18: u32,
    pub unk1C: u32,

    pub unk20: u32, // ...
    pub unk24: u32, // ...

    pub unk28: u32, // ...

    pub unk2C: u32,

    pub unk30: u32, // ...
    pub unk34: u32, // pad for pointer?
    // 0x38 here
    pub unk38: u32,
    pub unk3C: u32,

    pub unk40: u16, // ...
    pub unk42: u16, // ...

    pub unk44: u32,
    pub unk48: u32,
    pub unk4C: u32,
    pub unk50: u32,
    pub unk54: u32,
}

#[derive(Debug)]
pub struct Event {
    //pub name_off: u32,
    pub name: String,
    pub unk4: u32, // some offset of struct array of elem size 0x98?...
}

#[derive(Debug)]
pub struct MilesBank {
    // header shit
    pub file_size: u32,

    pub unk10: u32, // seek

    // pad shit
    // unk18: u64
    // unk20: u64
    // unk28: u64
    pub unk30: u32, // seek, structs size of 0x20, count unkA4, useless???
    pub unk38: u32, // seek, useless?
    pub unk40: u32, // seek, useless?
    pub unk48: u32, // seek, sounds?, 0x58

    // pad shit
    // unk50: u64
    pub unk58: u32,             // seek
    pub event_table_seek: u32,  // seek, structs of size 8
    pub unk68: u32,             // seek
    pub string_table_seek: u32, // seek
    pub unk78: u32,             // seek
    pub unk80: u32,             // seek
    pub unk88: u32,             // seek
    pub unk90: u32,             // seek

    pub unk98: u32,
    pub unk9C: u32,

    pub unkA0: u32, // count of unk48, 0x58
    pub unkA4: u32, // count of unk30, sinks???
    pub event_count: u32,
    pub unkAC: u32,

    pub unkB0: u32,
    pub unkB4: u32,
    pub unkB8: u32,
    pub tag: u32,

    pub events: Vec<Event>,
    pub sounds: Vec<Sound>,
}

impl MilesBank {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, MilesError> {
        let header = cursor.read_u32::<LE>()?;
        // CBNK
        if header != 0x43_42_4E_4B {
            return Err(MilesError::InvalidHeader(header));
        }

        let version = cursor.read_u32::<LE>()?;
        if version != 0xD {
            return Err(MilesError::UnknownVersion(version));
        }

        let file_size = cursor.read_u32::<LE>()?;

        let header = cursor.read_u32::<LE>()?;
        // BANK
        if header != 0x42_41_4E_4B {
            return Err(MilesError::InvalidHeader(header));
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
        assert_eq!(
            string_table_seek, unk10,
            "Охуеть, не встать - две разные стринг таблицы..."
        );
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

        let unkB0 = cursor.read_u32::<LE>()?;
        let unkB4 = cursor.read_u32::<LE>()?;
        let unkB8 = cursor.read_u32::<LE>()?;
        let tag = cursor.read_u32::<LE>()?;

        // ---
        let events = {
            let mut ret = Vec::<Event>::with_capacity(event_count as usize);

            for i in 0..event_count as u64 {
                cursor.seek(SeekFrom::Start(event_table_seek as u64 + i * 8))?;

                let name_off = cursor.read_u32::<LE>()?;
                let unk4 = cursor.read_u32::<LE>()?;

                cursor.seek(SeekFrom::Start(string_table_seek as u64 + name_off as u64))?;
                let name = util::string_from_buf(cursor);

                ret.push(Event { name, unk4 })
            }

            ret
        };

        let sounds = {
            let mut ret = Vec::<Sound>::with_capacity(unkA0 as usize);

            for i in 0..unkA0 as u64 {
                cursor.seek(SeekFrom::Start(unk48 as u64 + 0x10 + i * 0x58))?;

                //pub name_offset: u32,
                //pub name: String,
                let name_offset = cursor.read_u32::<LE>()?;

                ret.push(Sound {
                    sample_rate: cursor.read_u16::<LE>()?,
                    unk6: cursor.read_u16::<LE>()?,

                    channels: cursor.read_u8()?,
                    unk9: cursor.read_u8()?,
                    unkA: cursor.read_u32::<LE>()?,
                    unkE: cursor.read_u32::<LE>()?,
                    unk12: cursor.read_u32::<LE>()?,
                    unk16: cursor.read_u16::<LE>()?,
                    unk18: cursor.read_u32::<LE>()?,
                    unk1C: cursor.read_u32::<LE>()?,

                    unk20: cursor.read_u32::<LE>()?,
                    unk24: cursor.read_u32::<LE>()?,

                    unk28: cursor.read_u32::<LE>()?,

                    unk2C: cursor.read_u32::<LE>()?,

                    unk30: cursor.read_u32::<LE>()?,
                    unk34: cursor.read_u32::<LE>()?,
                    unk38: cursor.read_u32::<LE>()?,
                    unk3C: cursor.read_u32::<LE>()?,

                    unk40: cursor.read_u16::<LE>()?,
                    unk42: cursor.read_u16::<LE>()?,

                    unk44: cursor.read_u32::<LE>()?,
                    unk48: cursor.read_u32::<LE>()?,
                    unk4C: cursor.read_u32::<LE>()?,
                    unk50: cursor.read_u32::<LE>()?,
                    unk54: cursor.read_u32::<LE>()?,

                    name: {
                        cursor.seek(SeekFrom::Start(
                            string_table_seek as u64 + name_offset as u64,
                        ))?;
                        util::string_from_buf(cursor)
                    },
                })
            }

            ret
        };

        Ok(Self {
            file_size,

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

            unkB0,
            unkB4,
            unkB8,
            tag,

            events,
            sounds,
        })
    }
}
