use std::io::{Read, Seek};

use byteorder::{ReadBytesExt, LE};

use crate::studiohdr::DescType;

#[derive(Debug, PartialEq, Clone)]
pub struct VtxFile {
    start_pos: u64,

    pub version: u32,

    pub vertex_cache_size: u32,
    pub max_bones_per_strip: u16,
    pub max_bones_per_tri: u16,
    pub max_bones_per_vertex: u32,

    pub checksum: u32,

    pub lod_count: u32,

    pub material_replace_list_index: i32,

    pub body_part: DescType,
}

impl VtxFile {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<Self, std::io::Error> {
        let start_pos = cursor.stream_position()?;

        let version = cursor.read_u32::<LE>()?;
        assert_eq!(version, 7, "UNSUPPORTED VERSION");

        let vertex_cache_size = cursor.read_u32::<LE>()?;
        let max_bones_per_strip = cursor.read_u16::<LE>()?;
        let max_bones_per_tri = cursor.read_u16::<LE>()?;
        let max_bones_per_vertex = cursor.read_u32::<LE>()?;

        let checksum = cursor.read_u32::<LE>()?;

        let lod_count = cursor.read_u32::<LE>()?;

        let material_replace_list_index = cursor.read_i32::<LE>()?;

        let body_part_count = cursor.read_u32::<LE>()?;
        let body_part_index = cursor.read_i32::<LE>()?;

        assert_eq!(
            cursor.stream_position()? - start_pos,
            36,
            "VtxFile header missmatch"
        );

        Ok(Self {
            start_pos,

            version,

            vertex_cache_size,
            max_bones_per_strip,
            max_bones_per_tri,
            max_bones_per_vertex,

            checksum,

            lod_count,

            material_replace_list_index,

            body_part: (body_part_count, body_part_index),
        })
    }
}
