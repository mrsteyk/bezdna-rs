use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use crate::{util::string_from_buf, FileEntry};

pub const TEXTURE_REFS: [&str; 14] = [
    "_col", "_nml", "_gls", "_spc", "_ilm", "UNK5", "UNK6", "UNK7", "_bm", // ???
    "UNK9", "UNK10", "_ao", "_cav", "_opa",
];
// UNK14 - base colour for a decal/?detail? ??? sometimes illumination???
// UNK15 - water normal on a fucking rock??? or rock jagged granite detail??? detail's normal without even UNK14 AND UNK16 sometimes???
// UNK16 - mask for a decal (apply _col to UNK14)
// UNK17 - distortion AO???
// UNK18 - distortion normal

// UNK18 = refraction (effects\cloud_refract_01.A716488B638D3F3F.txtr 256x256)
// UNK19 = refraction 2?

/*
// wat de fuq they r doin' ova dere
if UNK15 or UNK16:
    UNK14 = colour
else:
    UNK14 = illum
*/

pub const TEXTURE_REFS_SE2: [&str; 20] = [
    "_color",  // Material Editor 2
    "_normal", // Material Editor 2
    "_rough",  // S&Box wiki
    "_spc", // Like metallness but full rgb colour, didn't find it in MatEd2 even with complex shader, only a fucking slider
    "_ilm", "UNK5", "UNK6", "UNK7", "_bm", // ???
    "UNK9", "UNK10", "_ao",    // Material Editor 2
    "_cav",   // Metallicness?
    "_trans", // is trans correct? (based on the _opa meaning opacity for ?"decals"?) Quote: `transparency / translucency map`
    "_decal_col_ilm", // UNK14, temp
    "_decal_normal", // UNK15
    "_decal_mask", // UNK16
    "_mat_pp", // UNK17
    "_distort_normal", // UNK18
    "_distort_normal2", // UNK19
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
        Some(self.generic.get_data_off().unwrap())
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
