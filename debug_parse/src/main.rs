use std::{fs::File, io::BufReader};

extern crate mdl;

fn main() {
    //let file = File::open("D:\\OriginGays\\Titanfall2\\vpk\\modelstst\\models\\error.mdl").unwrap();
    let file = File::open("D:\\OriginGays\\Titanfall2\\vpk\\modelstst\\models\\humans\\pilots\\pilot_medium_stalker_m.mdl").unwrap();

    let mut cursor = std::io::Cursor::new(BufReader::new(file));

    let brih = mdl::StudioModel::read(cursor.get_mut()).unwrap();

    println!("{:#?}", brih);
}
