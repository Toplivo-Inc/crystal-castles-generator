// src/config.rs
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
    Filter(FilterOperation),
    
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
#[serde(tag = "name")]
pub enum FilterOperation {
    #[serde(rename = "grain")]
    Grain {
        intensity: f32,
    },
    
    #[serde(rename = "blur")]
    Blur {
        radius: f32,
    },
    
    #[serde(rename = "double_vision")]
    DoubleVision {
        offset_x: i32,
        offset_y: i32,
        opacity: f32,
    },
    
    #[serde(rename = "vignette")]
    Vignette {
        intensity: f32,
    },
    
    #[serde(rename = "sepia")]
    Sepia,
    
    #[serde(rename = "brightness")]
    Brightness {
        value: f32,
    },
    
    #[serde(rename = "contrast")]
    Contrast {
        value: f32,
    },
    
    #[serde(rename = "saturation")]
    Saturation {
        value: f32,
    },
    
    #[serde(rename = "hue_rotate")]
    HueRotate {
        degrees: f32,
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