use device::{decrement_timers_routine, Chip8};
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
    // read cli args
    let args = env::args().collect::<Vec<_>>();
    let rom_path_str = args.get(1).expect("Rom path not provided");
    let rom_path = PathBuf::from_str(&rom_path_str).expect("Malformed rom path");

    // init device
    let mut device = Chip8::new(rom_path).unwrap();
    let display = Arc::clone(&device.display);
    let delay_timer = Arc::clone(&device.delay_timer);
    let sound_timer = Arc::clone(&device.sound_timer);
    let down_keys = device.down_keys.clone();
    let released_keys = device.released_keys.clone();
    let keymap = device.keymap.clone();

    // start threads
    let timers_thread =
        thread::spawn(move || decrement_timers_routine(vec![delay_timer, sound_timer]));
    let device_thread = thread::spawn(move || device.run().unwrap());

    // await on execution
    display_draw(display, down_keys, released_keys, keymap).await;
    device_thread.join().unwrap();
    timers_thread.join().unwrap();
}
