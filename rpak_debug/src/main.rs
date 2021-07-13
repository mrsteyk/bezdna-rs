use std::{
    any::{Any, TypeId},
    ascii::AsciiExt,
    convert::TryInto,
    fs::File,
    io::BufReader,
};

use rpak::FileEntry;

extern crate rpak;

fn apex(rpak: &rpak::apex::RPakFile) {
    println!("Apex mode");

    let header = &rpak.header;
    println!("{}\n", header.part_rpak);

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
        match file.get_ext() {
            "ui" => {
                let rui = file
                    .as_any()
                    .downcast_ref::<rpak::apex::filetypes::rui::RUI>()
                    .unwrap();
                println!("{}.{}.ui", rui.name, rui.get_guid());

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
            _ => {
                println!(
                    "{}.{:016X}.{:4} {:X?}",
                    match file.get_name() {
                        Some(v) => v,
                        _ => "",
                    },
                    file.get_guid(),
                    file.get_ext(),
                    file
                );
            }
        }
    }
}

fn main() {
    let file =
        File::open("D:\\SteamLibrary\\steamapps\\common\\Apex Legends\\paks\\Win64\\ui.rpak")
            .unwrap();
    let mut cursor = std::io::Cursor::new(BufReader::new(file));

    //println!("{:#?}", rpak);
    if let Ok(rpak) = rpak::parse_rpak(cursor.get_mut()) {
        let drpak = rpak.as_any();
        if let Some(arpak) = drpak.downcast_ref::<rpak::apex::RPakFile>() {
            apex(arpak)
        } else {
            // tf2(drpak.downcast_ref::<rpak::tf2::RPakFile>().unwrap())
        }
    }
}
