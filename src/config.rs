use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputConfig {
    pub source: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub destination: PathBuf,
    pub quality: Option<u8>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Operation {
    #[serde(rename = "resize")]
    Resize {
        width: u32,
        height: u32,
        filter: Option<String>,
    },
    
    #[serde(rename = "overlay")]
    Overlay {
        image: PathBuf,
        x: i32,
        y: i32,
        opacity: Option<f32>,
        blend_mode: Option<String>,
    },
    
    #[serde(rename = "filter")]
    Filter {
        name: String,
        intensity: Option<f32>,
        radius: Option<f32>,
        angle: Option<f32>,
        distance: Option<f32>,
        offset_x: Option<i32>,
        offset_y: Option<i32>,
        opacity: Option<f32>,
    },
    
    #[serde(rename = "text")]
    Text {
        content: String,
        font: PathBuf,
        size: f32,
        color: String,
        x: i32,
        y: i32,
        stroke: Option<Stroke>,
        shadow: Option<Shadow>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stroke {
    pub color: String,
    pub width: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shadow {
    pub color: String,
    pub blur: f32,
    pub offset_x: i32,
    pub offset_y: i32,
}
