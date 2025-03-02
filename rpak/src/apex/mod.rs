use std::cell::{RefCell, RefMut};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::rc::Rc;

use byteorder::{ReadBytesExt, LE};

use crate::decomp::decompress;
use crate::util::string_from_buf;
use crate::{DataChunk, SectionDesc};

use self::filetypes::FileGeneric;

pub mod filetypes;

#[derive(Debug)]
pub struct RPakHeader {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,

    pub typ: u64,
    pub unk10: u64,

    pub size_disk: u64,
    pub unk20: u64,
    pub unk28: u64,

    pub size_decompressed: u64,
    pub unk38: u64,
    pub unk40: u64,

    pub starpak_len: u16,     // 0x48
    pub starpak_opt_len: u16, // 0x4a
    pub sections_num: u16,    // 0x4c
    pub data_chunks_num: u16, // 0x4e

    pub part_rpak: u16, // 0x50

    pub unk52: u16,
    pub unk54: u32,
    pub num_files: u32,
    pub unk5c: u32,

    pub unk60: u32,
    pub unk64: u32,
    pub unk68: u32,
    pub unk6c: u32,

    pub unk70: u32,
    pub unk74: u32,
    pub unk78: u64,
}

impl RPakHeader {
    pub fn is_compressed(&self) -> bool {
        (self.flags >> 8) & 0xFF == 1
    }
    pub fn should_lla(&self) -> bool {
        // wait what?
        self.flags & 0x11 != 0
    }

    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, crate::RPakError> {
        cursor.seek(SeekFrom::Start(0))?;
        let magic = cursor.read_u32::<LE>()?;
        if magic != 0x6b615052 {
            return Err(crate::RPakError::InvalidMagic(magic));
        }
        let version = cursor.read_u16::<LE>()?;
        if version != 8 {
            return Err(crate::RPakError::InvalidVersion(version));
        }
        let flags = cursor.read_u16::<LE>()?;

        let header = RPakHeader {
            magic,
            version,
            flags,

            typ: cursor.read_u64::<LE>()?,
            unk10: cursor.read_u64::<LE>()?,
            size_disk: cursor.read_u64::<LE>()?,

            unk20: cursor.read_u64::<LE>()?,
            unk28: cursor.read_u64::<LE>()?,

            size_decompressed: cursor.read_u64::<LE>()?,
            unk38: cursor.read_u64::<LE>()?,
            unk40: cursor.read_u64::<LE>()?,

            starpak_len: cursor.read_u16::<LE>()?,
            starpak_opt_len: cursor.read_u16::<LE>()?,
            sections_num: cursor.read_u16::<LE>()?,
            data_chunks_num: cursor.read_u16::<LE>()?,

            part_rpak: cursor.read_u16::<LE>()?,
            unk52: cursor.read_u16::<LE>()?,

            unk54: cursor.read_u32::<LE>()?,
            num_files: cursor.read_u32::<LE>()?,
            unk5c: cursor.read_u32::<LE>()?,

            unk60: cursor.read_u32::<LE>()?,
            unk64: cursor.read_u32::<LE>()?,
            unk68: cursor.read_u32::<LE>()?,
            unk6c: cursor.read_u32::<LE>()?,

            unk70: cursor.read_u32::<LE>()?,
            unk74: cursor.read_u32::<LE>()?,

            unk78: cursor.read_u64::<LE>()?,
        };

        if cursor.stream_position()? != crate::HEADER_SIZE_APEX as u64 {
            return Err(crate::RPakError::Shiz("apex::RPakFile::read".to_owned()));
        }

        Ok(header)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RPakFile {
    pub header: RPakHeader,
    #[derivative(Debug = "ignore")]
    pub decompressed: Rc<RefCell<Cursor<Vec<u8>>>>,
    pub data_start: u64,

    pub starpak: String,
    pub starpak_opt: Option<String>,
    pub files: Vec<std::rc::Rc<dyn crate::FileEntry>>,
    pub sections: Vec<crate::SectionDesc>,
    #[derivative(Debug = "ignore")]
    pub data_chunks: Vec<DataChunk>,

    #[derivative(Debug = "ignore")]
    pub seeks: Vec<u64>,
}

impl crate::RPakFile for RPakFile {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_version(&self) -> crate::RPakVersion {
        crate::RPakVersion::APEX
    }
    fn get_files(&self) -> &Vec<std::rc::Rc<dyn crate::FileEntry>> {
        self.files.as_ref()
    }
    fn get_sections_desc(&self) -> &Vec<crate::SectionDesc> {
        self.sections.as_ref()
    }
    fn get_data_chunks(&self) -> &Vec<DataChunk> {
        self.data_chunks.as_ref()
    }

    fn is_compressed(&self) -> bool {
        self.header.is_compressed()
    }
    fn should_lla(&self) -> bool {
        self.header.should_lla()
    }

    fn get_decompressed(&self) -> RefMut<Cursor<Vec<u8>>> {
        (*self.decompressed).borrow_mut()
    }
}

impl RPakFile {
    pub fn read<R: Read + Seek + ReadBytesExt>(cursor: &mut R) -> Result<Self, crate::RPakError> {
        let header = RPakHeader::read(cursor)?;

        if header.part_rpak != 0 {
            todo!("Part RPak")
        }
        if header.unk74 != 0 {
            todo!()
        }

        cursor.seek(SeekFrom::Start(0))?;
        let mut vec = Vec::<u8>::new();
        cursor.read_to_end(&mut vec)?;
        // TODO: maybe check disk size?
        let mut decompressed = if header.is_compressed() {
            let mut d = decompress(
                &mut vec,
                header.size_decompressed as usize,
                crate::HEADER_SIZE_APEX,
            )?;
            d[..crate::HEADER_SIZE_APEX].clone_from_slice(&vec[..crate::HEADER_SIZE_APEX]);
            Cursor::new(d)
        } else {
            Cursor::new(vec)
        };

        decompressed.seek(SeekFrom::Start(crate::HEADER_SIZE_APEX as u64))?;
        let starpak = string_from_buf(&mut decompressed);

        let starpak_skipped = crate::HEADER_SIZE_APEX as u64 + header.starpak_len as u64;
        decompressed.seek(SeekFrom::Start(starpak_skipped))?;
        let starpak_opt = {
            let tmp = string_from_buf(&mut decompressed);
            match tmp.len() {
                0 => None,
                _ => Some(tmp),
            }
        };

        let starpak_opt_skipped = starpak_skipped + header.starpak_opt_len as u64;
        decompressed.seek(SeekFrom::Start(starpak_opt_skipped))?;
        let sections = SectionDesc::parse(&mut decompressed, header.sections_num)?;

        let sections_skipped = starpak_opt_skipped + (16 * header.sections_num as u64);
        decompressed.seek(SeekFrom::Start(sections_skipped))?;
        let data_chunks = DataChunk::parse(&mut decompressed, header.data_chunks_num)?;

        let data_chunks_skipped = sections_skipped + (12 * header.data_chunks_num as u64);
        // unk54 here

        let unk54_skipped = data_chunks_skipped + (8 * header.unk54 as u64);
        // parsing files is moved so we can get juicy file offsets

        let file_entries_skipped = unk54_skipped + (0x50 * header.num_files as u64);
        // unk5c here

        let unk5c_skipped = file_entries_skipped + (8 * header.unk5c as u64);
        // unk60 here

        let unk60_skipped = unk5c_skipped + (4 * header.unk60 as u64);
        // unk64 here

        let unk64_skipped = unk60_skipped + (4 * header.unk64 as u64);
        // unk68 here

        let unk68_skipped = unk64_skipped + header.unk68 as u64;
        // unk6c here

        let unk6c_skipped = unk68_skipped + (16 * header.unk6c as u64);
        // unk70 here

        let unk70_skipped = unk6c_skipped + (24 * header.unk70 as u64);

        // populate seek array
        let mut seeks = vec![0u64; header.data_chunks_num as usize];
        if header.data_chunks_num > 0 {
            seeks[0] = unk70_skipped;
            if header.data_chunks_num > 1 {
                for i in 1..header.data_chunks_num as usize {
                    seeks[i] = seeks[i - 1] + data_chunks[i - 1].size as u64;
                }
            }
        }

        // populate files array
        decompressed.seek(SeekFrom::Start(unk54_skipped))?;
        let mut files = Vec::<Rc<dyn crate::FileEntry>>::with_capacity(header.num_files as usize);
        for _ in 0..header.num_files {
            let generic = FileGeneric::read(&mut decompressed, &seeks).unwrap();
            // mb move the actual parsing?
            let bak_pos = decompressed.stream_position()?;
            let spec: Rc<dyn crate::FileEntry> = match generic.extension.as_str() {
                "txtr" => Rc::new(
                    filetypes::txtr::Texture::ctor(&mut decompressed, &seeks, generic).unwrap(),
                ),
                "matl" => Rc::new(
                    filetypes::matl::Material::ctor(&mut decompressed, &seeks, generic).unwrap(),
                ),
                "ui" => {
                    Rc::new(filetypes::rui::RUI::ctor(&mut decompressed, &seeks, generic).unwrap())
                }
                "uimg" => Rc::new(
                    filetypes::uimg::UImg::ctor(&mut decompressed, &seeks, generic).unwrap(),
                ),
                "dtbl" => Rc::new(
                    filetypes::dtbl::DataTable::ctor(&mut decompressed, &seeks, generic).unwrap(),
                ),
                "stgs" => Rc::new(
                    filetypes::stgs::Settings::ctor(&mut decompressed, &seeks, generic).unwrap(),
                ),
                "stlt" => Rc::new(
                    filetypes::stlt::SettingsLayout::ctor(&mut decompressed, &seeks, generic)
                        .unwrap(),
                ),
                _ => Rc::new(generic),
            };
            decompressed.seek(SeekFrom::Start(bak_pos))?;
            files.push(spec);
        }

        // We don't parse data just yet?...
        Ok(RPakFile {
            header,
            decompressed: Rc::new(RefCell::new(decompressed)),
            data_start: unk70_skipped,

            starpak,
            starpak_opt,
            files,
            sections,
            data_chunks,

            seeks,
        })
    }
}
