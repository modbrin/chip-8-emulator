use device::Chip8;
use graphics::display_draw;
use macroquad::window::Conf;
use std::{env, path::PathBuf, str::FromStr, sync::Arc, thread};

mod device;
mod graphics;
mod util;

fn window_conf() -> Conf {
    Conf {
        window_title: "CHIP-8 Emulator".to_owned(),
        fullscreen: false,
        window_height: 512,
        window_width: 1024,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = env::args().collect::<Vec<_>>();
    let rom_path_str = args.get(1).expect("Rom path not provided");
    let rom_path = PathBuf::from_str(&rom_path_str).expect("Malformed rom path");
    let mut device = Chip8::new(rom_path).unwrap();
    let display = Arc::clone(&device.display);
    let dh = thread::spawn(move || device.run().unwrap());
    display_draw(display).await;
    dh.join().unwrap();
}
