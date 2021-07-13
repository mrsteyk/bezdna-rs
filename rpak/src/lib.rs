#[macro_use]
extern crate derivative;

mod binding;

mod util;

mod consts;
use std::{
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom},
    rc::Rc,
};

use byteorder::{ReadBytesExt, LE};
pub use consts::*;

mod hashing;
pub use hashing::hash;

mod decomp;

pub mod apex;
pub mod tf2;

// dynamic shit per ext
/// This trait represents what every file in the game should have and I know the meaning of it and it's also very useful
pub trait FileEntry: Debug {
    // getters
    /// Internal name hash used by the lookup functions
    fn get_guid(&self) -> u64;
    /// File's extension (no longer than 4 characters)
    fn get_ext(&self) -> String;
    /// Implemented per file extension, not all types have it
    fn get_name(&self) -> Option<String>; // ergh?
    /// Offset from the start of the file of so called description.
    /// Every file should have this field
    fn get_desc_off(&self) -> u64;
    /// Predicted unaligned size of so called description
    fn get_desc_size(&self) -> usize;
    // mb easier handling??? TODO: is this better?
    /// Offset of data associated with this file, not every file has it
    fn get_data_off(&self) -> Option<u64>;
    /// StarPak offset of file's data. Not all files have it.
    fn get_star_off(&self) -> Option<u64>;
    /// Apex specific optional StarPak offset. Not all files have it.
    fn get_star_opt_off(&self) -> Option<u64>; // TF2 won't implement this rather...
}

// this is static across all(2) games
/// Section Descriptor of RPak
#[derive(Debug)]
pub struct SectionDesc {
    /// Weird section type, use `&0b111` to find out real type?
    pub section_type: u32,
    /// Align byte that sometimes gets used
    pub align_byte: u32,
    /// Unaligned(?) sum of sizes of all data chunk sharing the type of `section_type`
    pub size_unaligned: u64,
}

impl SectionDesc {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, std::io::Error> {
        Ok(Self {
            section_type: cursor.read_u32::<LE>()?,
            align_byte: cursor.read_u32::<LE>()?,
            size_unaligned: cursor.read_u64::<LE>()?,
        })
    }

    /// Parses an array of SectionDesc's of known size from a good-enough buffer
    ///
    /// # Arguments
    ///
    /// * `cursor` - A buffer which implements Read, Seek, ReadBytesExt
    /// * `size` - Known size of the array, u16 is the game's limitation @ the moment...
    pub fn parse<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
        size: u16,
    ) -> Result<Vec<Self>, std::io::Error> {
        let mut ret = Vec::with_capacity(size as usize);
        for _ in 0..size {
            ret.push(Self::read(cursor)?);
        }

        Ok(ret)
    }
}

/// This trait represents generic workflow with all RPakFiles
pub trait RPakFile: Debug {
    fn is_compressed(&self) -> bool;
    fn should_lla(&self) -> bool;

    fn get_decompressed(&self) -> &Cursor<Vec<u8>>;

    fn get_version(&self) -> RPakVersion;
    fn get_sections_desc(&self) -> Vec<SectionDesc>;
    fn get_files(&self) -> Vec<Rc<dyn FileEntry>>;
}

/// Takes rpak file and parses it into a viable format
pub fn parse_rpak<R: Read + Seek + ReadBytesExt>(
    cursor: &mut R,
) -> Result<Box<dyn RPakFile>, RPakError> {
    match get_rpak_version_cursor(cursor) {
        RPakVersion::TF2 => {
            todo!()
        }
        RPakVersion::APEX => match apex::RPakFile::read(cursor) {
            Ok(file) => Ok(Box::new(file)),
            Err(err) => Err(err),
        },
        ver => Err(RPakError::InvalidVersion(ver as u16)),
    }
}

/// Quick parses the header for the known RPak version
pub fn get_rpak_version(file: Vec<u8>) -> RPakVersion {
    if file.len() < 88 {
        RPakVersion::Invalid
    } else {
        match file[4] as u16 + ((file[5] as u16) << 8) {
            7 => RPakVersion::TF2,
            8 => RPakVersion::APEX,
            _ => RPakVersion::Invalid,
        }
    }
}

/// Quick parses the header of the buffer for the known RPak version
pub fn get_rpak_version_cursor<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> RPakVersion {
    match cursor.seek(SeekFrom::Start(4)) {
        Ok(_) => match cursor.read_u16::<LE>() {
            Ok(v) => match v {
                7 => RPakVersion::TF2,
                8 => RPakVersion::APEX,
                _ => RPakVersion::Invalid,
            },
            _ => RPakVersion::Invalid,
        },
        _ => RPakVersion::Invalid,
    }
}
