use std::{
    cell::RefCell,
    cmp::min,
    io::{Cursor, Read, Seek},
};

use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
use byteorder::{ReadBytesExt, LE};

use crate::{MilesError, transforms::ddct};

use crate::transforms;

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

// константы квантов которые равны 10**(i*0.066399999) с лютой погрешностью, но так декодит сам бинкавин
pub const QUANTS: [f32; 96] = [
    1.0,
    1.1651986837387085,
    1.357688069343567,
    1.5819764137268066,
    1.843316912651062,
    2.1478304862976074,
    2.5026493072509766,
    2.916083812713623,
    3.3978171348571777,
    3.959132194519043,
    4.613175868988037,
    5.3752665519714355,
    6.263253688812256,
    7.2979350090026855,
    8.503544807434082,
    9.908319473266602,
    11.545161247253418,
    13.452406883239746,
    15.674727439880371,
    18.264171600341797,
    21.281391143798828,
    24.797048568725586,
    28.893489837646484,
    33.666656494140625,
    39.22834777832031,
    45.70882034301758,
    53.259857177734375,
    62.058319091796875,
    72.31027221679688,
    84.2558364868164,
    98.17479705810547,
    114.39314270019531,
    133.29074096679688,
    155.31021118164062,
    180.96725463867188,
    210.86280822753906,
    245.69708251953125,
    286.2859191894531,
    333.5799865722656,
    388.6869812011719,
    452.8975830078125,
    527.7156982421875,
    614.8936157226562,
    716.4732666015625,
    834.833740234375,
    972.7472534179688,
    1133.44384765625,
    1320.687255859375,
    1538.8631591796875,
    1793.0814208984375,
    2089.296142578125,
    2434.445068359375,
    2836.6123046875,
    3305.21728515625,
    3851.23486328125,
    4487.4541015625,
    5228.775390625,
    6092.5625,
    7099.04638671875,
    8271.7998046875,
    9638.2900390625,
    11230.5234375,
    13085.7919921875,
    15247.5478515625,
    17766.423828125,
    20701.4140625,
    24121.259765625,
    28106.0625,
    32749.1484375,
    38159.265625,
    44463.125,
    51808.37890625,
    60367.0546875,
    70339.6171875,
    81959.6328125,
    95499.2578125,
    111275.6171875,
    129658.203125,
    151077.578125,
    176035.390625,
    205116.21875,
    239001.15625,
    278483.84375,
    324489.03125,
    378094.1875,
    440554.875,
    513333.96875,
    598136.0625,
    696947.375,
    812082.1875,
    946237.1875,
    1102554.375,
    1284694.875,
    1496924.875,
    1744214.875,
    2032357.0,
];

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
    pub root: f32, // +12 aka +0xC
}

impl BinkA {
    //pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, MilesError> {
    pub fn read(slice: &[u8]) -> Result<Self, MilesError> {
        let mut cursor = Cursor::new(slice);

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
        // v9 never appears to be used?
        let (root, _v9, frame_len) = if sample_rate < 44100 {
            if sample_rate < 22050 {
                (0.0883883461356f32, 0.0625f32, 512u32)
            } else {
                (0.0625f32, 0.0441941730678f32, 1024u32)
            }
        } else {
            (0.0441941730678f32, 0.03125f32, 2048u32)
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

        if channels_half_ceil > 1 {
            todo!("{} > 1", channels_half_ceil)
        }

        let mut buf = Vec::<u8>::new();
        cursor.read_to_end(&mut buf)?;
        let bitbuf = BitReadBuffer::new(buf.as_slice(), LittleEndian);
        let mut bitstream = BitReadStream::new(bitbuf);
        if bitstream.read_int::<u16>(16).unwrap() != 0x99_99 {
            panic!("Brih")
        }

        let read_float = |bitstream: &mut BitReadStream<LittleEndian>| {
            let power: u32 = bitstream.read_int(5).unwrap();
            let x = bitstream.read_int::<u32>(23).unwrap();
            let f = (x as f32) * (2f32.powi(power as i32 - 23));
            if bitstream.read_bool().unwrap() {
                return -f;
            } else {
                return f;
            }
        };

        bitstream.read_int::<u16>(2).unwrap(); // алгоритмы сжатия блять...

        // this probably means that some channels are read like stereo and others are read like normal???
        for ii in 0..channels_half_ceil {
            let channels = v53[ii as usize];
            let mut coeffs = [0f64; 2048];
            let mut quants = [0f32; 25];

            let mut ddct_w = [0f64; 2560];
            let mut fft4g_ip = [0i32; 48];

            for i in 0..channels {
                let bands = &bands[(ii + i) as usize];

                let mut q = 0f32;
                coeffs[0] = read_float(&mut bitstream) as f64;
                coeffs[1] = read_float(&mut bitstream) as f64;

                for i in 0..num_bands as usize {
                    let val: usize = bitstream.read_int(8).unwrap();
                    quants[i] = QUANTS[min(val, 95)];
                }

                let mut k = 0;
                for j in 0..num_bands {
                    if bands[j as usize] * 2 < 2 {
                        k = j;
                        q = quants[k as usize];
                    } else {
                        break;
                    }
                }

                let mut j = 2;
                while j < frame_len {
                    let mut jj = if bitstream.read_bool().unwrap() {
                        // RLE
                        j + RLE_LENGTH_TABLE[bitstream.read_int::<usize>(4).unwrap()] * 8
                    } else {
                        // NO RLE
                        j + 8
                    };

                    if jj > frame_len {
                        jj = frame_len
                    }

                    let width: u32 = bitstream.read_int(4).unwrap();
                    println!(
                        "jj {} | {} | {} | {} {}",
                        jj, width, j, coeffs[0], coeffs[1]
                    );
                    if width == 0 {
                        j = jj;
                        // no need to zero coeffs???
                        //todo!();
                        while bands[k as usize] * 2 < j {
                            q = quants[k as usize];
                            k += 1;
                        }
                    } else {
                        while j < jj {
                            if bands[k as usize] * 2 == i {
                                q = quants[k as usize];
                                k += 1;
                            }
                            let coeff: u32 = bitstream.read_int(width as usize).unwrap();
                            if coeff != 0 {
                                if bitstream.read_bool().unwrap() {
                                    coeffs[j as usize] = -q as f64 * coeff as f64;
                                } else {
                                    coeffs[j as usize] = q as f64 * coeff as f64;
                                }
                            } else {
                                coeffs[j as usize] = 0f64;
                            }

                            j += 1;
                        }
                    }
                }

                // судя по тому что мне пришлось трэшнуть 2 бита - это дцт, а там пиздец...
                ddct(
                    frame_len as i32,
                    1,
                    coeffs.as_mut(),
                    fft4g_ip.as_mut(),
                    ddct_w.as_mut(),
                );

                for c in coeffs.iter_mut() {
                    *c = *c * root as f64;
                }

                println!("{:?} {} {}", &coeffs[..frame_len as usize], frame_len, i);
            }
        }

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
            root,
        })
    }
}
