use crate::device::{is_pixel_on, loc_to_idx, DISPLAY_H, DISPLAY_SIZE, DISPLAY_W};
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

pub async fn display_draw(display: Arc<Mutex<[u8; DISPLAY_SIZE]>>) {
    let tiles_w = DISPLAY_W as f32;
    let tiles_h = DISPLAY_H as f32;

    loop {
        clear_background(BLACK);

        let sw = screen_width();
        let sh = screen_height();

        let tw = sw / tiles_w;
        let th = sh / tiles_h;
        let dh = display.lock().unwrap();
        for x_i in 0..DISPLAY_W {
            for y_i in 0..DISPLAY_H {
                if let Some(&v) = dh.get(loc_to_idx(x_i, y_i)) {
                    draw_rectangle(
                        x_i as f32 * tw,
                        y_i as f32 * th,
                        tw,
                        th,
                        Color::from_rgba(v, v, v, u8::MAX),
                    );
                }
            }
        }
        drop(dh);

        next_frame().await
    }
}
