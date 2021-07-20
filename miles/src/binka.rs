use std::{
    cell::RefCell,
    io::{Read, Seek},
};

use byteorder::{ReadBytesExt, LE};

use crate::MilesError;

// hdr: u32 header is "1FCB"
// unk4: u8 idk MUST BE NOT GREATER THAN 2, version mb?
// \t unk4 == 2 -> (v7: u16, v6: u16, 0x14 as u32) = 0x14, 0x16, 0x14
// unk5: u8 idk // v9
// sr: u16 followed by sample rate, used to determine chunk size
// \t???(v10) is computed like so:
// \t\t sr < 22050 -> unk5<<10 (usually 1024)
// \t\t sr < 44100 -> unk5<<11 (usually 2048)
// \t\t ELSE -> unk5<<12 (usually 4096)
// \tFrame length???(v20) is computed like so:
// \t\t sr < 22050 -> 512
// \t\t sr < 44100 -> 1024
// \t\t ELSE -> 2048

// Не такая же как и в ВМА, пиздёж повсюду блять
/// Used to calculate number of bands alongside some other stuff
pub const CRITICAL_FREQS: [u32; 25] = [
    0, 100, 200, 300, 400, 510, 630, 770, 920, 1080, 1270, 1480, 1720, 2000, 2320, 2700, 3150,
    3700, 4400, 5300, 6400, 7700, 9500, 12000, 15500,
];

// Эта хуйня была придумана укропами блять...
// 1 байт если да - 4 байта читай и смотри сюда крч | одна из частей декода ебучего блока
/// RLE sample length
pub const RLE_LENGTH_TABLE: [u32; 16] = [2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 32, 64];

#[derive(Debug)]
pub struct Header {
    pub hdr: u32,           // 0x0 - 0x42_43_46_31 'BCF1'
    pub ver: u8,            // 0x4 - ???
    pub channels: u8,       // 0x5 - confirmed based on 2 files...
    pub sample_rate: u16,   // 0x6
    pub samples_count: u32, // 0x8
    pub unkC: u32,          // 0xC

    pub unk10: u32, // 0x10 same as 0x18 in mbnk

    // depends on ver, I will only do 2 I guess...
    pub unk14: u16, // 0x14 - ???, dword in ver != 2
    pub unk16: u16, // 0x16 - ???
}

#[derive(Debug)]
pub struct BinkA {
    start_pos: u64, // allow streaming shit...

    pub hdr: Header,

    pub frame_len: u32,

    /// All bands for all channels
    pub bands: RefCell<Vec<Vec<u32>>>,

    pub unk14_arr: Vec<u16>,

    pub v53: Vec<u32>,
}

impl BinkA {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, MilesError> {
        let start_pos = cursor.stream_position().unwrap();

        let hdr = cursor.read_u32::<LE>()?;
        if hdr != 0x42_43_46_31 {
            return Err(MilesError::InvalidHeader(hdr));
        }

        let ver = cursor.read_u8()?;
        if ver > 2 {
            return Err(MilesError::UnknownVersion(ver as u32));
        }

        if ver != 2 {
            todo!("Ver is 1 or 0? {}", ver)
        }

        let channels = cursor.read_u8()?;
        let sample_rate = cursor.read_u16::<LE>()?;
        let samples_count = cursor.read_u32::<LE>()?;
        let unkC = cursor.read_u32::<LE>()?;

        let unk10 = cursor.read_u32::<LE>()?;

        let unk14 = cursor.read_u16::<LE>()?;
        let unk16 = cursor.read_u16::<LE>()?;

        if unk14 > 0x80 {
            todo!("{} > 0x80", unk14)
        }

        let mut unk14_arr = Vec::<u16>::with_capacity(unk14 as usize);
        for _ in 0..unk14 {
            unk14_arr.push(cursor.read_u16::<LE>()?);
        }

        let channels_half_ceil = (channels as u32 + 1) >> 1;
        let v53 = {
            let mut ret = Vec::<u32>::with_capacity(channels_half_ceil as usize);
            for i in 0..channels_half_ceil {
                let sub = if (channels as u32 - i) < 2 {
                    1u32
                } else {
                    0u32
                };

                ret.push(2 - sub);
            }

            ret
        };

        let half_sr = (sample_rate + 1) >> 1;
        let frame_len = if sample_rate < 44100 {
            if sample_rate < 22050 {
                512u32
            } else {
                1024
            }
        } else {
            2048
        };
        let half_frame_len = frame_len >> 1;
        let num_bands = {
            let mut ret = 25u32;
            for i in 0..CRITICAL_FREQS.len() {
                if CRITICAL_FREQS[i] >= half_sr as u32 {
                    ret = i as u32;
                    break;
                }
            }

            ret
        };
        let bands = {
            let mut ret = Vec::<Vec<u32>>::with_capacity(channels_half_ceil as usize);

            for _ in 0..channels_half_ceil {
                let mut bands = Vec::<u32>::with_capacity(num_bands as usize + 1);
                bands.push(1);
                for i in 1..num_bands {
                    bands.push(CRITICAL_FREQS[i as usize - 1] * half_frame_len / half_sr as u32);
                }
                bands.push(half_frame_len);

                ret.push(bands);
            }
            
            ret
        };

        Ok(Self {
            start_pos,

            hdr: Header {
                hdr,
                ver,

                channels,
                sample_rate,
                samples_count,
                unkC,

                unk10,

                unk14,
                unk16,
            },

            frame_len,
            bands: RefCell::new(bands),

            unk14_arr,
            v53,
        })
    }
}
