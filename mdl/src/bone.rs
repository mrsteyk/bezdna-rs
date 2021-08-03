use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::util;

use crate::se::*;

const WTF_SIZE: usize = 3;

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

    // there should be extra 3 floats???
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
    Invalid(u32),
    None,
    AxisInterp, // 1 TODO
    QuatInterp, // 2 TODO
    AimAtBone,  // 3 TODO
    AimAttach,  // 4 TODO
    Jiggle,     // 5 TODO
}

bitflags! {
    #[derive(Default)]
    pub struct Contents: u32 {
        const EMPTY = 0x0;
        const SOLID = 0x1;
        const WINDOW = 0x2;
        const AUX = 0x4;
        const GRATE = 0x8;
        const SLIME = 0x10;
        const WATER = 0x20;
        const MIST = 0x40;
        const OPAQUE = 0x80;

        const ALL_VISIBLE = 0xFF;

        const TESTFOGVOLUME = 0x100;

        const UNUSED5 = 0x200;
        const UNUSED6 = 0x4000;

        const TEAM1 = 0x800;
        const TEAM2 = 0x1000;

        const IGNORE_NODRAW_OPAQUE = 0x2000;

        const MOVEABLE = 0x4000;

        const AREA_PORTAL = 0x8000;

        const PLAYER_CLIP = 0x10000;
        const MONSTER_CLIP = 0x20000;

        const CURRENT_0 = 0x40000;
        const CURRENT_90 = 0x80000;
        const CURRENT_180 = 0x100000;
        const CURRENT_270 = 0x200000;
        const CURRENT_UP = 0x400000;
        const CURRENT_DOWN = 0x800000;

        const ORIGIN = 0x1000000;
        const MONSTER = 0x2000000;
        const DEBRIS = 0x4000000;
        const DETAIL = 0x8000000;
        const TRANSLUCENT = 0x10000000;
        const LADDER = 0x20000000;

        const HITBOX = 0x40000000;
    }
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

    pub wtf: [f32; WTF_SIZE],

    pub flags: u32,

    // wtf x2
    //pub procedural_rule: ProceduralRule,
    pub procedural_rule_type: u32,
    pub procedural_rule_index: i32,

    pub physics_bone_index: i32, // ???

    pub surface_prop_name: String,

    pub contents: Contents,

    pub idk: [i32; 12],
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

        // let wtf = cursor.read_f32::<LE>()?;
        // // debug-ish shit
        // cursor.read_f32::<LE>()?;
        // cursor.read_f32::<LE>()?;

        // TODO:
        //  ____ ___ ____   _____ _  _____   _____ ___  ____   ___
        // | __ )_ _/ ___| |  ___/ \|_   _| |_   _/ _ \|  _ \ / _ \
        // |  _ \| | |  _  | |_ / _ \ | |     | || | | | | | | | | |
        // | |_) | | |_| | |  _/ ___ \| |     | || |_| | |_| | |_| |
        // |____/___\____| |_|/_/   \_\_|     |_| \___/|____/ \___/

        let mut wtf = [0f32; WTF_SIZE];
        cursor.read_f32_into::<LE>(&mut wtf)?;

        let flags = cursor.read_u32::<LE>()?;

        let procedural_rule_type = cursor.read_u32::<LE>()?;
        let procedural_rule_index = cursor.read_i32::<LE>()?;
        let _procedural_rule = match procedural_rule_type {
            0 => ProceduralRule::None,
            1 => ProceduralRule::AxisInterp,
            2 => ProceduralRule::QuatInterp,
            3 => ProceduralRule::AimAtBone,
            4 => ProceduralRule::AimAttach,
            5 => ProceduralRule::Jiggle,
            v => ProceduralRule::Invalid(v), //panic!("Invalid procedural type {}", v),
        };

        let physics_bone_index = cursor.read_i32::<LE>()?;

        let surface_prop_name_index = cursor.read_i32::<LE>()?;

        let contents = cursor.read_u32::<LE>()?;

        let mut idk = [0i32; 12];
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

            wtf,

            flags,

            procedural_rule_type,
            procedural_rule_index,

            physics_bone_index,

            surface_prop_name,

            // What the actual fuck is this
            contents: unsafe { Contents::from_bits_unchecked(contents) },

            idk,
        })
    }
}
