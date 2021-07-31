use std::{
    cell::{RefCell, RefMut},
    io::{Cursor, Read, Seek, SeekFrom},
    rc::Rc,
};

use byteorder::{ReadBytesExt, LE};

use crate::{decomp::decompress, util::string_from_buf, DataChunk, SectionDesc};

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

    pub size_decompressed: u64,
    pub unk30: u64,

    pub starpak_len: u16,
    pub sections_num: u16,
    pub data_chunks_num: u16,

    pub part_rpak: u16,

    pub unk40: u32,
    pub num_files: u32,
    pub unk48: u32,
    pub unk4c: u32,

    pub unk50: u32,
    pub unk54: u32,
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
        if version != 7 {
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

            size_decompressed: cursor.read_u64::<LE>()?,
            unk30: cursor.read_u64::<LE>()?,

            starpak_len: cursor.read_u16::<LE>()?,
            sections_num: cursor.read_u16::<LE>()?,
            data_chunks_num: cursor.read_u16::<LE>()?,

            part_rpak: cursor.read_u16::<LE>()?,

            unk40: cursor.read_u32::<LE>()?,
            num_files: cursor.read_u32::<LE>()?,
            unk48: cursor.read_u32::<LE>()?,
            unk4c: cursor.read_u32::<LE>()?,

            unk50: cursor.read_u32::<LE>()?,
            unk54: cursor.read_u32::<LE>()?,
        };

        if cursor.stream_position()? != crate::HEADER_SIZE_TF2 as u64 {
            return Err(crate::RPakError::Shiz("tf2::RPakFile::read".to_owned()));
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
        crate::RPakVersion::TF2
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

        cursor.seek(SeekFrom::Start(0))?;
        let mut vec = Vec::<u8>::new();
        cursor.read_to_end(&mut vec)?;
        // TODO: maybe check disk size?
        let mut decompressed = if header.is_compressed() {
            let mut d = decompress(
                &mut vec,
                header.size_decompressed as usize,
                crate::HEADER_SIZE_TF2,
            )?;
            d[..crate::HEADER_SIZE_TF2].clone_from_slice(&vec[..crate::HEADER_SIZE_TF2]);
            Cursor::new(d)
        } else {
            Cursor::new(vec)
        };

        decompressed.seek(SeekFrom::Start(crate::HEADER_SIZE_TF2 as u64))?;
        let starpak = string_from_buf(&mut decompressed);

        let starpak_skipped = crate::HEADER_SIZE_TF2 as u64 + header.starpak_len as u64;
        decompressed.seek(SeekFrom::Start(starpak_skipped))?;
        let sections = SectionDesc::parse(&mut decompressed, header.sections_num)?;

        let sections_skipped = starpak_skipped + (16 * header.sections_num as u64);
        decompressed.seek(SeekFrom::Start(sections_skipped))?;
        let data_chunks = DataChunk::parse(&mut decompressed, header.data_chunks_num)?;

        let data_chunks_skipped = sections_skipped + (12 * header.data_chunks_num as u64);
        // unk40 here

        let unk40_skipped = data_chunks_skipped + (8 * header.unk40 as u64);
        // parsing files is moved so we can get juicy file offsets

        let file_entries_skipped = unk40_skipped + (72 * header.num_files as u64);
        // unk48

        let unk48_skipped = file_entries_skipped + (8 * header.unk48 as u64);
        // unk4c here

        let unk4c_skipped = unk48_skipped + (4 * header.unk4c as u64);
        // unk50 here

        let unk50_skipped = unk4c_skipped + (4 * header.unk50 as u64);
        // unk54 here

        let unk54_skipped = unk50_skipped + (1 * header.unk54 as u64);

        // populate seek array
        let mut seeks = vec![0u64; header.data_chunks_num as usize];
        if header.data_chunks_num > 0 {
            seeks[0] = unk54_skipped;
            if header.data_chunks_num > 1 {
                for i in 1..header.data_chunks_num as usize {
                    seeks[i] = seeks[i - 1] + data_chunks[i - 1].size as u64;
                }
            }
        }

        // populate files array
        decompressed.seek(SeekFrom::Start(unk40_skipped))?;
        let mut files = Vec::<Rc<dyn crate::FileEntry>>::with_capacity(header.num_files as usize);
        for _ in 0..header.num_files {
            let generic = FileGeneric::read(&mut decompressed, &seeks).unwrap();
            // mb move the actual parsing?
            let bak_pos = decompressed.stream_position()?;
            let spec: Rc<dyn crate::FileEntry> = match generic.extension.as_str() {
                // "txtr" => Rc::new(
                //     filetypes::txtr::Texture::ctor(&mut decompressed, &seeks, generic).unwrap(),
                // ),
                "matl" => {
                    Rc::new(filetypes::matl::Material::ctor(&mut decompressed, &seeks, generic).unwrap())
                }
                // "ui" => {
                //     Rc::new(filetypes::rui::RUI::ctor(&mut decompressed, &seeks, generic).unwrap())
                // }
                // "uimg" => Rc::new(
                //     filetypes::uimg::UImg::ctor(&mut decompressed, &seeks, generic).unwrap(),
                // ),
                // "dtbl" => Rc::new(
                //     filetypes::dtbl::DataTable::ctor(&mut decompressed, &seeks, generic).unwrap(),
                // ),
                // "stgs" => Rc::new(
                //     filetypes::stgs::Settings::ctor(&mut decompressed, &seeks, generic).unwrap(),
                // ),
                // "stlt" => Rc::new(
                //     filetypes::stlt::SettingsLayout::ctor(&mut decompressed, &seeks, generic)
                //         .unwrap(),
                // ),
                _ => Rc::new(generic),
            };
            decompressed.seek(SeekFrom::Start(bak_pos))?;
            files.push(spec);
        }

        // We don't parse data just yet?...
        Ok(Self {
            header,
            decompressed: Rc::new(RefCell::new(decompressed)),
            data_start: unk54_skipped,

            starpak,
            files,
            sections,
            data_chunks,

            seeks,
        })
    }
}
