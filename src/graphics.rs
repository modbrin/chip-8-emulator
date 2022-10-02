use crate::device::{loc_to_idx, DISPLAY_H, DISPLAY_SIZE, DISPLAY_W};
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

const BORDER_OFFSET_PERCENT: u8 = 5;

pub async fn display_draw(display: Arc<Mutex<[u8; DISPLAY_SIZE]>>) {
    let tiles_w = DISPLAY_W as f32;
    let tiles_h = DISPLAY_H as f32;
    let offset = BORDER_OFFSET_PERCENT as f32 / 100.0;

    loop {
        clear_background(BLACK);

        let sw = screen_width();
        let sh = screen_height();
        let tw = sw / tiles_w;
        let th = sh / tiles_h;
        let sw_off = tw * offset;
        let sh_off = th * offset;

        let display_handle = display.lock().unwrap();
        let display_state = display_handle.clone();
        drop(display_handle); // minimize time holding display lock
        for x_i in 0..DISPLAY_W {
            for y_i in 0..DISPLAY_H {
                if let Some(&v) = display_state.get(loc_to_idx(x_i, y_i)) {
                    draw_rectangle(
                        x_i as f32 * tw + sw_off,
                        y_i as f32 * th + sh_off,
                        tw - sw_off,
                        th - sh_off,
                        Color::from_rgba(v, v, v, u8::MAX),
                    );
                }
            }
        }
        // println!("FPS: {:.1}", get_fps());
        next_frame().await
    }
}
