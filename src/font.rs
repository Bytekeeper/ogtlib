use crate::rect_pack::*;
use crate::sprite_batch::*;
use crate::texture::*;
use crate::{Color, Context};
use fontdue as fd;
use glam::{vec2, Vec2};
use image::{Rgba, RgbaImage};

struct Glyph {
    metrics: fd::Metrics,
    sprite: Region,
}

pub struct Font {
    texture: Texture,
    glyphs: Vec<Glyph>,
}

impl Font {
    pub fn draw_text(
        &self,
        context: &Context,
        batch: &mut SpriteBatch,
        txt: &str,
        pos: Vec2,
        color: Color,
    ) {
        batch.set_texture(self.texture.clone());
        let mut c_pos = pos;
        for c in txt.chars() {
            let glyph = &self.glyphs[c as usize - 32];
            batch.add(
                context,
                glyph.sprite,
                color,
                Vec2::ZERO,
                c_pos + vec2(glyph.metrics.xmin as f32, glyph.metrics.ymin as f32),
            );
            c_pos.x += glyph.metrics.advance_width;
            c_pos.y += glyph.metrics.advance_height;
        }
    }

    pub fn measure(&self, txt: &str) -> (Vec2, Vec2) {
        let mut width = 0.0;
        let mut height = 0.0;
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        let mut min_width: f32 = 0.0;
        let mut min_height: f32 = 0.0;
        for g in txt.chars().map(|c| &self.glyphs[c as usize - 32]) {
            let metrics = &g.metrics;
            min_width = min_width.min(width + metrics.xmin as f32);
            min_height = min_height.min(height + metrics.ymin as f32);
            max_width = max_width.max(width + metrics.xmin as f32 + metrics.width as f32);
            max_height = max_height.max(height + metrics.ymin as f32 + metrics.height as f32);
            width += metrics.advance_width;
            height += metrics.advance_height;
        }
        (vec2(min_width, min_height), vec2(max_width, max_height))
    }
}

pub struct LoadedFont {
    //line_metrics: fd::LineMetrics,
    font: fd::Font,
}

impl LoadedFont {
    pub fn from_bytes(data: &[u8]) -> Self {
        let font = fd::Font::from_bytes(data, fd::FontSettings::default()).unwrap();
        Self { font }
    }

    pub fn create_font(&self, context: &Context, size: f32) -> Font {
        let rasterized_chars: Vec<_> = (32..255_u8)
            .map(|c| self.font.rasterize(c as char, size))
            .collect();
        let mut rects: Vec<_> = rasterized_chars
            .iter()
            .map(|c| Rect::wh(c.0.width as u32 + 2, c.0.height as u32 + 2))
            .collect();
        let dim = pack(&mut rects, 4096).unwrap();
        let mut img = RgbaImage::new(dim.0, dim.1);
        for (c, r) in rasterized_chars.iter().zip(rects.iter()) {
            for (p, &v) in c.1.iter().enumerate() {
                img.put_pixel(
                    r.x + p as u32 % c.0.width as u32 + 1,
                    r.y + p as u32 / c.0.width as u32 + 1,
                    Rgba([255, 255, 255, v]),
                );
            }
        }
        let w = img.width();
        let h = img.height();
        let texture = TextureBuilder::from_bytes(&img.into_raw(), w, h).build(context);
        let glyphs = rasterized_chars
            .iter()
            .zip(rects)
            .map(|(c, r)| Glyph {
                metrics: c.0,
                sprite: Region {
                    top_left: [r.x as f32 + 1.0, r.y as f32 + 1.0],
                    bottom_right: [(r.x + r.width - 1) as f32, (r.y + r.height - 1) as f32],
                },
            })
            .collect();
        Font { texture, glyphs }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use glam::uvec2;

    #[test]
    fn test() {
        LoadedFont::from_bytes(include_bytes!("../examples/Hack-Regular.ttf"));
    }
}
