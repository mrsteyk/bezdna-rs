use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use rpak::FileEntry;

extern crate rpak;

fn apex(rpak: &rpak::apex::RPakFile, guid_name: &HashMap<u64, String>) {
    println!("Apex mode");

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
            "{}.{}.{:016X}.{:4} {:?}",
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

                print!("Writing decompressed... ");
                let decomp = rpak.get_decompressed();
                std::fs::write(args[1].to_owned() + ".raw", decomp.get_ref()).unwrap();
                println!("ok");

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
                    // tf2(drpak.downcast_ref::<rpak::tf2::RPakFile>().unwrap())
                    todo!()
                }
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}
