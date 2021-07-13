use std::{fs::File, io::BufReader};

extern crate rpak;

fn main() {
    let file =
        File::open("D:\\SteamLibrary\\steamapps\\common\\Apex Legends\\paks\\Win64\\common.rpak")
            .unwrap();
    let mut cursor = std::io::Cursor::new(BufReader::new(file));

    let rpak = rpak::parse_rpak(cursor.get_mut());

    println!("{:#?}", rpak);
}
