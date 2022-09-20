use macroquad::prelude::*;

pub async fn squares_demo() {
    let x = 63 as f32;
    let y = 31 as f32;
    let tiles_w = 64 as f32;
    let tiles_h = 32 as f32;

    
    loop {
        clear_background(BLACK);

        let sw = screen_width();
        let sh = screen_height();

        let tw = sw / tiles_w;
        let th = sh / tiles_h;
        for x_i in 0..64 {
            for y_i in 0..32 {
                if (x_i + y_i) % 2 == 0 {
                    draw_rectangle(x_i as f32 * tw, y_i as f32 * th, tw, th, WHITE);
                }
            }
        }

        next_frame().await
    }
}