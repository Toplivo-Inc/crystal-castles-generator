#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_variables)]
#![allow(dead_code)]

use image::ImageReader;
use macroquad::{miniquad::window::set_window_size, prelude::*};
use core::config;
use core::processor;
use std::cmp;
use std::fs;

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
    let config_content = fs::read_to_string("tests/configs/test1.json").unwrap();
    let config: config::Config = serde_json::from_str(&config_content).unwrap();
    let processed_image = processor::ImageProcessor::process(&config).unwrap();
    processor::ImageProcessor::save_image(&processed_image, &config.output).unwrap();

    let texture1 = Texture2D::from_any_image_file("tests/samples/anon.png");
    let texture2 = Texture2D::from_any_image_file("tests/outputs/test1_output.png");

    set_window_size(
        texture1.width() as u32 + texture2.width() as u32,
        cmp::max(texture1.height() as u32, texture2.height() as u32),
    );

    loop {
        clear_background(WHITE);

        draw_texture(&texture1, 0.0, 0.0, WHITE);
        draw_texture(&texture2, texture1.width(), 0.0, WHITE);

        next_frame().await;
    }
}
