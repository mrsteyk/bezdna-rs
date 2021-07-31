use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use crate::{util::string_from_buf, FileEntry};

pub const TEXTURE_REFS: [&str; 14] = [
    "_col", "_nml", "_gls", "_spc", "_ilm", "UNK5", "UNK6", "UNK7", "_bm", // ???
    "UNK9", "UNK10", "_ao", "_cav", "_opa",
];

#[derive(Debug)]
pub struct Material {
    pub generic: super::FileGeneric,

    pub guid: u64,
    pub name: String,
    pub surface_props: String,

    pub texture_guids: Vec<u64>,
}

impl crate::FileEntry for Material {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_guid(&self) -> u64 {
        self.generic.get_guid()
    }

    fn get_desc_off(&self) -> u64 {
        self.generic.get_desc_off()
    }
    fn get_data_off(&self) -> Option<u64> {
        assert_eq!(self.generic.get_data_off(), None);
        None
    }
    fn get_desc_size(&self) -> usize {
        self.generic.get_desc_size()
    }

    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn get_star_off(&self) -> Option<u64> {
        assert_eq!(self.generic.get_star_off(), None);
        None
    }
    fn get_star_opt_off(&self) -> Option<u64> {
        None // we know for sure
    }

    fn get_ext(&self) -> &str {
        "matl"
    }
}

impl Material {
    pub fn ctor<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
        seeks: &[u64],
        generic: super::FileGeneric,
    ) -> Result<Self, std::io::Error> {
        cursor.seek(SeekFrom::Start(generic.get_desc_off()))?;

        let _unk0 = cursor.read_u64::<LE>()?;
        assert_eq!(_unk0, 0, "pad0 isn't 0!");
        let _unk8 = cursor.read_u64::<LE>()?;
        assert_eq!(_unk8, 0, "pad8 isn't 0!");

        let guid = cursor.read_u64::<LE>()?;

        let name_seek = {
            let id = cursor.read_u32::<LE>()?;
            let off = cursor.read_u32::<LE>()?;
            seeks[id as usize] + off as u64
        };

        let mat_name_seek = {
            let id = cursor.read_u32::<LE>()?;
            let off = cursor.read_u32::<LE>()?;
            seeks[id as usize] + off as u64
        };

        cursor.seek(SeekFrom::Start(name_seek))?;
        let name = string_from_buf(cursor);
        cursor.seek(SeekFrom::Start(mat_name_seek))?;
        let surface_props = string_from_buf(cursor);

        cursor.seek(SeekFrom::Start(generic.get_desc_off() + 0x98))?;

        let unk98_seek = {
            let id = cursor.read_u32::<LE>()?;
            let off = cursor.read_u32::<LE>()?;
            seeks[id as usize] + off as u64
        };

        let unkA0_seek = {
            let id = cursor.read_u32::<LE>()?;
            let off = cursor.read_u32::<LE>()?;
            seeks[id as usize] + off as u64
        };

        let refcnt = (unkA0_seek - unk98_seek) / 8;

        let texture_guids = {
            let mut ret = Vec::<u64>::with_capacity(refcnt as usize);

            cursor.seek(SeekFrom::Start(unk98_seek))?;

            for _ in 0..refcnt {
                ret.push(cursor.read_u64::<LE>()?);
            }

            ret
        };

        Ok(Self {
            generic,

            guid,
            name,
            surface_props,

            texture_guids,
        })
    }
}
