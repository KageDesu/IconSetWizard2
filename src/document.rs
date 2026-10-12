use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

pub const CELL: u32 = 32;
pub const COLS: usize = 16;
pub const MAX_ROWS: usize = 256;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Pixels {
    pub rgba: Vec<u8>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub pixels: Arc<Pixels>,
    pub visible: bool,
    pub opacity: u8,
    pub x: i32,
    pub y: i32,
    pub turns: u8,
    pub flip: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub angle: f32,
    #[serde(default = "unit_scale")]
    pub scale: f32,
    #[serde(default)]
    pub hue: f32,
    #[serde(default)]
    pub smooth: bool,
}

fn unit_scale() -> f32 {
    1.0
}

impl Layer {
    pub fn new(name: String, image: RgbaImage) -> Self {
        Self {
            name,
            pixels: Arc::new(Pixels {
                rgba: image.into_raw(),
            }),
            visible: true,
            opacity: 255,
            x: 0,
            y: 0,
            turns: 0,
            flip: false,
            flip_y: false,
            angle: 0.0,
            scale: 1.0,
            hue: 0.0,
            smooth: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub tiles: Vec<Vec<Layer>>,
}

impl Document {
    pub fn from_png(path: &Path) -> Result<Self, String> {
        let image = read_png(path)?;
        if image.width() != CELL * COLS as u32
            || image.height() == 0
            || image.height() % CELL != 0
            || image.height() / CELL > MAX_ROWS as u32
        {
            return Err(
                "Iconset должен иметь ширину 512 px и полные ряды по 32 px (до 256 рядов).".into(),
            );
        }
        let mut tiles = Vec::new();
        for y in 0..image.height() / CELL {
            for x in 0..COLS as u32 {
                let tile =
                    image::imageops::crop_imm(&image, x * CELL, y * CELL, CELL, CELL).to_image();
                tiles.push(vec![Layer::new("Оригинал".into(), tile)]);
            }
        }
        Ok(Self { version: 2, tiles })
    }

    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.version, 1 | 2)
            || self.tiles.is_empty()
            || !self.tiles.len().is_multiple_of(COLS)
            || self.tiles.len() > MAX_ROWS * COLS
        {
            return Err("Неподдерживаемая или некорректная сетка проекта.".into());
        }
        let mut bytes = 0usize;
        for tile in &self.tiles {
            if tile.len() > 128 {
                return Err("Слишком много слоёв в иконке.".into());
            }
            for layer in tile {
                bytes += layer.pixels.rgba.len();
                if layer.pixels.rgba.len() != (CELL * CELL * 4) as usize
                    || layer.turns > 3
                    || layer.x.abs_diff(0) > CELL
                    || layer.y.abs_diff(0) > CELL
                    || !layer.angle.is_finite()
                    || layer.angle.abs() > 360.0
                    || !layer.scale.is_finite()
                    || !(0.1..=4.0).contains(&layer.scale)
                    || !layer.hue.is_finite()
                    || layer.hue.abs() > 360.0
                {
                    return Err("Некорректные пиксели или параметры слоя.".into());
                }
            }
        }
        if bytes > 64 * 1024 * 1024 {
            return Err("Проект превышает лимит 64 MB пиксельных данных.".into());
        }
        Ok(())
    }

    pub fn render_tile(&self, index: usize) -> RgbaImage {
        let mut out = RgbaImage::new(CELL, CELL);
        for (number, layer) in self.tiles[index].iter().enumerate() {
            if !layer.visible {
                continue;
            }
            if number == 0
                && layer.opacity == 255
                && layer.x == 0
                && layer.y == 0
                && layer.turns == 0
                && !layer.flip
                && !layer.flip_y
                && layer.angle == 0.0
                && layer.scale == 1.0
                && layer.hue == 0.0
            {
                out = RgbaImage::from_raw(CELL, CELL, layer.pixels.rgba.clone()).unwrap();
                continue;
            }
            let radians = (layer.angle + f32::from(layer.turns) * 90.0).to_radians();
            let (sin, cos) = radians.sin_cos();
            for dy in 0..CELL {
                for dx in 0..CELL {
                    // Inverse mapping avoids holes when scaling up. Pixel centers rotate
                    // around the center of the 32x32 cell, with integer translation last.
                    let x = dx as f32 - layer.x as f32 - 15.5;
                    let y = dy as f32 - layer.y as f32 - 15.5;
                    let mut sx = (cos * x + sin * y) / layer.scale;
                    let mut sy = (-sin * x + cos * y) / layer.scale;
                    if layer.flip {
                        sx = -sx;
                    }
                    if layer.flip_y {
                        sy = -sy;
                    }
                    let mut src = sample(layer, sx + 15.5, sy + 15.5);
                    if layer.hue != 0.0 {
                        rotate_hue(&mut src, layer.hue);
                    }
                    let sa = u64::from(src[3]) * u64::from(layer.opacity);
                    if sa == 0 {
                        continue;
                    }
                    let dst = out.get_pixel_mut(dx, dy);
                    let da = u64::from(dst[3]) * 255;
                    let alpha = sa + (da * (65025 - sa) + 32512) / 65025;
                    for c in 0..3 {
                        let premul = u64::from(src[c]) * sa
                            + (u64::from(dst[c]) * da * (65025 - sa) + 32512) / 65025;
                        dst[c] = ((premul + alpha / 2) / alpha) as u8;
                    }
                    dst[3] = ((alpha + 127) / 255) as u8;
                }
            }
        }
        out
    }

