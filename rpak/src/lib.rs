mod binding;

mod consts;
pub use consts::*;

mod hashing;
pub use hashing::hash;

// dynamic shit per ext
pub trait FileEntry {
    // getters
    fn get_ext(&self) -> String;
    fn get_name(&self) -> String;
    fn get_desc_off(&self) -> u64;
    fn get_data_off(&self) -> u64;
    fn get_star_off(&self) -> u64;
    fn get_star_opt_off(&self) -> u64; // WILL BE 0/-1 ON TF2
}

// this is static across all(2) games
pub struct SectionDesc {}

pub trait RPakFile {
    fn get_version(&self) -> RPakVersion;
    fn get_files(&self) -> dyn FileEntry;
}

fn get_decompressed_size(
    a1: &mut [u64; 18],
    file_buf: &mut Vec<u8>,
    header_size: usize,
) -> usize {
    unsafe {
        binding::get_decompressed_size(
            a1 as *mut _ as *mut i64,
            file_buf.as_mut_ptr(),
            -1,
            file_buf.len() as i64,
            0,
            header_size as i64,
        ) as usize
    }
}

fn decompress_rpak(
    a1: &mut [u64; 18],
    file_buf: &mut Vec<u8>,
    decompressed_size: usize,
) -> Option<Vec<u8>> {
    unsafe {
        let mut out: Vec<u8> = Vec::with_capacity(decompressed_size);

        a1[1] = out.as_mut_ptr() as u64;
        a1[3] = u64::MAX;

        if binding::decompress_rpak(
            a1 as *mut _ as *mut i64,
            file_buf.len() as u64,
            decompressed_size as u64,
        ) == 1
        {
            Some(out)
        } else {
            None
        }
    }
}

pub enum Error {
    InvalidDecompressedSize,
    DecompressionError,
}

// for the fans of being raw out there...
pub fn decompress(
    file_buf: &mut Vec<u8>,
    decompressed_size_expected: usize,
    header_size: usize,
) -> std::result::Result<Vec<u8>, Error> {
    let mut a1 = [0u64; 18];
    let decompressed_size = get_decompressed_size(&mut a1, file_buf, header_size);
    if decompressed_size_expected == decompressed_size {
        match decompress_rpak(&mut a1, file_buf, decompressed_size) {
            Some(ret) => Ok(ret),
            None => Err(Error::DecompressionError),
        }
    } else {
        Err(Error::InvalidDecompressedSize)
    }
}