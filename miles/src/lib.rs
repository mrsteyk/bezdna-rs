use std::{
    any::Any,
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
};

use byteorder::{ReadBytesExt, LE};

mod util;

pub mod tf2;

#[derive(Debug)]
pub enum MilesError {
    InvalidHeader(u32),
    UnknownVersion(u32),
    IOError(std::io::Error),
}

#[derive(Debug)]
pub enum MilesVersion {
    TF2,  // 0xD
    Apex, // 0x28...
}

impl From<std::io::Error> for MilesError {
    fn from(item: std::io::Error) -> Self {
        Self::IOError(item)
    }
}

pub fn get_project_version<R: Read + Seek + ReadBytesExt>(
    cursor: &mut R,
) -> Result<u32, MilesError> {
    cursor.seek(SeekFrom::Start(0))?;

    let header = cursor.read_u32::<LE>()?;
    if header != 0x43_50_52_4A {
        return Err(MilesError::InvalidHeader(header));
    }

    let version = cursor.read_u32::<LE>()?;
    if version != 0xD {
        return Err(MilesError::UnknownVersion(version));
    }

    Ok(version)
}

pub trait MilesProject: Debug {
    fn as_any(&self) -> &dyn Any;
    fn get_version(&self) -> MilesVersion;
}