    pub fn render(&self) -> RgbaImage {
        let mut out = RgbaImage::new(CELL * COLS as u32, CELL * (self.tiles.len() / COLS) as u32);
        for index in 0..self.tiles.len() {
            image::imageops::replace(
                &mut out,
                &self.render_tile(index),
                (index % COLS) as i64 * CELL as i64,
                (index / COLS) as i64 * CELL as i64,
            );
        }
        out
    }
}

fn sample(layer: &Layer, x: f32, y: f32) -> [u8; 4] {
    let pixel = |x: i32, y: i32| -> [u8; 4] {
        if !(0..32).contains(&x) || !(0..32).contains(&y) {
            return [0; 4];
        }
        let offset = ((y * 32 + x) * 4) as usize;
        layer.pixels.rgba[offset..offset + 4].try_into().unwrap()
    };
    if !layer.smooth {
        return pixel(x.round() as i32, y.round() as i32);
    }
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let mut sum = [0.0f32; 4];
    for (ox, oy, weight) in [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ] {
        let p = pixel(ix + ox, iy + oy);
        let a = p[3] as f32 * weight;
        for c in 0..3 {
            sum[c] += p[c] as f32 * a;
        }
        sum[3] += a;
    }
    if sum[3] <= 0.0 {
        return [0; 4];
    }
    [
        (sum[0] / sum[3]).round() as u8,
        (sum[1] / sum[3]).round() as u8,
        (sum[2] / sum[3]).round() as u8,
        sum[3].round() as u8,
    ]
}

fn rotate_hue(pixel: &mut [u8; 4], offset: f32) {
    let [r, g, b] = [
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
    ];
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    if chroma == 0.0 {
        return;
    }
    let hue = if max == r {
        ((g - b) / chroma).rem_euclid(6.0)
    } else if max == g {
        (b - r) / chroma + 2.0
    } else {
        (r - g) / chroma + 4.0
    };
    let h = (hue + offset / 60.0).rem_euclid(6.0);
    let second = chroma * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let rgb = match h.floor() as u8 {
        0 => [chroma, second, 0.0],
        1 => [second, chroma, 0.0],
        2 => [0.0, chroma, second],
        3 => [0.0, second, chroma],
        4 => [second, 0.0, chroma],
        _ => [chroma, 0.0, second],
    };
    for c in 0..3 {
        pixel[c] = ((rgb[c] + min) * 255.0).round() as u8;
    }
}

pub fn fit_cell(image: RgbaImage) -> RgbaImage {
    if image.width() == CELL && image.height() == CELL {
        image
    } else {
        image::imageops::resize(&image, CELL, CELL, image::imageops::FilterType::Nearest)
    }
}

pub fn decode_png(bytes: &[u8]) -> Result<RgbaImage, String> {
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    reader.limits(png_limits());
    reader
        .decode()
        .map(|i| i.to_rgba8())
        .map_err(|e| e.to_string())
}

fn png_limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    limits
}

