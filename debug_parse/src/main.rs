use std::{fs::File, io::BufReader};

extern crate mdl;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    //let file = File::open("D:\\OriginGays\\Titanfall2\\vpk\\modelstst\\models\\error.mdl").unwrap();
    let file = File::open(&args[1]).unwrap();

    let mut cursor = std::io::Cursor::new(BufReader::new(file));

    let brih = mdl::StudioMdl::read(cursor.get_mut()).unwrap();

    println!("{:#?}", brih);
}
