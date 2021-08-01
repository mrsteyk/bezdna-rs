use std::{fs::File, io::BufReader, path::Path};

use rpak::tf2;

use serde::{Serialize, Deserialize};

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

        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let file = File::open(path).unwrap();
        println!("stem: {}", file_stem);
        let mut cursor = std::io::Cursor::new(BufReader::new(file));
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
                                    break;
                                }
                            }
                            if bad {
                                continue;
                            }
                            let folder_str = tf2_mat.name.replace("\\", "/");
                            //let folder = Path::new(&folder_str);
                            //let mat_str = material_out_path.file_name().unwrap(); // for SE2 shit
                            let mat_path = material_out_path.join(&folder_str);

                            std::fs::create_dir_all(mat_path).unwrap();

                            let find_texture = |vec: &Vec<u64>, i: usize| {
                                if vec.len() <= i {
                                    return "".to_owned();
                                }
                                let guid = vec[i];
                                if let Some(txtr) = files.iter().find(|x| x.get_guid() == guid) {
                                    return txtr.get_name().unwrap().replace("\\", "/");
                                } else {
                                    return "".to_owned();
                                }
                            };

                            let material_json_str = folder_str + ".json";
                            let material_json_path = material_out_path.join(&material_json_str);

                            let color = find_texture(&tf2_mat.texture_guids, 0);
                            let normal = find_texture(&tf2_mat.texture_guids, 1);
                            let rough = find_texture(&tf2_mat.texture_guids, 2);
                            let spec = find_texture(&tf2_mat.texture_guids, 3);
                            let illumm = find_texture(&tf2_mat.texture_guids, 4);
                            let bm = find_texture(&tf2_mat.texture_guids, 8);
                            let ao = find_texture(&tf2_mat.texture_guids, 11);
                            let cav = find_texture(&tf2_mat.texture_guids, 12);
                            let opa = find_texture(&tf2_mat.texture_guids, 13);

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
                            std::fs::write(material_json_path, mat_serial);
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
