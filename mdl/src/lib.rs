#[macro_use]
extern crate bitflags;

use byteorder::{ReadBytesExt, LE};
use std::io::{ErrorKind, Read, Seek, SeekFrom};

pub mod se;

mod studiohdr;
use studiohdr::StudioHdrT;

mod studiobodypart;
pub use studiobodypart::StudioBodyPart;
//use studiobodypart::StudioBodyPartT;

mod hitbox;
pub use hitbox::{StudioHitbox, StudioHitboxSet};
//use hitbox::StudioHitboxSetT;

mod bone;
pub use bone::{ProceduralRule, StudioBone};
//use bone::StudioBoneT;

mod attachment;
pub use attachment::StudioAttachment;
//use attachment::StudioAttachmentT;

mod model;
pub use model::StudioModel;
//use model::StudioModelT;

mod mesh;
pub use mesh::StudioMesh;
//use mesh::StudioMeshT;

pub mod vtx;
pub mod vvd;

mod util;

#[derive(Debug, PartialEq)]
pub struct StudioMdl {
    header: (StudioHdrT, u64),

    // Parsed data
    pub body_parts: Vec<StudioBodyPart>,
    pub hitbox_sets: Vec<StudioHitboxSet>,
    pub bones: Vec<StudioBone>,
    pub local_attachments: Vec<StudioAttachment>,

    pub animation_only: bool,

    // Parsed embeds
    pub vtx: vtx::VtxFile,
    pub vvd: vvd::VvdFile,
}

impl StudioMdl {
    // TODO: better Error handling?...
    pub fn read<R: Read + Seek + ReadBytesExt>(
        cursor: &mut R,
    ) -> std::result::Result<StudioMdl, std::io::Error> {
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

        hdr.local_attachment_desc.0 = cursor.read_u32::<LE>()?;
        hdr.local_attachment_desc.1 = cursor.read_i32::<LE>()?;

        // SKIP TO 0x164
        cursor.seek(SeekFrom::Start(start_reading + 0x164))?;
        hdr.const_directional_light_dot = cursor.read_u8()?;
        hdr.root_lod = cursor.read_u8()?;
        hdr.num_allowed_root_lods = cursor.read_u8()?;

        // SKIP TO 0x17c
        cursor.seek(SeekFrom::Start(start_reading + 0x17c))?;
        hdr.maya_name_index = cursor.read_u32::<LE>()?;

        // SKIP TO 0x1ac
        cursor.seek(SeekFrom::Start(start_reading + 0x1ac))?;
        hdr.texture_file_offset = cursor.read_u32::<LE>()?;
        hdr.vertex_file_offset = cursor.read_u32::<LE>()?;

        // TODO: parse body parts and shit...

        let mut body_parts = Vec::<StudioBodyPart>::with_capacity(hdr.body_part_desc.0 as usize);
        for i in 0..hdr.body_part_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.body_part_desc.1 as u64 + (0x10 * i) as u64,
            ))?;
            body_parts.push(StudioBodyPart::read(cursor)?);
        }

        let mut hitbox_sets = Vec::<StudioHitboxSet>::with_capacity(hdr.hitbox_set_desc.0 as usize);
        for i in 0..hdr.hitbox_set_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.hitbox_set_desc.1 as u64 + (0xC * i) as u64,
            ))?;
            hitbox_sets.push(StudioHitboxSet::read(cursor)?);
        }

        let mut bones = Vec::<StudioBone>::with_capacity(hdr.bone_desc.0 as usize);
        for i in 0..hdr.bone_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.bone_desc.1 as u64 + (0xf4 * i) as u64,
            ))?;
            bones.push(StudioBone::read(cursor)?);
        }

        let mut local_attachments =
            Vec::<StudioAttachment>::with_capacity(hdr.local_attachment_desc.0 as usize);
        for i in 0..hdr.local_attachment_desc.0 {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.local_attachment_desc.1 as u64 + (0x5c * i) as u64,
            ))?;
            local_attachments.push(StudioAttachment::read(cursor)?);
        }

        let vtx = {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.texture_file_offset as u64,
            ))?;
            vtx::VtxFile::read(cursor)?
        };

        let vvd = {
            cursor.seek(SeekFrom::Start(
                start_reading + hdr.vertex_file_offset as u64,
            ))?;
            vvd::VvdFile::read(cursor)?
        };

        Ok(StudioMdl {
            header: (hdr, start_reading),

            body_parts,
            hitbox_sets,
            bones,
            local_attachments,

            animation_only: (hdr.body_part_desc.0 == 0) && (hdr.local_seq_desc.0 > 0),

            vtx,
            vvd,
        })
    }
}
