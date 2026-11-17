//! Overall desktop background: user image (cover/contain/stretch), solid, or a
//! two-stop vertical gradient. Always returns an opaque pixmap.

use image::{imageops::FilterType, GenericImageView};
use tiny_skia::Pixmap;

use crate::config::{rgb, Background, BgFit, BgMode};

pub fn render(bg: &Background, w: u32, h: u32) -> Pixmap {
    match bg.mode {
        BgMode::Image => render_image(bg, w, h).unwrap_or_else(|| gradient(bg, w, h)),
        BgMode::Solid => solid(bg, w, h),
        BgMode::Gradient => gradient(bg, w, h),
    }
}

fn solid(bg: &Background, w: u32, h: u32) -> Pixmap {
    let mut pm = Pixmap::new(w.max(1), h.max(1)).unwrap_or_else(empty);
    let (r, g, b) = rgb(&bg.color_primary, 0x101010);
    let mut data = vec![0u8; (w as usize) * (h as usize) * 4];
    for px in data.as_chunks_mut::<4>().0 {
        px.copy_from_slice(&[r, g, b, 255]);
    }
    pm.data_mut().copy_from_slice(&data);
    pm
}

fn gradient(bg: &Background, w: u32, h: u32) -> Pixmap {
    let mut pm = Pixmap::new(w.max(1), h.max(1)).unwrap_or_else(empty);
    let (r1, g1, b1) = rgb(&bg.color_primary, 0x0b1220);
    let (r2, g2, b2) = rgb(&bg.color_secondary, 0x12261c);
    let hh = h.max(1) as f32;
    let mut data = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        let t = (y as f32 / (hh - 1.0).max(1.0)).clamp(0.0, 1.0);
        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
        let px = [mix(r1, r2), mix(g1, g2), mix(b1, b2), 255];
        let row = (y as usize) * (w as usize) * 4;
        for x in 0..(w as usize) {
            let i = row + x * 4;
            data[i..i + 4].copy_from_slice(&px);
        }
    }
    pm.data_mut().copy_from_slice(&data);
    pm
}

fn render_image(bg: &Background, w: u32, h: u32) -> Option<Pixmap> {
    if bg.image_path.trim().is_empty() {
        return None;
    }
    let img = image::open(bg.image_path.trim()).ok()?;
    let (iw, ih) = img.dimensions();
    if iw == 0 || ih == 0 {
        return None;
    }
    let w = w.max(1);
    let h = h.max(1);

    let canvas: image::RgbaImage = match bg.fit {
        BgFit::Stretch => img.resize_exact(w, h, FilterType::Triangle).to_rgba8(),
        BgFit::Cover => img.resize_to_fill(w, h, FilterType::Triangle).to_rgba8(),
        BgFit::Contain => {
            let scaled = img.resize(w, h, FilterType::Triangle).to_rgba8();
            let (sw, sh) = scaled.dimensions();
            let (r, g, b) = rgb(&bg.color_primary, 0x101010);
            let mut canvas = image::RgbaImage::from_pixel(w, h, image::Rgba([r, g, b, 255]));
            let ox = ((w - sw) / 2) as i64;
            let oy = ((h - sh) / 2) as i64;
            for (x, y, p) in scaled.enumerate_pixels() {
                let (dx, dy) = (x as i64 + ox, y as i64 + oy);
                if dx >= 0 && dy >= 0 && (dx as u32) < w && (dy as u32) < h {
                    canvas.put_pixel(dx as u32, dy as u32, *p);
                }
            }
            canvas
        }
    };

    let mut pm = Pixmap::new(w, h)?;
    let src = canvas.as_raw();
    let dst = pm.data_mut();
    for (i, px) in src.as_chunks::<4>().0.iter().enumerate() {
        let a = px[3] as u32;
        let o = i * 4;
        if a == 255 {
            dst[o..o + 4].copy_from_slice(px);
        } else {
            // Premultiply for tiny-skia's storage format.
            dst[o] = ((px[0] as u32 * a) / 255) as u8;
            dst[o + 1] = ((px[1] as u32 * a) / 255) as u8;
            dst[o + 2] = ((px[2] as u32 * a) / 255) as u8;
            dst[o + 3] = a as u8;
        }
    }
    Some(pm)
}

fn empty() -> Pixmap {
    Pixmap::new(1, 1).expect("1x1 pixmap")
}
