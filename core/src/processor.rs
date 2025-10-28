use crate::config::{Config, FilterOperation, Operation, Shadow, Stroke};
use anyhow::Result;
use image::{DynamicImage, ImageFormat, Rgba, imageops::colorops};
use imageproc::drawing;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn process(config: &Config) -> Result<DynamicImage> {
        let mut img = image::open(&config.input.source)?;

        println!("Loaded image: {}x{}", img.width(), img.height());

        for operation in &config.operations {
            println!("Apply {:?}", operation);
            img = Self::apply_operation(&img, operation)?;
        }

        Ok(img)
    }

    // TODO: create missing dirs in path
    pub fn save_image(
        img: &DynamicImage,
        output_config: &crate::config::OutputConfig,
    ) -> Result<()> {
        let format = Self::determine_format(output_config);

        match format {
            ImageFormat::Jpeg => {
                if let Some(quality) = output_config.quality {
                    // Для простоты используем стандартное сохранение
                    // В реальном приложении можно использовать библиотеку с поддержкой качества
                    img.save_with_format(&output_config.destination, ImageFormat::Jpeg)?;
                } else {
                    img.save(&output_config.destination)?;
                }
            }
            _ => {
                img.save(&output_config.destination)?;
            }
        }

        Ok(())
    }

    fn determine_format(output_config: &crate::config::OutputConfig) -> ImageFormat {
        if let Some(ref format) = output_config.format {
            match format.to_lowercase().as_str() {
                "jpeg" | "jpg" => ImageFormat::Jpeg,
                "png" => ImageFormat::Png,
                "gif" => ImageFormat::Gif,
                "bmp" => ImageFormat::Bmp,
                "ico" => ImageFormat::Ico,
                "tiff" | "tif" => ImageFormat::Tiff,
                "webp" => ImageFormat::WebP,
                _ => ImageFormat::from_path(&output_config.destination).unwrap_or(ImageFormat::Png),
            }
        } else {
            ImageFormat::from_path(&output_config.destination).unwrap_or(ImageFormat::Png)
        }
    }

    fn apply_operation(img: &DynamicImage, operation: &Operation) -> Result<DynamicImage> {
        match operation {
            // TODO: allow resize without saving the aspect ratio, maybe need add as a new resize option
            Operation::Resize {
                width,
                height,
                filter,
            } => {
                let filter_type = match filter.as_deref().unwrap_or("lanczos3") {
                    "nearest" => image::imageops::FilterType::Nearest,
                    "triangle" => image::imageops::FilterType::Triangle,
                    "catmull" => image::imageops::FilterType::CatmullRom,
                    "gaussian" => image::imageops::FilterType::Gaussian,
                    "lanczos3" => image::imageops::FilterType::Lanczos3,
                    _ => image::imageops::FilterType::Lanczos3,
                };
                Ok(img.resize(*width, *height, filter_type))
            }

            Operation::Overlay {
                image: overlay_path,
                x,
                y,
                opacity,
                blend_mode: _,
            } => {
                let overlay = image::open(overlay_path)?;
                let mut result = img.clone();

                if let Some(opacity_value) = opacity {
                    Self::overlay_with_opacity(&mut result, &overlay, *x, *y, *opacity_value);
                } else {
                    image::imageops::overlay(&mut result, &overlay, *x as i64, *y as i64);
                }
                Ok(result)
            }

            Operation::Filter(filter_op) => Self::apply_filter_operation(img, filter_op),

            Operation::Text {
                content,
                font,
                size,
                color,
                x,
                y,
                stroke,
                shadow,
            } => Self::draw_text(
                img,
                content,
                font,
                *size,
                color,
                *x,
                *y,
                stroke.as_ref(),
                shadow.as_ref(),
            ),
        }
    }

    fn apply_filter_operation(
        img: &DynamicImage,
        filter_op: &FilterOperation,
    ) -> Result<DynamicImage> {
        match filter_op {
            FilterOperation::Grain { intensity } => Ok(Self::add_grain(img, *intensity)),
            FilterOperation::Blur { radius } => Ok(img.blur(*radius)),
            FilterOperation::DoubleVision {
                offset_x,
                offset_y,
                opacity,
            } => Ok(Self::double_vision(img, *offset_x, *offset_y, *opacity)),
            FilterOperation::Vignette { intensity } => Ok(Self::vignette(img, *intensity)),
            FilterOperation::Sepia => Ok(Self::sepia(img)),
            FilterOperation::Brightness { value } => Ok(Self::brightness(img, *value)),
            FilterOperation::Contrast { value } => Ok(Self::contrast(img, *value)),
            FilterOperation::Saturation { value } => Ok(Self::saturation(img, *value)),
            FilterOperation::HueRotate { degrees } => Ok(Self::hue_rotate(img, *degrees)),
        }
    }

    fn overlay_with_opacity(
        base: &mut DynamicImage,
        overlay: &DynamicImage,
        x: i32,
        y: i32,
        opacity: f32,
    ) {
        let base_rgba = base.to_rgba8();
        let overlay_rgba = overlay.to_rgba8();
        let (overlay_width, overlay_height) = overlay_rgba.dimensions();

        let mut result = base_rgba;

        for ox in 0..overlay_width {
            for oy in 0..overlay_height {
                let base_x = x + ox as i32;
                let base_y = y + oy as i32;

                if base_x >= 0
                    && base_x < result.width() as i32
                    && base_y >= 0
                    && base_y < result.height() as i32
                {
                    let overlay_pixel = overlay_rgba.get_pixel(ox, oy);
                    let base_pixel = result.get_pixel(base_x as u32, base_y as u32);

                    if overlay_pixel[3] > 0 {
                        let alpha = (overlay_pixel[3] as f32 / 255.0) * opacity;

                        let blended = [
                            (base_pixel[0] as f32 * (1.0 - alpha) + overlay_pixel[0] as f32 * alpha)
                                as u8,
                            (base_pixel[1] as f32 * (1.0 - alpha) + overlay_pixel[1] as f32 * alpha)
                                as u8,
                            (base_pixel[2] as f32 * (1.0 - alpha) + overlay_pixel[2] as f32 * alpha)
                                as u8,
                            (base_pixel[3] as f32 * (1.0 - alpha) + overlay_pixel[3] as f32 * alpha)
                                as u8,
                        ];

                        result.put_pixel(base_x as u32, base_y as u32, Rgba(blended));
                    }
                }
            }
        }

        *base = DynamicImage::ImageRgba8(result);
    }

    // Реализации фильтров остаются такими же, как в предыдущей версии
    fn add_grain(img: &DynamicImage, intensity: f32) -> DynamicImage {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut result = img.to_rgba8();
        let (width, height) = result.dimensions();

        for x in 0..width {
            for y in 0..height {
                let mut pixel = *result.get_pixel(x, y);
                let noise = (rng.r#gen::<f32>() - 0.5) * intensity * 255.0;

                pixel[0] = (pixel[0] as f32 + noise).clamp(0.0, 255.0) as u8;
                pixel[1] = (pixel[1] as f32 + noise).clamp(0.0, 255.0) as u8;
                pixel[2] = (pixel[2] as f32 + noise).clamp(0.0, 255.0) as u8;

                result.put_pixel(x, y, pixel);
            }
        }

        DynamicImage::ImageRgba8(result)
    }

    fn double_vision(
        img: &DynamicImage,
        offset_x: i32,
        offset_y: i32,
        opacity: f32,
    ) -> DynamicImage {
        let mut result = img.clone().to_rgba8();
        let original = img.to_rgba8();
        let (width, height) = result.dimensions();

        for x in 0..width {
            for y in 0..height {
                let src_x = x as i32 + offset_x;
                let src_y = y as i32 + offset_y;

                if src_x >= 0 && src_x < width as i32 && src_y >= 0 && src_y < height as i32 {
                    let ghost_pixel = original.get_pixel(src_x as u32, src_y as u32);
                    let mut current_pixel = *result.get_pixel(x, y);

                    current_pixel[0] = (current_pixel[0] as f32 * (1.0 - opacity)
                        + ghost_pixel[0] as f32 * opacity)
                        as u8;
                    current_pixel[1] = (current_pixel[1] as f32 * (1.0 - opacity)
                        + ghost_pixel[1] as f32 * opacity)
                        as u8;
                    current_pixel[2] = (current_pixel[2] as f32 * (1.0 - opacity)
                        + ghost_pixel[2] as f32 * opacity)
                        as u8;

                    result.put_pixel(x, y, current_pixel);
                }
            }
        }

        DynamicImage::ImageRgba8(result)
    }

    fn vignette(img: &DynamicImage, intensity: f32) -> DynamicImage {
        let mut result = img.to_rgba8();
        let (width, height) = result.dimensions();
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_dist = (center_x * center_x + center_y * center_y).sqrt();

        for x in 0..width {
            for y in 0..height {
                let dx = center_x - x as f32;
                let dy = center_y - y as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let factor = 1.0 - (dist / max_dist) * intensity;

                let mut pixel = *result.get_pixel(x, y);
                pixel[0] = (pixel[0] as f32 * factor) as u8;
                pixel[1] = (pixel[1] as f32 * factor) as u8;
                pixel[2] = (pixel[2] as f32 * factor) as u8;

                result.put_pixel(x, y, pixel);
            }
        }

        DynamicImage::ImageRgba8(result)
    }

    fn sepia(img: &DynamicImage) -> DynamicImage {
        let mut result = img.to_rgba8();
        let (width, height) = result.dimensions();

        for x in 0..width {
            for y in 0..height {
                let mut pixel = *result.get_pixel(x, y);
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                let tr = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0);
                let tg = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0);
                let tb = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0);

                pixel[0] = tr as u8;
                pixel[1] = tg as u8;
                pixel[2] = tb as u8;

                result.put_pixel(x, y, pixel);
            }
        }

        DynamicImage::ImageRgba8(result)
    }

    fn brightness(img: &DynamicImage, value: f32) -> DynamicImage {
        let mut result = img.to_rgba8();
        colorops::brighten_in_place(&mut result, value as i32);
        DynamicImage::ImageRgba8(result)
    }

    fn contrast(img: &DynamicImage, value: f32) -> DynamicImage {
        let mut result = img.to_rgba8();
        colorops::contrast_in_place(&mut result, value);
        DynamicImage::ImageRgba8(result)
    }

    fn saturation(img: &DynamicImage, value: f32) -> DynamicImage {
        // Упрощенная реализация насыщенности
        let mut result = img.to_rgba8();
        let (width, height) = result.dimensions();

        for x in 0..width {
            for y in 0..height {
                let mut pixel = *result.get_pixel(x, y);
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                let gray = 0.299 * r + 0.587 * g + 0.114 * b;

                pixel[0] = (gray + (r - gray) * value).clamp(0.0, 255.0) as u8;
                pixel[1] = (gray + (g - gray) * value).clamp(0.0, 255.0) as u8;
                pixel[2] = (gray + (b - gray) * value).clamp(0.0, 255.0) as u8;

                result.put_pixel(x, y, pixel);
            }
        }

        DynamicImage::ImageRgba8(result)
    }

    fn hue_rotate(img: &DynamicImage, degrees: f32) -> DynamicImage {
        let mut result = img.to_rgba8();
        colorops::huerotate_in_place(&mut result, degrees as i32);
        DynamicImage::ImageRgba8(result)
    }

    fn draw_text(
        img: &DynamicImage,
        content: &str,
        font_path: &Path,
        size: f32,
        color: &str,
        x: i32,
        y: i32,
        stroke: Option<&Stroke>,
        shadow: Option<&Shadow>,
    ) -> Result<DynamicImage> {
        let mut result = img.to_rgba8();

        return Ok(DynamicImage::ImageRgba8(result)); // TODO: implement text, nw it doesnt work at all

        let font_data = std::fs::read(font_path)?;
        let font = ab_glyph::FontArc::try_from_vec(font_data)?;

        let scale = size;
        let color = Self::parse_color(color)?;

        if let Some(shadow) = shadow {
            let shadow_color = Self::parse_color(&shadow.color)?;
            drawing::draw_text_mut(
                &mut result,
                shadow_color,
                x + shadow.offset_x,
                y + shadow.offset_y,
                scale,
                &font,
                content,
            );
        }

        if let Some(stroke) = stroke {
            let stroke_color = Self::parse_color(&stroke.color)?;
            let stroke_width = stroke.width as i32;

            for dx in -stroke_width..=stroke_width {
                for dy in -stroke_width..=stroke_width {
                    if dx != 0 || dy != 0 {
                        drawing::draw_text_mut(
                            &mut result,
                            stroke_color,
                            x + dx,
                            y + dy,
                            scale,
                            &font,
                            content,
                        );
                    }
                }
            }
        }

        drawing::draw_text_mut(&mut result, color, x, y, scale, &font, content);

        Ok(DynamicImage::ImageRgba8(result))
    }

    fn parse_color(hex: &str) -> Result<Rgba<u8>> {
        let hex = hex.trim_start_matches('#');
        if hex.len() < 6 {
            return Err(anyhow::anyhow!("Invalid color format"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)?
        } else {
            255
        };

        Ok(Rgba([r, g, b, a]))
    }
}
