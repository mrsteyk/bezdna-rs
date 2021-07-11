type DescType = (u32, i32);

// repr(C) will allow for easier reading in the future hopefully...
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioHdrT {
    pub header: u32,  // 0x0 - 0x54534449
    pub version: u32, // 0x4 - 0x35

    pub checksum: u32,     // 0x8 - ???
    pub mdlnameindex: u32, // 0xC - what the actual fuck...

    pub name: [u8; 64], // 0x10 - full model path, idk why

    pub length: u32, // 0x50 - ???

    pub eye_pos: [f32; 3],        // 0x54
    pub illum_position: [f32; 3], // 0x60
    pub hull_min: [f32; 3],       // 0x6c
    pub hull_max: [f32; 3],       // 0x78
    pub view_bbmin: [f32; 3],     // 0x84
    pub view_bbmax: [f32; 3],     // 0x90

    pub flags: u32, // 0x9c

    //numbones: u32, // 0xa0
    //boneindex: u32, // 0xa4
    pub bone_desc: DescType,

    //numbonecontrollers: u32, // 0xa8
    //bonecontrollerindex: u32, // 0xac
    pub bone_controller_desc: DescType,

    //numhitboxsets: u32, // 0xB0
    //hitboxsetindex: u32, // 0xB4
    pub hitbox_set_desc: DescType,

    //numlocalanim: u32, // 0xB8
    //localanimindex: u32, // 0xBC
    pub local_anim_desc: DescType,

    //numlocalseq: u32, // 0xC0
    //localseqindex: u32, // 0xC4
    pub local_seq_desc: DescType,

    pub activitylistversion: u32, // 0xC8
    pub eventsindexed: u32,       // 0xCC

    //numtextures: u32, // 0xD0
    //textureindex: u32, // 0xD4
    pub texture_desc: DescType,

    // TODO: find out what that is...
    pub numcdtextures: u32,  // 0xD8
    pub cdtextureindex: u32, // 0xDC

    pub numskinref: u32,      // 0xE0
    pub numskinfamilies: u32, // 0xE4
    pub skinindex: u32,       // 0xE8

    //numbodyparts: u32, // 0xEC
    //bodypartindex: u32, // 0xF0 - offset
    pub body_part_desc: DescType,

    pub local_attachment_desc: DescType, // 0xF4, 0xF8

    pub const_directional_light_dot: u8, // 0x164
    // -- skip to 0x165
    pub root_lod: u8,              // 0x165
    pub num_allowed_root_lods: u8, // 0x166

    // -- skip to 0x17c
    pub maya_name_index: u32, // 0x17c ???

    pub texture_file_offset: u32, // 0x1ac ???
    // -- skip to 0x1b0
    pub vertex_file_offset: u32, // 0x1b0
}

impl Default for StudioHdrT {
    #[inline]
    fn default() -> StudioHdrT {
        unsafe { std::mem::zeroed() } // very Rust like to complain at zeroed shit to be uninitialised
    }
}
