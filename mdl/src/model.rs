use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::{util, StudioMesh};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioModelT {
    pub name: [u8; 64],
    pub typee: u32,
    pub bounding_radius: f32,

    pub num_meshes: u32,
    pub meshes_index: i32,

    pub vertex_index: i32,
    pub tangent_index: i32,

    pub numattachment: u32,
    pub attachment_index: i32,

    pub numeyeball: u32,
    pub eyeball_index: i32,

    pub vertex_data: u32,
    pub tangent_data: u32,

    // 0x74
    pad: [u32; 2],

    // skip to 0x7c
    pub v6: i32, // 0x7c
    pub v7: i32, // 0x80

    // 0x84
    pad1: [u32; 4],
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioModel {
    start_pos: u64,

    pub name: String,
    pub typ: u32,
    pub bounding_radius: f32,

    pub meshes: Vec<StudioMesh>,
    pub eyeballs: Vec<()>,

    pub vertex_data: u32,
    pub tangent_data: u32,

    pub v6: u32,
    pub v7: u32,
}

impl StudioModel {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioModel, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let mut name_raw = [0u8; 64];
        cursor.read_exact(&mut name_raw)?;
        let name = unsafe { util::str_from_u8_nul_utf8_unchecked(&name_raw).to_owned() };

        let typ = cursor.read_u32::<LE>()?;
        let bounding_radius = cursor.read_f32::<LE>()?;

        let mesh_num = cursor.read_u32::<LE>()?;
        let mesh_index = cursor.read_i32::<LE>()?;

        let _vertex_num = cursor.read_u32::<LE>()?;
        let _vertex_index = cursor.read_i32::<LE>()?;
        let _tanget_index = cursor.read_i32::<LE>()?;

        let _eyeball_num = cursor.read_u32::<LE>()?;
        let _eyeball_index = cursor.read_i32::<LE>()?;

        let vertex_data = cursor.read_u32::<LE>()?;
        let tangent_data = cursor.read_u32::<LE>()?;

        cursor.seek(SeekFrom::Start(start_reading + 0x7C))?;
        let v6 = cursor.read_u32::<LE>()?;
        let v7 = cursor.read_u32::<LE>()?;

        let mut meshes: Vec<StudioMesh> = Vec::new();
        for i in 0..mesh_num {
            cursor.seek(SeekFrom::Start(
                start_reading + mesh_index as u64 + (0x74 * i) as u64,
            ))?;
            meshes.push(StudioMesh::read(cursor)?);
        }

        // Eyeballs are the bane of my existence...

        Ok(StudioModel {
            start_pos: start_reading,

            name,
            typ,
            bounding_radius,

            meshes,
            eyeballs: Vec::new(),

            vertex_data,
            tangent_data,

            v6,
            v7,
        })
    }
}
