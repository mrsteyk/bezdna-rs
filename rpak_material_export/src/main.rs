use std::{
    borrow::BorrowMut,
    io::{Cursor, Read, Seek, SeekFrom},
    os::windows::prelude::FileExt,
};
#[forbid(unsafe_code)]
use std::{fs::File, io::BufReader, path::Path};

use rpak::tf2;

use serde::{Deserialize, Serialize};

// instead of hashmap?
#[derive(Debug, Serialize, Deserialize)]
struct MaterialJson<'a> {
    surface_properties: &'a str,
    // textures
    color: String,
    normal: String,
    rough: String,
    spec: String,
    illumm: String,
    bm: String,
    ao: String,
    cav: String,
    opa: String, // or trans???
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{:#?}", args);
    if args.len() < 3 {
        eprintln!("Invalid usage!");
    } else {
        let path = Path::new(&args[1]);
        let material_out_path = Path::new(&args[2]);

        let mut starpak = if args.len() > 3 {
            Some(File::open(&args[3]).unwrap())
        } else {
            None
        };

        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let file = File::open(path).unwrap();
        println!("stem: {}", file_stem);
        let mut cursor = std::io::Cursor::new(BufReader::new(file));

        let common_file = File::open(path.parent().unwrap_or(Path::new(".")).join("common.rpak"));
        let mut common_cursor = if let Ok(cf) = &common_file {
            Some(std::io::Cursor::new(BufReader::new(cf)))
        } else {
            None
        };
        let common_rpak = if let Some(cr) = common_cursor.as_mut() {
            Some(rpak::parse_rpak(cr.get_mut()).unwrap())
        } else {
            None
        };

        match rpak::parse_rpak(cursor.get_mut()) {
            Ok(drpak) => {
                // TODO: apex...
                let files = drpak.get_files();
                for f in files {
                    if f.get_ext() == "matl" {
                        if let Some(tf2_mat) =
                            f.as_any().downcast_ref::<tf2::filetypes::matl::Material>()
                        {
                            // TF2 logic...

                            // lets """assert""" before anything bad happens...
                            if tf2_mat.texture_guids.len()
                                > tf2::filetypes::matl::TEXTURE_REFS_SE2.len()
                            {
                                eprintln!(
                                    "Material has too many textures! {} | {}",
                                    tf2_mat.texture_guids.len(),
                                    tf2_mat.name
                                );
                                continue;
                            }
                            let mut bad = false;
                            for guid in &tf2_mat.texture_guids {
                                if *guid == 0 {
                                    continue; // no texture present for this thing
                                }
                                if let Some(atexture) = files.iter().find(|x| x.get_guid() == *guid)
                                {
                                    let texture = atexture
                                        .as_any()
                                        .downcast_ref::<tf2::filetypes::txtr::Texture>() // TODO: texture type shit
                                        .unwrap();
                                    //assert_neq!(, "UNKNOWN", "Unknown compression type for {}", texture.texture_type)
                                    if tf2::filetypes::txtr::TEXTURE_ALGOS
                                        [texture.texture_type as usize]
                                        == "UNKNOWN"
                                    {
                                        bad = true;
                                        eprintln!(
                                            "Material has bad compression type {} | {} | {}",
                                            texture.texture_type, texture.name, tf2_mat.name
                                        );
                                        break;
                                    }
                                } else {
                                    bad = true;
                                    eprintln!(
                                        "Material has texture not in this RPak {:016X} | {}",
                                        guid, tf2_mat.name
                                    );
                                    if let Some(common) = common_rpak.as_ref() {
                                        if let Some(common_texture) = common
                                            .get_files()
                                            .iter()
                                            .find(|x| x.get_guid() == *guid)
                                        {
                                            bad = false;
                                            eprintln!(
                                                "Material has texture FROM COMMON {:016X} | {} | {}",
                                                guid, common_texture.get_name().unwrap(), tf2_mat.name
                                            );
                                            continue;
                                        }
                                    }
                                    break;
                                }
                            }
                            if bad {
                                continue;
                            }
                            let folder_str = &tf2_mat.name.replace("\\", "/");
                            //let folder = Path::new(&folder_str);
                            //let mat_str = material_out_path.file_name().unwrap(); // for SE2 shit
                            let mat_path = material_out_path.join(&folder_str);

                            std::fs::create_dir_all(mat_path).unwrap();

                            // lets export all textures...
                            for i in 0..tf2_mat.texture_guids.len() {
                                let guid = &tf2_mat.texture_guids[i];
                                if *guid == 0 {
                                    continue;
                                }
                                let mut is_from_common = false;
                                let texture_any = if let Some(common) = &common_rpak {
                                    if let Some(tmp) = files.iter().find(|x| x.get_guid() == *guid)
                                    {
                                        tmp
                                    } else {
                                        is_from_common = true;
                                        common
                                            .get_files()
                                            .iter()
                                            .find(|x| x.get_guid() == *guid)
                                            .unwrap()
                                    }
                                } else {
                                    files.iter().find(|x| x.get_guid() == *guid).unwrap()
                                };
                                let texture = texture_any
                                    .as_any()
                                    .downcast_ref::<tf2::filetypes::txtr::Texture>()
                                    .unwrap();
                                if starpak.is_none()
                                    || texture.mipmaps.last().unwrap().typ
                                        == tf2::filetypes::txtr::MipMapType::RPak
                                {
                                    let mipmap = texture
                                        .mipmaps
                                        .iter()
                                        .rev()
                                        .find(|x| x.typ == tf2::filetypes::txtr::MipMapType::RPak)
                                        .unwrap();
                                    let decomp = if is_from_common {
                                        common_rpak.as_ref().unwrap().get_decompressed()
                                    } else {
                                        drpak.get_decompressed()
                                    };

                                    let mut data =
                                        Vec::<u8>::with_capacity(0x80 + mipmap.size as usize);

                                    match tf2::filetypes::txtr::TEXTURE_ALGOS
                                        [texture.texture_type as usize]
                                    {
                                        // Народный формат ящетаю
                                        "DXT1" => {
                                            data.extend_from_slice(include_bytes!("dxt1_hdr"));
                                        }
                                        "BC4U" => {
                                            data.extend_from_slice(include_bytes!("bc4u_hdr"));
                                        }
                                        "BC5U" => {
                                            data.extend_from_slice(include_bytes!("bc5u_hdr"));
                                        }
                                        "BC7U" => {
                                            data.extend_from_slice(include_bytes!("bc7u_hdr"));
                                        }
                                        "BC6H" => {
                                            // TODO
                                            eprintln!("Encountered BC6H! {}", texture.name);
                                            continue;
                                        }
                                        v => {
                                            panic!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x14..0x14 + 4]
                                        .copy_from_slice(&(mipmap.size as u32).to_le_bytes());

                                    data.extend_from_slice(
                                        &decomp.get_ref()[mipmap.off as usize
                                            ..mipmap.off as usize + mipmap.size as usize],
                                    );

                                    std::fs::write(
                                        material_out_path.join(format!(
                                            "{}/{}.dds",
                                            folder_str,
                                            tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
                                        )),
                                        data,
                                    )
                                    .unwrap();
                                } else if let Some(_star) = starpak.as_mut() {
                                    let mipmap = texture.mipmaps.last().unwrap();

                                    let mut data =
                                        Vec::<u8>::with_capacity(0x80 + mipmap.size as usize);

                                    match tf2::filetypes::txtr::TEXTURE_ALGOS
                                        [texture.texture_type as usize]
                                    {
                                        // Народный формат ящетаю
                                        "DXT1" => {
                                            data.extend_from_slice(include_bytes!("dxt1_hdr"));
                                        }
                                        "BC4U" => {
                                            data.extend_from_slice(include_bytes!("bc4u_hdr"));
                                        }
                                        "BC5U" => {
                                            data.extend_from_slice(include_bytes!("bc5u_hdr"));
                                        }
                                        "BC7U" => {
                                            data.extend_from_slice(include_bytes!("bc7u_hdr"));
                                        }
                                        "BC6H" => {
                                            // TODO
                                            eprintln!("Encountered BC6H! {}", texture.name);
                                            continue;
                                        }
                                        v => {
                                            unreachable!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x14..0x14 + 4]
                                        .copy_from_slice(&(mipmap.size as u32).to_le_bytes());

                                    if mipmap.off & 0xF != 0 {
                                        let p = Path::new(&args[3]);
                                        let star_id = mipmap.off & 0xF;

                                        let starpak_name = format!(
                                            "{}({:02}).starpak",
                                            p.file_stem().unwrap().to_str().unwrap(),
                                            star_id
                                        );
                                        let fullpath = p.parent().unwrap().join(&starpak_name);
                                        // eprintln!(
                                        //     "Filename: {} | {}",
                                        //     &starpak_name,
                                        //     fullpath.as_os_str().to_str().unwrap()
                                        // );

                                        let mut ff = File::open(fullpath).unwrap();
                                        ff.seek(SeekFrom::Start(mipmap.off - star_id)).unwrap();
                                        let mut buf = vec![0u8; mipmap.size as usize]; //Vec::<u8>::with_capacity(mipmap.size as usize);
                                        ff.read_exact(buf.as_mut()).unwrap();

                                        data.extend_from_slice(buf.as_slice());
                                    } else {
                                        _star.seek(SeekFrom::Start(mipmap.off)).unwrap();
                                        let mut buf = vec![0u8; mipmap.size as usize]; //Vec::<u8>::with_capacity(mipmap.size as usize);
                                        _star.read_exact(buf.as_mut()).unwrap();

                                        data.extend_from_slice(buf.as_slice());
                                    };

                                    std::fs::write(
                                        material_out_path.join(format!(
                                            "{}/{}.dds",
                                            folder_str,
                                            tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
                                        )),
                                        data,
                                    )
                                    .unwrap();
                                } else {
                                    unreachable!()
                                }
                                //println!("{:#?}", mipmap);
                            }

                            let find_texture = |i: usize| {
                                if tf2_mat.texture_guids.len() <= i {
                                    return "".to_owned();
                                }
                                let guid = tf2_mat.texture_guids[i];
                                if let Some(txtr) = files.iter().find(|x| x.get_guid() == guid) {
                                    return format!(
                                        "{}/{}.dds",
                                        folder_str,
                                        tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
                                    )
                                    .to_owned(); //txtr.get_name().unwrap().replace("\\", "/");
                                } else {
                                    if common_rpak.is_none() {
                                        return "".to_owned();
                                    }
                                    if let Some(txtr) = common_rpak
                                        .as_ref()
                                        .unwrap()
                                        .get_files()
                                        .iter()
                                        .find(|x| x.get_guid() == guid)
                                    {
                                        return format!(
                                            "{}/{}.dds",
                                            folder_str,
                                            tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
                                        )
                                        .to_owned();
                                    } else {
                                        return "".to_owned();
                                    }
                                }
                            };

                            let material_json_str = folder_str.to_owned() + ".json";
                            let material_json_path = material_out_path.join(&material_json_str);

                            let color = find_texture(0);
                            let normal = find_texture(1);
                            let rough = find_texture(2);
                            let spec = find_texture(3);
                            let illumm = find_texture(4);
                            let bm = find_texture(8);
                            let ao = find_texture(11);
                            let cav = find_texture(12);
                            let opa = find_texture(13);

                            let matjs = MaterialJson {
                                surface_properties: &tf2_mat.surface_props,

                                color,
                                normal,
                                rough,
                                spec,
                                illumm,
                                bm,
                                ao,
                                cav,
                                opa,
                            };
                            let mat_serial = serde_json::to_string_pretty(&matjs).unwrap();
                            std::fs::write(material_json_path, mat_serial).unwrap();
                        } else {
                            todo!("Apex?");
                            //let apex_mat = f.as_any().downcast_ref::<apex::filetypes::matl::Material>().unwrap();
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{:#?}", err);
            }
        }
    }
}
