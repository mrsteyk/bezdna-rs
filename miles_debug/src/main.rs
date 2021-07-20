use std::{fs::{self, File}, io::BufReader, path::Path};

extern crate miles;

fn binka(path: &Path) {
    //let file = File::open(path).unwrap();
    //let mut cursor = std::io::Cursor::new(BufReader::new(file));
    let buf = fs::read(path).unwrap();

    let binka = miles::binka::BinkA::read(buf.as_slice());
    println!("{:#X?}", binka);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Invalid usage!")
    } else {
        let path = Path::new(&args[1]);
        if path.extension().unwrap().to_str().unwrap_or("") == "binka" {
            return binka(path);
        } else {
            let file = File::open(path).unwrap();
            let mut cursor = std::io::Cursor::new(BufReader::new(file));

            let miles_project = miles::tf2::MilesProject::read(cursor.get_mut());
            println!("{:#X?}", miles_project);

            if args.len() > 2 {
                let path = Path::new(&args[2]);
                let file = File::open(path).unwrap();
                let mut cursor = std::io::Cursor::new(BufReader::new(file));

                let miles_bank = miles::tf2::mbnk::MilesBank::read(cursor.get_mut());
                println!("---\n{:#X?}", miles_bank);
            }
        }
    }
}
