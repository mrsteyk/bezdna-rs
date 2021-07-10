use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

mod studiohdr;
use studiohdr::StudioHdrT;

mod studiobodypart;
pub use studiobodypart::StudioBodyPart;
use studiobodypart::StudioBodyPartT;

mod hitbox;
use hitbox::StudioHitboxSetT;
pub use hitbox::{StudioHitbox, StudioHitboxSet};

mod bone;
pub use bone::StudioBone;
use bone::StudioBoneT;

mod util;

#[derive(Debug, PartialEq, Clone)]
pub struct StudioModel {
    header: (StudioHdrT, u64),

    // Parsed data
    pub body_parts: Vec<StudioBodyPart>,
    pub hitbox_sets: Vec<StudioHitboxSet>,
    pub bones: Vec<StudioBone>,
}

impl StudioModel {
    // TODO: better Error handling?...
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioModel, std::io::Error> {
        let mut hdr: StudioHdrT = Default::default();

        let start_reading = cursor.stream_position()?;
        hdr.header = cursor.read_u32::<LE>()?;
        if hdr.header != 0x54534449 {
            return Err(std::io::Error::from(ErrorKind::InvalidInput));
        }

        hdr.version = cursor.read_u32::<LE>()?;
        if hdr.version != 0x35 {
            return Err(std::io::Error::from(ErrorKind::Unsupported));
        }

        // TODO: find out the algo
        hdr.checksum = cursor.read_u32::<LE>()?;
        hdr.mdlnameindex = cursor.read_u32::<LE>()?; // such useful...

        cursor.read_exact(&mut hdr.name)?;

        hdr.length = cursor.read_u32::<LE>()?;

        cursor.read_f32_into::<LE>(&mut hdr.eye_pos)?;
        cursor.read_f32_into::<LE>(&mut hdr.illum_position)?;
        cursor.read_f32_into::<LE>(&mut hdr.hull_min)?;
        cursor.read_f32_into::<LE>(&mut hdr.hull_max)?;
        cursor.read_f32_into::<LE>(&mut hdr.view_bbmin)?;
        cursor.read_f32_into::<LE>(&mut hdr.view_bbmax)?;

        hdr.flags = cursor.read_u32::<LE>()?;

        hdr.bone_desc.0 = cursor.read_u32::<LE>()?;
        hdr.bone_desc.1 = cursor.read_i32::<LE>()?;

        hdr.bone_controller_desc.0 = cursor.read_u32::<LE>()?;
        hdr.bone_controller_desc.1 = cursor.read_i32::<LE>()?;

        hdr.hitbox_set_desc.0 = cursor.read_u32::<LE>()?;
        hdr.hitbox_set_desc.1 = cursor.read_i32::<LE>()?;

        hdr.local_anim_desc.0 = cursor.read_u32::<LE>()?;
        hdr.local_anim_desc.1 = cursor.read_i32::<LE>()?;

        hdr.local_seq_desc.0 = cursor.read_u32::<LE>()?;
        hdr.local_seq_desc.1 = cursor.read_i32::<LE>()?;

        hdr.activitylistversion = cursor.read_u32::<LE>()?;
        hdr.eventsindexed = cursor.read_u32::<LE>()?;

        hdr.texture_desc.0 = cursor.read_u32::<LE>()?;
        hdr.texture_desc.1 = cursor.read_i32::<LE>()?;

        // is this even useful?
        hdr.numcdtextures = cursor.read_u32::<LE>()?;
        hdr.cdtextureindex = cursor.read_u32::<LE>()?;

        hdr.numskinref = cursor.read_u32::<LE>()?;
        hdr.numskinfamilies = cursor.read_u32::<LE>()?;
        hdr.skinindex = cursor.read_u32::<LE>()?;

        hdr.body_part_desc.0 = cursor.read_u32::<LE>()?;
        hdr.body_part_desc.1 = cursor.read_i32::<LE>()?;

        // SKIP TO 0x164
        cursor.seek(SeekFrom::Start(start_reading + 0x164))?;
        hdr.const_directional_light_dot = cursor.read_u8()?;
        hdr.root_lod = cursor.read_u8()?;
        hdr.num_allowed_root_lods = cursor.read_u8()?;

        // SKIP TO 0x17c
        cursor.seek(SeekFrom::Start(start_reading + 0x17c))?;
        hdr.maya_name_index = cursor.read_u32::<LE>()?;

        // SKIP TO 0x1b0
        cursor.seek(SeekFrom::Start(start_reading + 0x1b0))?;
        hdr.vertex_file_offset = cursor.read_u32::<LE>()?;

        // TODO: parse body parts and shit...

        let mut body_parts: Vec<StudioBodyPart> = Vec::new();
        for i in 0..hdr.body_part_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.body_part_desc.1 as u64 + (0x10 * i) as u64,
            ))?;
            body_parts.push(StudioBodyPart::read(cursor)?);
        }

        let mut hitbox_sets: Vec<StudioHitboxSet> = Vec::new();
        for i in 0..hdr.hitbox_set_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.hitbox_set_desc.1 as u64 + (0xC * i) as u64,
            ))?;
            hitbox_sets.push(StudioHitboxSet::read(cursor)?);
        }

        let mut bones: Vec<StudioBone> = Vec::new();
        for i in 0..hdr.bone_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.bone_desc.1 as u64 + (0xf4 * i) as u64,
            ))?;
            bones.push(StudioBone::read(cursor)?);
        }

        Ok(StudioModel {
            header: (hdr, start_reading),

            body_parts,
            hitbox_sets,
            bones,
        })
    }
}
