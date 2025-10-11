#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_variables)]
#![allow(dead_code)]

use image::ImageReader;

use macroquad::{miniquad::window::set_window_size, prelude::*};

#[macroquad::main("window 1")]
async fn main() {
    let img = ImageReader::open("samples/anon.jpg")
        .expect("failed to open")
        .decode()
        .expect("failed to decode");
    
    // Convert to RGBA if needed
    let rgba_img = img.to_rgba8();
    let (image_width, image_height) = rgba_img.dimensions();
    
    // Get the raw bytes
    let decoded_bytes = rgba_img.into_raw();
    
    // Create texture from RGBA bytes
    let texture = Texture2D::from_rgba8(image_width as u16, image_height as u16, &decoded_bytes);
    
    set_window_size(image_width, image_height);
    
    loop {
        clear_background(WHITE);
        
        draw_texture(&texture, 0.0, 0.0, WHITE);
        
        next_frame().await;
    }
}