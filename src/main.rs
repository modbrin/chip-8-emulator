use device::Chip8;
use graphics::triangle_demo;
use std::{env, path::PathBuf, str::FromStr};

mod device;
mod util;
mod graphics;

fn main() {
    // let args = env::args().collect::<Vec<_>>();
    // let rom_path_str = args.get(1).expect("Rom path not provided");
    // let rom_path = PathBuf::from_str(&rom_path_str).expect("Malformed rom path");
    // Chip8::new(rom_path).unwrap().run().unwrap();
    triangle_demo();
}
