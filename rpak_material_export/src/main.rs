use std::{
    cmp::min,
    io::{Read, Seek, SeekFrom},
};
#[forbid(unsafe_code)]
use std::{fs::File, io::BufReader, path::Path};

use rpak::{apex, tf2};

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

    // UNK14
    decal_col: String,
    illum2: String,

    // >=UNK15
    decal_normal: String, // 15
    decal_mask: String,   // 16
    material_postprocessing: String,
    distort_normal: String,
    distort_normal2: String,
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

        // TODO: common_early logic in Apex...
        let early_file = File::open(
            path.parent()
                .unwrap_or(Path::new("."))
                .join("common_early.rpak"),
        );
        let mut early_cursor = if let Ok(cf) = &early_file {
            Some(std::io::Cursor::new(BufReader::new(cf)))
        } else {
            None
        };
        let early_rpak = if let Some(cr) = early_cursor.as_mut() {
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
                                //continue;
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

                            // lets export NOT all textures...
                            for i in 0..min(
                                tf2_mat.texture_guids.len(),
                                tf2::filetypes::matl::TEXTURE_REFS_SE2.len(),
                            ) {
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
                                            data.extend_from_slice(include_bytes!("bc6h_hdr"));
                                        }
                                        v => {
                                            panic!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
                                    data[0x14..0x14 + 4]
                                        .copy_from_slice(&(mipmap.size as u32).to_le_bytes());

                                    data.extend_from_slice(
                                        &decomp.get_ref()[mipmap.off as usize
                                            ..mipmap.off as usize + mipmap.size as usize],
                                    );

                                    std::fs::write(
                                        material_out_path.join(prefix_file(
                                            &tf2_mat.texture_guids,
                                            folder_str,
                                            i,
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
                                            data.extend_from_slice(include_bytes!("bc6h_hdr"));
                                        }
                                        v => {
                                            unreachable!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
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
                                        material_out_path.join(prefix_file(
                                            &tf2_mat.texture_guids,
                                            folder_str,
                                            i,
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
                                    return prefix_file(&tf2_mat.texture_guids, folder_str, i);
                                //txtr.get_name().unwrap().replace("\\", "/");
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
                                        return prefix_file(&tf2_mat.texture_guids, folder_str, i);
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

                            // UNK14
                            let (decal_col, illum2) = if ((tf2_mat.texture_guids.len() > 15)
                                && tf2_mat.texture_guids[15] != 0)
                                || ((tf2_mat.texture_guids.len() > 16)
                                    && tf2_mat.texture_guids[16] != 0)
                            {
                                (find_texture(14), "".to_owned())
                            } else {
                                ("".to_owned(), find_texture(14))
                            };

                            let decal_normal = find_texture(15);
                            let decal_mask = find_texture(16);
                            let material_postprocessing = find_texture(17);
                            let distort_normal = find_texture(18);
                            let distort_normal2 = find_texture(19);

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

                                decal_col,
                                illum2,

                                decal_normal,
                                decal_mask,
                                material_postprocessing,
                                distort_normal,
                                distort_normal2,
                            };
                            let mat_serial = serde_json::to_string_pretty(&matjs).unwrap();
                            std::fs::write(material_json_path, mat_serial).unwrap();
                        } else {
                            let apex_mat = f
                                .as_any()
                                .downcast_ref::<apex::filetypes::matl::Material>()
                                .unwrap();

                            // lets """assert""" before anything bad happens...
                            if apex_mat.texture_guids.len()
                                > tf2::filetypes::matl::TEXTURE_REFS_SE2.len()
                            {
                                eprintln!(
                                    "Material has too many textures! {} | {}",
                                    apex_mat.texture_guids.len(),
                                    apex_mat.name
                                );
                                continue;
                            }
                            let mut bad = false;
                            for guid in &apex_mat.texture_guids {
                                if *guid == 0 {
                                    continue; // no texture present for this thing
                                }
                                if let Some(atexture) = files.iter().find(|x| x.get_guid() == *guid)
                                {
                                    let texture = atexture
                                        .as_any()
                                        .downcast_ref::<apex::filetypes::txtr::Texture>() // TODO: texture type shit
                                        .unwrap();
                                    //assert_neq!(, "UNKNOWN", "Unknown compression type for {}", texture.texture_type)
                                    if apex::filetypes::txtr::TEXTURE_ALGOS
                                        [texture.texture_type as usize]
                                        == "UNKNOWN"
                                    {
                                        bad = true;
                                        eprintln!(
                                            "Material has bad compression type {} | {}",
                                            texture.texture_type, apex_mat.name
                                        );
                                        break;
                                    }
                                } else {
                                    bad = true;
                                    eprintln!(
                                        "Material has texture not in this RPak {:016X} | {}",
                                        guid, apex_mat.name
                                    );
                                    if let Some(common) = common_rpak.as_ref() {
                                        if common
                                            .get_files()
                                            .iter()
                                            .find(|x| x.get_guid() == *guid)
                                            .is_some()
                                        {
                                            bad = false;
                                            eprintln!(
                                                "Material has texture FROM COMMON {:016X} | {}",
                                                guid, apex_mat.name
                                            );
                                            continue;
                                        }
                                    }
                                    if bad {
                                        if let Some(common) = early_rpak.as_ref() {
                                            if common
                                                .get_files()
                                                .iter()
                                                .find(|x| x.get_guid() == *guid)
                                                .is_some()
                                            {
                                                bad = false;
                                                eprintln!(
                                                    "Material has texture FROM COMMON_EARLY {:016X} | {}",
                                                    guid, apex_mat.name
                                                );
                                                continue;
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                            if bad {
                                continue;
                            }
                            let folder_str = &apex_mat.name.replace("\\", "/");
                            //let folder = Path::new(&folder_str);
                            //let mat_str = material_out_path.file_name().unwrap(); // for SE2 shit
                            let mat_path = material_out_path.join(&folder_str);

                            std::fs::create_dir_all(mat_path).unwrap();

                            // lets export all textures...
                            for i in 0..apex_mat.texture_guids.len() {
                                let guid = &apex_mat.texture_guids[i];
                                if *guid == 0 {
                                    continue;
                                }
                                enum FromRPak {
                                    Current,
                                    Common,
                                    Early,
                                }
                                let mut is_from_where = FromRPak::Current;
                                let texture_any = if let Some(common) = &common_rpak {
                                    if let Some(tmp) = files.iter().find(|x| x.get_guid() == *guid)
                                    {
                                        tmp
                                    } else if let Some(tmp) =
                                        common.get_files().iter().find(|x| x.get_guid() == *guid)
                                    {
                                        is_from_where = FromRPak::Common;
                                        tmp
                                    } else {
                                        is_from_where = FromRPak::Early;
                                        early_rpak
                                            .as_ref()
                                            .unwrap()
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
                                    .downcast_ref::<apex::filetypes::txtr::Texture>()
                                    .unwrap();

                                let starpak_opt_exists = if starpak.is_some() {
                                    let p = Path::new(&args[3]);
                                    let pp = p.parent().unwrap_or(Path::new(".")).join(format!(
                                        "{}.opt.starpak",
                                        p.file_stem().unwrap().to_str().unwrap()
                                    ));
                                    eprintln!("Checking for {}", pp.to_str().unwrap());
                                    pp.exists()
                                } else {
                                    false
                                };

                                // This shit will trip on 100% optional textures but I doubt those exist...
                                if starpak.is_none()
                                    || texture.mipmaps.last().unwrap().typ
                                        == apex::filetypes::txtr::MipMapType::RPak
                                    || (texture
                                        .mipmaps
                                        .iter()
                                        .find(|x| {
                                            x.typ == apex::filetypes::txtr::MipMapType::StarPak
                                        })
                                        .is_none()
                                        && !starpak_opt_exists)
                                {
                                    let mipmap = texture
                                        .mipmaps
                                        .iter()
                                        .rev()
                                        .find(|x| x.typ == apex::filetypes::txtr::MipMapType::RPak)
                                        .unwrap();
                                    let decomp = match is_from_where {
                                        FromRPak::Current => drpak.get_decompressed(),
                                        FromRPak::Common => {
                                            common_rpak.as_ref().unwrap().get_decompressed()
                                        }
                                        FromRPak::Early => {
                                            early_rpak.as_ref().unwrap().get_decompressed()
                                        }
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
                                            data.extend_from_slice(include_bytes!("bc6h_hdr"));
                                        }
                                        v => {
                                            panic!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
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
                                    let mipmap = if starpak_opt_exists {
                                        texture.mipmaps.last().unwrap()
                                    } else {
                                        texture
                                            .mipmaps
                                            .iter()
                                            .rev()
                                            .find(|x| {
                                                x.typ == apex::filetypes::txtr::MipMapType::StarPak
                                            })
                                            .unwrap()
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
                                            data.extend_from_slice(include_bytes!("bc6h_hdr"));
                                        }
                                        v => {
                                            unreachable!("We should've not been here... {}", v);
                                        }
                                    }

                                    data[0xC..0xC + 4]
                                        .copy_from_slice(&(mipmap.height as u32).to_le_bytes());
                                    data[0x10..0x10 + 4]
                                        .copy_from_slice(&(mipmap.width as u32).to_le_bytes());
                                    data[0x14..0x14 + 4]
                                        .copy_from_slice(&(mipmap.size as u32).to_le_bytes());

                                    // or 0xFF???
                                    // We need to open .opt if we are optional mipmap
                                    if mipmap.off & 0xF != 0
                                        || mipmap.typ
                                            == apex::filetypes::txtr::MipMapType::StarPakOpt
                                    {
                                        let p = Path::new(&args[3]);
                                        let star_id = mipmap.off & 0xF;

                                        //eprintln!("WARNING: ALMOST DEPRECATED LOGIC! NEW APEX ONLY HAS PC_ALL NUMBERED UP TO A 1, WE WILL ATTEMPT {}", star_id);

                                        let starpak_name = if star_id != 0 {
                                            format!(
                                                "{}({:02}).{}starpak",
                                                p.file_stem().unwrap().to_str().unwrap(),
                                                star_id,
                                                if mipmap.typ
                                                    == apex::filetypes::txtr::MipMapType::StarPakOpt
                                                {
                                                    "opt."
                                                } else {
                                                    ""
                                                }
                                            )
                                        } else {
                                            format!(
                                                "{}.{}starpak",
                                                p.file_stem().unwrap().to_str().unwrap(),
                                                if mipmap.typ
                                                    == apex::filetypes::txtr::MipMapType::StarPakOpt
                                                {
                                                    "opt."
                                                } else {
                                                    ""
                                                }
                                            )
                                        };
                                        let fullpath = p.parent().unwrap().join(&starpak_name);
                                        eprintln!(
                                            "Filename: {} | {}",
                                            &starpak_name,
                                            fullpath.as_os_str().to_str().unwrap()
                                        );

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
                                if apex_mat.texture_guids.len() <= i {
                                    return "".to_owned();
                                }
                                let guid = apex_mat.texture_guids[i];
                                if let Some(txtr) = files.iter().find(|x| x.get_guid() == guid) {
                                    return format!(
                                        "{}/{}.dds",
                                        folder_str,
                                        tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
                                    )
                                    .to_owned(); //txtr.get_name().unwrap().replace("\\", "/");
                                } else {
                                    if let Some(early) = early_rpak.as_ref() {
                                        if early
                                            .get_files()
                                            .iter()
                                            .find(|x| x.get_guid() == guid)
                                            .is_some()
                                        {
                                            return format!(
                                                "{}/{}.dds",
                                                folder_str,
                                                apex::filetypes::matl::TEXTURE_REFS_SE2[i]
                                            )
                                            .to_owned();
                                        } else {
                                            return "".to_owned();
                                        }
                                    }
                                    if common_rpak.is_none() {
                                        return "".to_owned();
                                    }
                                    if common_rpak
                                        .as_ref()
                                        .unwrap()
                                        .get_files()
                                        .iter()
                                        .find(|x| x.get_guid() == guid)
                                        .is_some()
                                    {
                                        return format!(
                                            "{}/{}.dds",
                                            folder_str,
                                            apex::filetypes::matl::TEXTURE_REFS_SE2[i]
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
                                surface_properties: &apex_mat.surface_props,

                                color,
                                normal,
                                rough,
                                spec,
                                illumm,
                                bm,
                                ao,
                                cav,
                                opa,

                                // UNK14
                                decal_col: "".to_owned(),
                                illum2: "".to_owned(),

                                // >=UNK15
                                decal_normal: "".to_owned(), // 15
                                decal_mask: "".to_owned(),   // 16
                                material_postprocessing: "".to_owned(),
                                distort_normal: "".to_owned(),
                                distort_normal2: "".to_owned(),
                            };
                            let mat_serial = serde_json::to_string_pretty(&matjs).unwrap();
                            std::fs::write(material_json_path, mat_serial).unwrap();
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

fn prefix_file(guids: &Vec<u64>, folder_str: &str, i: usize) -> String {
    format!(
        "{}/{}.dds",
        folder_str,
        if i == 14 {
            if ((guids.len() > 15) && guids[15] != 0) || ((guids.len() > 16) && guids[16] != 0) {
                "_decal_col"
            } else {
                "_ilm2"
            }
        } else {
            tf2::filetypes::matl::TEXTURE_REFS_SE2[i]
        }
    )
}
