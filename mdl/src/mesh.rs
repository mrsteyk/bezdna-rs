use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::util;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioMeshT {
    pub material: u32,
    pub model_index: i32,

    pub vertex_num: u32,
    pub vertex_offset: i32,

    pub flex_num: u32,
    pub flex_offset: i32,

    pub material_type: u32,
    pub material_param: u32,

    pub id: u32,

    pub centre: [f32; 3],

    pub vertex_data: u32,

    // skip to 0x34
    pub num_vertex_lod: [u32; 8],
    pub unk: [u32; 8],
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioMesh {
    start_pos: u64,

    pub material: u32,
    // idk if I need to know the model...
    pub vertex: (u32, i32),
    pub flex: (u32, i32),

    pub material_param: (u32, u32), // type, param

    pub id: u32,
    pub centre: [f32; 3],

    pub vertex_data: u32,
    pub num_vertex_lod: [u32; 8],
}

impl StudioMesh {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioMesh, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let material = cursor.read_u32::<LE>()?;
        // brih
        let _model = cursor.read_i32::<LE>()?;

        let vertex_num = cursor.read_u32::<LE>()?;
        let vertex_off = cursor.read_i32::<LE>()?;

        let flex_num = cursor.read_u32::<LE>()?;
        let flex_off = cursor.read_i32::<LE>()?;

        let material_type = cursor.read_u32::<LE>()?;
        let material_param = cursor.read_u32::<LE>()?;

        let id = cursor.read_u32::<LE>()?;

        let mut centre = [0f32; 3];
        cursor.read_f32_into::<LE>(&mut centre)?;

        let vertex_data = cursor.read_u32::<LE>()?;

        let mut num_vertex_lod = [0u32; 8];
        cursor.read_u32_into::<LE>(&mut num_vertex_lod)?;

        Ok(StudioMesh {
            start_pos: start_reading,

            material,

            vertex: (vertex_num, vertex_off),
            flex: (flex_num, flex_off),

            material_param: (material_type, material_param),

            id,
            centre,

            vertex_data,
            num_vertex_lod,
        })
    }
}
