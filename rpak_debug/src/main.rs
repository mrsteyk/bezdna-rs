use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

use byteorder::{ReadBytesExt, LE};
use rpak::{
    apex::{
        self,
        filetypes::{dtbl::ColumnData, stlt::SettingsItem},
    },
    FileEntry, RPakFile,
};

extern crate rpak;

mod util;

fn apex(rpak: &rpak::apex::RPakFile, guid_name: &HashMap<u64, String>) {
    println!("Apex mode");

    let decomp = rpak.decompressed.borrow();
    let mut cursor = std::io::Cursor::new(decomp.get_ref().as_slice());

    // LOAD EARLY RPAK
    let file = File::open(
        "D:\\SteamLibrary\\steamapps\\common\\Apex Legends\\paks\\Win64\\common_early.rpak",
    )
    .unwrap();
    let mut cursor_early = std::io::Cursor::new(BufReader::new(file));
    let early = apex::RPakFile::read(cursor_early.get_mut()).unwrap();

    let header = &rpak.header;
    println!("{} | {}\n", header.part_rpak, header.is_compressed());

    println!("{:#?}", header);

    println!(
        "StarPak: {}\nStarPak: {}\n",
        rpak.starpak,
        match rpak.starpak_opt.as_ref() {
            Some(v) => v,
            _ => "NONE",
        }
    );

    println!("Sections:");
    for i in 0..rpak.sections.len() {
        let sect = &rpak.sections[i];
        println!("{}: {:?}", i, sect);
    }

    println!("\nDataChunks:");
    for i in 0..rpak.data_chunks.len() {
        let chunk = &rpak.data_chunks[i];
        println!("{}: @{:016X} {:?}", i, rpak.seeks[i], chunk);
    }

    println!("\nFiles:");
    for file in &rpak.files {
        let real_name = if let Some(ret) = guid_name.get(&file.get_guid()) {
            ret.as_str()
        } else {
            ""
        };

        println!(
            "{}.{}.{:016X}.{:4} {:X?}",
            real_name,
            match file.get_name() {
                Some(v) => v,
                _ => "",
            },
            file.get_guid(),
            file.get_ext(),
            file,
        );

        match file.get_ext() {
            "ui" => {
                let rui = file
                    .as_any()
                    .downcast_ref::<rpak::apex::filetypes::rui::RUI>()
                    .unwrap();

                //println!("{}.{:016X}.ui | {}", rui.name, rui.get_guid(), real_name);

                println!("\tDesc@{:016X}", rui.get_desc_off());
                println!("\tUnk1@{:016X}", rui.unk1.2);
                println!("\tUnk2@{:016X}", rui.unk2.2);

                println!("\tArgClusters[{}]", rui.arg_clusters.len());
                for cluster in &rui.arg_clusters {
                    println!("\t\t{:?}", cluster);
                }
                println!("\tArgs[{}]", rui.args.len());
                for arg in &rui.args {
                    println!("\t\t{:?}", arg);
                }
            }
            "dtbl" => {
                let dtbl = file
                    .as_any()
                    .downcast_ref::<rpak::apex::filetypes::dtbl::DataTable>()
                    .unwrap();

                print!("\t");

                for column in &dtbl.column_names {
                    print!("{}\t", column);
                }

                println!();

                for row in &dtbl.data {
                    print!("\t");
                    for col in row {
                        match col {
                            ColumnData::String(v) => print!("\"{}\"", v),
                            ColumnData::Asset(v) => print!("$\"{}\"", v),
                            ColumnData::AssetNoPreCache(v) => print!("$\"{}\"", v),

                            ColumnData::Bool(v) => print!("{}", v),
                            ColumnData::Float(v) => print!("{}", v),
                            ColumnData::Int(v) => print!("{}", v),

                            ColumnData::Vector(v) => print!("{:?}", v),

                            ColumnData::Invalid(v) => todo!("Invalid: {}", v),
                        }
                        print!("\t");
                    }
                    println!();
                }
            }
            "stgs" => {
                let stgs = file
                    .as_any()
                    .downcast_ref::<rpak::apex::filetypes::stgs::Settings>()
                    .unwrap();

                // Rust Inc.
                let early_files = &early.files;
                if let Some(stlt_generic) =
                    early_files.iter().find(|x| x.get_guid() == stgs.stlt_hash)
                {
                    let stlt = stlt_generic
                        .as_any()
                        .downcast_ref::<rpak::apex::filetypes::stlt::SettingsLayout>()
                        .unwrap();

                    // for i in &stlt.items {
                    //     cursor.seek(SeekFrom::Start(stgs.unk8 + i.unk4 as u64)).unwrap();
                    //     match i.typ {
                    //         0 => {},
                    //         2 => {
                    //             println!("\t{}:\t{}", i.name, cursor.read_f32::<LE>().unwrap());
                    //         },
                    //         5 => {
                    //             let val = util::string_from_buf(cursor.get_mut());
                    //             println!("\t{}:\t{}", i.name, val);
                    //         }
                    //         v => {
                    //             panic!("{} is unk! {:X}", v, (stgs.unk8 + i.unk4 as u64));
                    //         }
                    //     }
                    // }
                    let defuakt = &SettingsItem {
                        typ: 0,
                        name: "BORK".to_owned(),
                        unk4: 0,
                    };
                    for i in 0..stgs.unk4C {
                        cursor
                            .seek(SeekFrom::Start(stgs.unk38 + 12 * i as u64))
                            .unwrap();
                        let unk0 = cursor.read_u16::<LE>().unwrap();
                        let unk2 = cursor.read_u16::<LE>().unwrap();
                        let unk4 = cursor.read_u32::<LE>().unwrap();
                        // next is the value...

                        let item = (&stlt.items)
                            .iter()
                            .find(|x| x.unk4 == unk4)
                            .unwrap_or(defuakt);
                        match unk2 {
                            0 => {
                                let val = cursor.read_u32::<LE>().unwrap();
                                println!("\t{}|{}:\tADD({}) | {:X}", unk0, &item.name, val, unk4);
                            }
                            1 => {
                                let val = cursor.read_u32::<LE>().unwrap();
                                println!("\t{}|{}:\tMUL({}) | {:X}", unk0, &item.name, val, unk4);
                            }
                            2 => {
                                let val = cursor.read_f32::<LE>().unwrap();
                                println!("\t{}|{}:\tADD({}f) | {:X}", unk0, &item.name, val, unk4);
                            }
                            3 => {
                                let val = cursor.read_f32::<LE>().unwrap();
                                println!("\t{}|{}:\tMUL({}f) | {:X}", unk0, &item.name, val, unk4);
                            }
                            4 => {
                                let val = cursor.read_u8().unwrap();
                                println!("\t{}|{}:\t{} | {:X}", unk0, &item.name, val, unk4);
                            }
                            5 => {
                                let val = cursor.read_f32::<LE>().unwrap();
                                println!("\t{}|{}:\t{}f | {:X}", unk0, &item.name, val, unk4);
                            }
                            6 => {
                                let off = cursor.read_u32::<LE>().unwrap();
                                cursor
                                    .seek(SeekFrom::Start(stgs.unk18 + off as u64))
                                    .unwrap();
                                let val = util::string_from_buf(&mut cursor);
                                println!("\t{}|{}:\t{} | {:X}", unk0, &item.name, val, unk4);
                            }
                            v => {
                                panic!("{} is unk! {:X}", v, cursor.stream_position().unwrap());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn tf2(rpak: &rpak::tf2::RPakFile) {
    println!("TF2 mode");

    // let decomp = rpak.decompressed.borrow();
    // let mut cursor = std::io::Cursor::new(decomp.get_ref().as_slice());

    let header = &rpak.header;
    println!("{} | {}\n", header.part_rpak, header.is_compressed());

    println!("{:#?}", header);

    println!("StarPak: {}\n", rpak.starpak);

    println!("Sections:");
    for i in 0..rpak.sections.len() {
        let sect = &rpak.sections[i];
        println!("{}: {:?}", i, sect);
    }

    println!("\nDataChunks:");
    for i in 0..rpak.data_chunks.len() {
        let chunk = &rpak.data_chunks[i];
        println!("{}: @{:016X} {:?}", i, rpak.seeks[i], chunk);
    }

    println!("\nFiles:");
    for file in &rpak.files {
        println!(
            "{}.{:016X}.{:4} {:X?}",
            match file.get_name() {
                Some(v) => v,
                _ => "",
            },
            file.get_guid(),
            file.get_ext(),
            file,
        );

        match file.get_ext() {
            "matl" => {
                let matl = file
                    .as_any()
                    .downcast_ref::<rpak::tf2::filetypes::matl::Material>()
                    .unwrap();

                println!(
                    "\tDesc@{:016X} | {:016X}",
                    matl.get_desc_off(),
                    matl.generic.desc_size
                );
                println!("\tSurface Properties: {}", matl.surface_props);
                println!("\tTextureRefs[{}]:", matl.texture_guids.len());
                for i in 0..matl.texture_guids.len() {
                    let ref_name = if matl.texture_guids.len()
                        <= rpak::tf2::filetypes::matl::TEXTURE_REFS.len()
                    {
                        rpak::tf2::filetypes::matl::TEXTURE_REFS[i]
                    } else {
                        ""
                    };
                    println!("\t\t0x{:016X} {}", matl.texture_guids[i], ref_name);
                }
            }
            // "ui" => {
            //     let rui = file
            //         .as_any()
            //         .downcast_ref::<rpak::apex::filetypes::rui::RUI>()
            //         .unwrap();

            //     //println!("{}.{:016X}.ui | {}", rui.name, rui.get_guid(), real_name);

            //     println!("\tDesc@{:016X}", rui.get_desc_off());
            //     println!("\tUnk1@{:016X}", rui.unk1.2);
            //     println!("\tUnk2@{:016X}", rui.unk2.2);

            //     println!("\tArgClusters[{}]", rui.arg_clusters.len());
            //     for cluster in &rui.arg_clusters {
            //         println!("\t\t{:?}", cluster);
            //     }
            //     println!("\tArgs[{}]", rui.args.len());
            //     for arg in &rui.args {
            //         println!("\t\t{:?}", arg);
            //     }
            // }
            // "dtbl" => {
            //     let dtbl = file
            //         .as_any()
            //         .downcast_ref::<rpak::apex::filetypes::dtbl::DataTable>()
            //         .unwrap();

            //     print!("\t");

            //     for column in &dtbl.column_names {
            //         print!("{}\t", column);
            //     }

            //     println!();

            //     for row in &dtbl.data {
            //         print!("\t");
            //         for col in row {
            //             match col {
            //                 ColumnData::String(v) => print!("\"{}\"", v),
            //                 ColumnData::Asset(v) => print!("$\"{}\"", v),
            //                 ColumnData::AssetNoPreCache(v) => print!("$\"{}\"", v),

            //                 ColumnData::Bool(v) => print!("{}", v),
            //                 ColumnData::Float(v) => print!("{}", v),
            //                 ColumnData::Int(v) => print!("{}", v),

            //                 ColumnData::Vector(v) => print!("{:?}", v),

            //                 ColumnData::Invalid(v) => todo!("Invalid: {}", v),
            //             }
            //             print!("\t");
            //         }
            //         println!();
            //     }
            // }
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Invalid usage!")
    } else {
        let path = Path::new(&args[1]);
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let file = File::open(path).unwrap();
        println!("stem: {}", file_stem);
        let mut cursor = std::io::Cursor::new(BufReader::new(file));

        match rpak::parse_rpak(cursor.get_mut()) {
            Ok(rpak) => {
                let drpak = rpak.as_any();

                // Borrow checker...
                {
                    print!("Writing decompressed... ");
                    let decomp = rpak.get_decompressed();
                    std::fs::write(args[1].to_owned() + ".raw", decomp.get_ref()).unwrap();
                    println!("ok");
                }

                let guid_name = {
                    let mut ret = rpak::predict_names(&*rpak, file_stem.to_owned());

                    if args.len() > 2 {
                        let file = File::open(&args[2]).unwrap();
                        let buf = BufReader::new(file);

                        buf.lines().for_each(|f| {
                            // doing the replace makes it look nicer...
                            let line = f.expect("Line brih").replace("\\", "/");
                            let hash = rpak::hash(line.clone());
                            //println!("{}", &line);
                            ret.insert(hash, line);
                        });
                    }

                    ret
                };

                if let Some(arpak) = drpak.downcast_ref::<rpak::apex::RPakFile>() {
                    apex(arpak, &guid_name)
                } else {
                    if let Some(trpak) = drpak.downcast_ref::<rpak::tf2::RPakFile>() {
                        tf2(trpak)
                    } else {
                        unimplemented!()
                    }
                }
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}