pub fn read_png(path: &Path) -> Result<RgbaImage, String> {
    let reader = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    if reader.format() != Some(image::ImageFormat::Png) {
        return Err("Выберите изображение PNG.".into());
    }
    let mut reader = reader;
    reader.limits(png_limits());
    reader
        .decode()
        .map(|i| i.to_rgba8())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn hue_uses_blue_channel_preserves_alpha_and_grey() {
        let mut blue = [0, 0, 255, 123];
        rotate_hue(&mut blue, 120.0);
        assert_eq!(blue, [255, 0, 0, 123]);
        rotate_hue(&mut blue, -120.0);
        assert_eq!(blue, [0, 0, 255, 123]);
        let mut grey = [127, 127, 127, 80];
        rotate_hue(&mut grey, 73.0);
        assert_eq!(grey, [127, 127, 127, 80]);
    }

    #[test]
    fn inverse_transform_scale_arbitrary_rotation_and_vertical_flip() {
        let mut image = RgbaImage::new(CELL, CELL);
        image.put_pixel(15, 15, Rgba([255, 10, 20, 255]));
        let mut layer = Layer::new("pixel".into(), image);
        let original = layer.pixels.clone();
        layer.scale = 2.0;
        let mut doc = Document {
            version: 2,
            tiles: vec![vec![layer]],
        };
        assert_eq!(doc.render_tile(0).pixels().filter(|p| p[3] > 0).count(), 4);
        doc.tiles[0][0].scale = 1.0;
        doc.tiles[0][0].flip_y = true;
        assert_eq!(doc.render_tile(0)[(15, 16)].0, [255, 10, 20, 255]);
        doc.tiles[0][0].flip_y = false;
        doc.tiles[0][0].scale = 2.0;
        doc.tiles[0][0].angle = 45.0;
        assert!(doc.render_tile(0).pixels().any(|p| p[3] > 0));
        assert_eq!(doc.tiles[0][0].pixels.rgba, original.rgba);
    }

    #[test]
    fn smoothing_uses_premultiplied_alpha_without_dark_fringe() {
        let mut image = RgbaImage::new(CELL, CELL);
        image.put_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let mut layer = Layer::new("edge".into(), image);
        layer.smooth = true;
        assert_eq!(sample(&layer, 10.5, 10.0), [255, 0, 0, 128]);
        layer.smooth = false;
        assert_eq!(sample(&layer, 10.1, 10.0), [255, 0, 0, 255]);
    }

    #[test]
    fn quarter_turns_are_exact_with_new_angle_and_both_flips() {
        let image = RgbaImage::from_fn(CELL, CELL, |x, y| {
            Rgba([x as u8, y as u8, (x + y) as u8, 255])
        });
        let mut layer = Layer::new("palette".into(), image.clone());
        layer.angle = 90.0;
        let mut doc = Document {
            version: 2,
            tiles: vec![vec![layer]],
        };
        assert_eq!(doc.render_tile(0), image::imageops::rotate90(&image));
        doc.tiles[0][0].angle = 0.0;
        doc.tiles[0][0].flip = true;
        doc.tiles[0][0].flip_y = true;
        assert_eq!(doc.render_tile(0), image::imageops::rotate180(&image));
    }

    #[test]
    fn old_projects_default_new_properties_and_new_projects_roundtrip() {
        let doc = Document {
            version: 1,
            tiles: vec![vec![Layer::new("old".into(), RgbaImage::new(CELL, CELL))]; COLS],
        };
        let mut json = serde_json::to_value(&doc).unwrap();
        for tile in json["tiles"].as_array_mut().unwrap() {
            for layer in tile.as_array_mut().unwrap() {
                for field in ["flip_y", "angle", "scale", "hue", "smooth"] {
                    layer.as_object_mut().unwrap().remove(field);
                }
            }
        }
        let mut restored: Document = serde_json::from_value(json).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.tiles[0][0].scale, 1.0);
        restored.version = 2;
        restored.tiles[0][0].angle = 37.0;
        restored.tiles[0][0].hue = -92.0;
        restored.tiles[0][0].flip_y = true;
        let new: Document =
            serde_json::from_slice(&serde_json::to_vec(&restored).unwrap()).unwrap();
        new.validate().unwrap();
        assert!(new.tiles[0][0] == restored.tiles[0][0]);
        restored.tiles[0][0].scale = f32::NAN;
        assert!(restored.validate().is_err());
    }

    #[test]
    fn translucent_source_over_and_hidden_layers() {
        let mut tile = vec![
            Layer::new(
                "base".into(),
                RgbaImage::from_pixel(CELL, CELL, Rgba([0, 0, 255, 255])),
            ),
            Layer::new(
                "top".into(),
                RgbaImage::from_pixel(CELL, CELL, Rgba([255, 0, 0, 128])),
            ),
        ];
        let mut doc = Document {
            version: 1,
            tiles: vec![tile.clone(); COLS],
        };
        assert_eq!(doc.render_tile(0)[(0, 0)].0, [128, 0, 127, 255]);
        tile[1].visible = false;
        doc.tiles[0] = tile;
        assert_eq!(doc.render_tile(0)[(0, 0)].0, [0, 0, 255, 255]);
    }

    #[test]
    fn transform_clips_without_destroying_source_and_project_roundtrips() {
        let mut pixels = RgbaImage::new(CELL, CELL);
        pixels.put_pixel(0, 0, Rgba([255, 40, 20, 255]));
        let mut layer = Layer::new("corner".into(), pixels);
        layer.turns = 1;
        layer.x = 1;
        let mut doc = Document {
            version: 1,
            tiles: vec![vec![layer]; COLS],
        };
        assert_eq!(doc.render_tile(0)[(31, 0)][3], 0);
        doc.tiles[0][0].x = 0;
        assert_eq!(doc.render_tile(0)[(31, 0)].0, [255, 40, 20, 255]);
        let restored: Document =
            serde_json::from_slice(&serde_json::to_vec(&doc).unwrap()).unwrap();
        restored.validate().unwrap();
        assert_eq!(doc.render(), restored.render());
    }
}
