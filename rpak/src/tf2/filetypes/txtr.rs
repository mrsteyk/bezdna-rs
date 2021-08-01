use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

pub use crate::apex::filetypes::txtr::{MipMap, MipMapType};
use crate::{util::string_from_buf, FileEntry};

const TEXTURE_SKIPS: [(u32, u32, u32); 64] = [
    (8, 4, 4),
    (8, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (8, 4, 4),
    (8, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 4, 4),
    (16, 1, 1),
    (16, 1, 1),
    (16, 1, 1),
    (12, 1, 1),
    (12, 1, 1),
    (12, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (8, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (2, 1, 1),
    (1, 1, 1),
    (1, 1, 1),
    (1, 1, 1),
    (1, 1, 1),
    (1, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (4, 1, 1),
    (2, 1, 1),
    (16, 4, 4),
    (16, 5, 4),
];

pub const TEXTURE_ALGOS: [&str; 64] = [
    "DXT1",    // 0
    "DXT1",    // 1
    "UNKNOWN", // 2
    "UNKNOWN", // 3
    "UNKNOWN", // 4
    "UNKNOWN", // 5
    "BC4U",    // 6
    "UNKNOWN", // 7
    "BC5U",    // 8
    "UNKNOWN", // 9
    "BC6H",    // 10 // DDS DX10?
    "UNKNOWN", // 11
    "UNKNOWN", // 12
    "BC7U",    // 13 // DDS DX10 0x62
    "UNKNOWN", // 14
    "UNKNOWN", // 15
    "UNKNOWN", // 16
    "UNKNOWN", // 17
    "UNKNOWN", // 18
    "UNKNOWN", // 19
    "UNKNOWN", // 20
    "UNKNOWN", // 21
    "UNKNOWN", // 22
    "UNKNOWN", // 23
    "UNKNOWN", // 24
    "UNKNOWN", // 25
    "UNKNOWN", // 26
    "UNKNOWN", // 27
    "UNKNOWN", // 28
    "UNKNOWN", // 29
    "UNKNOWN", // 30
    "UNKNOWN", // 31
    "UNKNOWN", // 32
    "UNKNOWN", // 33
    "UNKNOWN", // 34
    "UNKNOWN", // 35
    "UNKNOWN", // 36
    "UNKNOWN", // 37
    "UNKNOWN", // 38
    "UNKNOWN", // 39
    "UNKNOWN", // 40
    "UNKNOWN", // 41
    "UNKNOWN", // 42
    "UNKNOWN", // 43
    "UNKNOWN", // 44 // ??? no fourcc; DDPF_ALPHAPIXELS | DDPF_LUMINANCE
    "UNKNOWN", // 45
    "UNKNOWN", // 46
    "UNKNOWN", // 47
    "UNKNOWN", // 48
    "UNKNOWN", // 49
    "UNKNOWN", // 50
    "UNKNOWN", // 51
    "UNKNOWN", // 52
    "UNKNOWN", // 53
    "UNKNOWN", // 54
    "UNKNOWN", // 55
    "UNKNOWN", // 56
    "UNKNOWN", // 57
    "UNKNOWN", // 58
    "UNKNOWN", // 59
    "UNKNOWN", // 60
    "UNKNOWN", // 61
    "UNKNOWN", // 62
    "UNKNOWN", // 63
];

#[derive(Debug)]
pub struct Texture {
    pub generic: super::FileGeneric,

    pub guid: u64,
    pub name: String,
    pub width: u16,
    pub height: u16,

    pub texture_type: u16,
    //pub layers_count: u8, // TODO
    pub mipmaps: Vec<MipMap>,
    //pub total_size: u32, // TODO
}

impl crate::FileEntry for Texture {
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
        self.generic.get_star_off()
    }
    fn get_star_opt_off(&self) -> Option<u64> {
        None // we know for sure
    }

    fn get_ext(&self) -> &str {
        "txtr"
    }
}

impl Texture {
    pub fn ctor<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
        seeks: &[u64],
        generic: super::FileGeneric,
    ) -> Result<Self, std::io::Error> {
        cursor.seek(SeekFrom::Start(generic.get_desc_off()))?;

        let guid = cursor.read_u64::<LE>()?;
        assert_eq!(generic.get_guid(), guid, "пакер еблан");

        let name_seek = {
            let id = cursor.read_u32::<LE>()?;
            let off = cursor.read_u32::<LE>()?;
            seeks[id as usize] + off as u64
        };

        let width = cursor.read_u16::<LE>()?;
        let height = cursor.read_u16::<LE>()?;

        let _unk14 = cursor.read_u16::<LE>()?;

        let texture_type = cursor.read_u16::<LE>()?;

        let _unk18 = cursor.read_u64::<LE>()?;
        let _unk20 = cursor.read_u8()?;

        let rpak_mipmaps_num = cursor.read_u8()?;
        let starpak_mipmaps_num = cursor.read_u8()?;

        let mipmaps_num = (starpak_mipmaps_num as u32) + (rpak_mipmaps_num as u32);

        //let mut total_size_check = 0u64;

        let mipmaps = {
            // leftover logic from c+p
            let unk1e = 1; // TODO: //if layers_count == 0 { 1 } else { layers_count };

            let mut rpak_off = generic.get_data_off().unwrap(); // or panic...
            let mut starpak_off = generic.get_star_off().unwrap_or(0u64);

            let mut ret = Vec::<MipMap>::with_capacity(mipmaps_num as usize);
            //for i in mipmaps_num..=0 {
            for i in (0..mipmaps_num).rev() {
                let typ = if i < (starpak_mipmaps_num as u32) {
                    MipMapType::StarPak
                } else {
                    MipMapType::RPak
                };

                let (v15, v14, v16) = TEXTURE_SKIPS[texture_type as usize];

                let v17 = if (width >> i) > 1 { width >> i } else { 1 };
                let v22 = if (height >> i) > 1 { height >> i } else { 1 };

                let v21 = (v14 + v17 as u32 - 1) as u32 / v14;
                let v23 = v21 * ((v16 + v22 as u32 - 1) / v16);
                let v25 = v15 * v23;

                let size = ((v25 + 15) & 0xFFFFFFF0) as u64;
                let skip_size = unk1e as u64 * size;
                //let skip_size = size;

                let off = match typ {
                    MipMapType::RPak => {
                        let ret = rpak_off;
                        rpak_off += skip_size;
                        ret
                    }
                    MipMapType::StarPak => {
                        let ret = starpak_off;
                        starpak_off += skip_size;
                        ret
                    }
                    _ => {
                        panic!("TF2 WAT?")
                    }
                };

                //total_size_check += skip_size;

                ret.push(MipMap {
                    typ,
                    off,
                    width: v17,
                    height: v22,
                    size,
                });
            }
            ret
        };

        cursor.seek(SeekFrom::Start(name_seek))?;
        let name = string_from_buf(cursor);

        Ok(Self {
            generic,

            guid,
            name,

            width,
            height,

            texture_type,
            //layers_count: , // TODO
            mipmaps,
        })
    }
}
