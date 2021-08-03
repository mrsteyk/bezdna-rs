use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::util;

use crate::se::*;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioBoneT {
    // I don't recall the actual name...
    pub sznameindex: i32, // 0x0
    pub parent: i32,      // 0x4, -1 - none

    pub bone_controller: [i32; 6], // 0x8, -1 - none

    pub pos: [f32; 3],  // 0x20
    pub quat: [f32; 4], // 0x2c
    pub rot: [f32; 3],  // 0x3c

    pub pos_scale: [f32; 3], // 0x48
    pub rot_scale: [f32; 3], // 0x54
    // ???
    pub pos2bone: [[f32; 3]; 4], // 0x60

    pub quat_align: [f32; 4], // 0x90

    pub flags: u32, // 0xA0

    pub procedural_rule_type: i32,  // 0xA4
    pub procedural_rule_index: i32, // 0xA8

    pub physics_bone_index: i32, // 0xAC

    pub surface_prop_name_index: i32, // 0xB0

    pub contents: i32, // 0xB4

    pub idk: [i32; 15],
    // size MUST be 0xf4 if to trust myself...
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProceduralRule {
    None,
    AxisInterp, // 1 TODO
    QuatInterp, // 2 TODO
    AimAtBone,  // 3 TODO
    AimAttach,  // 4 TODO
    Jiggle,     // 5 TODO
}

#[derive(Debug, PartialEq, Clone)]
pub struct StudioBone {
    start_pos: u64,

    pub name: String,
    pub parent: i32, // index makes sense for exporting ngl

    pub bone_controller: [i32; 6],

    pub pos: Vec3,
    pub quat: Quat,
    pub rot: Vec3,

    pub pos_scale: Vec3,
    pub rot_scale: Vec3,

    pub pos2bone: [[f32; 3]; 4],

    pub quat_align: Quat,

    pub flags: u32,

    pub procedural_rule: ProceduralRule,
    pub procedural_rule_index: i32,

    pub physics_bone_index: i32, // ???

    pub surface_prop_name: String,

    pub contents: i32,

    pub idk: [i32; 15],
}

impl StudioBone {
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioBone, std::io::Error> {
        let start_reading = cursor.stream_position()?;

        let bone_name_index = cursor.read_i32::<LE>()?;

        let parent = cursor.read_i32::<LE>()?;

        let mut bone_controller = [0i32; 6];
        cursor.read_i32_into::<LE>(&mut bone_controller)?;

        let mut pos = [0f32; 3];
        cursor.read_f32_into::<LE>(&mut pos)?;
        let mut quat = [0f32; 4];
        cursor.read_f32_into::<LE>(&mut quat)?;
        let mut rot = [0f32; 3];
        cursor.read_f32_into::<LE>(&mut rot)?;

        let mut pos_scale = [0f32; 3];
        cursor.read_f32_into::<LE>(&mut pos_scale)?;
        let mut rot_scale = [0f32; 3];
        cursor.read_f32_into::<LE>(&mut rot_scale)?;

        let pos2bone = {
            // or whatever the fuck it's called
            let mut row0 = [0f32; 3];
            let mut row1 = [0f32; 3];
            let mut row2 = [0f32; 3];
            let mut row3 = [0f32; 3];
            cursor.read_f32_into::<LE>(&mut row0)?;
            cursor.read_f32_into::<LE>(&mut row1)?;
            cursor.read_f32_into::<LE>(&mut row2)?;
            cursor.read_f32_into::<LE>(&mut row3)?;

            // or re-arrange?
            [row0, row1, row2, row3]
        };

        let mut quat_align = [0f32; 4];
        cursor.read_f32_into::<LE>(&mut quat_align)?;

        let flags = cursor.read_u32::<LE>()?;

        let procedural_rule_type = cursor.read_i32::<LE>()?;
        let procedural_rule_index = cursor.read_i32::<LE>()?;
        let procedural_rule = match procedural_rule_type {
            0 => ProceduralRule::None,
            1 => ProceduralRule::AxisInterp,
            2 => ProceduralRule::QuatInterp,
            3 => ProceduralRule::AimAtBone,
            4 => ProceduralRule::AimAttach,
            5 => ProceduralRule::Jiggle,
            v => panic!("Invalid procedural type {}", v),
        };

        let physics_bone_index = cursor.read_i32::<LE>()?;

        let surface_prop_name_index = cursor.read_i32::<LE>()?;

        let contents = cursor.read_i32::<LE>()?;

        let mut idk = [0i32; 15];
        cursor.read_i32_into::<LE>(&mut idk)?;

        assert_eq!(
            cursor.stream_position()? - start_reading,
            0xf4,
            "StudioBone read missmatch"
        );

        let name = if bone_name_index != 0 {
            cursor.seek(SeekFrom::Start(start_reading + bone_name_index as u64))?;
            util::string_from_buf(cursor)
        } else {
            "".to_owned()
        };

        let surface_prop_name = if surface_prop_name_index != 0 {
            cursor.seek(SeekFrom::Start(
                start_reading + surface_prop_name_index as u64,
            ))?;
            util::string_from_buf(cursor)
        } else {
            "".to_owned()
        };

        Ok(StudioBone {
            start_pos: start_reading,

            name,
            parent,

            bone_controller,

            pos,
            quat,
            rot,

            pos_scale,
            rot_scale,

            pos2bone,

            quat_align,

            flags,

            procedural_rule,
            procedural_rule_index,

            physics_bone_index,

            surface_prop_name,

            contents,

            idk,
        })
    }
}
