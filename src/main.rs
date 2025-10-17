#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::cmp;
use image::ImageReader;
use macroquad::{miniquad::window::set_window_size, prelude::*};

pub trait FromAnyImageFile {
    fn from_any_image_file(fname: &str) -> Texture2D;
}

impl FromAnyImageFile for Texture2D {
    fn from_any_image_file(fname: &str) -> Texture2D {
        let img = ImageReader::open(fname)
            .expect("failed to open")
            .decode()
            .expect("failed to decode");

        let rgba_img = img.to_rgba8();
        let (image_width, image_height) = rgba_img.dimensions();
        let decoded_bytes = rgba_img.into_raw();
        Texture2D::from_rgba8(image_width as u16, image_height as u16, &decoded_bytes)
    }
}

#[macroquad::main("window 1")]
async fn main() {

    let texture1 = Texture2D::from_any_image_file("samples/do.png");
    let texture2 = Texture2D::from_any_image_file("samples/posle.png");
    
    set_window_size(texture1.width() as u32 + texture2.width() as u32, cmp::max(texture1.height() as u32, texture2.height() as u32));
    
    loop {
        clear_background(WHITE);
        
        draw_texture(&texture1, 0.0, 0.0, WHITE);
        draw_texture(&texture2, texture1.width(), 0.0, WHITE);
        
        next_frame().await;
    }
}