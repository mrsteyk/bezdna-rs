use std::{fs::File, io::BufReader, path::Path};

extern crate miles;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Invalid usage!")
    } else {
        let path = Path::new(&args[1]);
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
