//! Overall desktop background: user image (cover/contain/stretch), solid, or a
//! two-stop vertical gradient. Always returns an opaque pixmap.

use image::imageops::FilterType;
use tiny_skia::Pixmap;

use crate::config::{rgb, Background, BgFit, BgMode, ZOOM_MAX, ZOOM_MIN};

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

/// Fit the picture into the canvas, then apply the user's zoom and pan.
///
/// Only the visible intersection is resampled, so a 4x zoom on a 4K wallpaper
/// costs one canvas-sized buffer instead of a 7680 px one. Uncovered pixels keep
/// the configured fallback colour, which is what `Contain`, a zoom below 1.0 and
/// an over-pan all leave behind.
fn render_image(bg: &Background, w: u32, h: u32) -> Option<Pixmap> {
    if bg.image_path.trim().is_empty() {
        return None;
    }
    let img = image::open(bg.image_path.trim()).ok()?.to_rgba8();
    let (iw, ih) = img.dimensions();
    if iw == 0 || ih == 0 {
        return None;
    }
    let w = w.max(1);
    let h = h.max(1);

    // 1. Base scale from `fit`. Only Stretch is allowed different axes.
    let (base_x, base_y) = match bg.fit {
        BgFit::Stretch => (w as f32 / iw as f32, h as f32 / ih as f32),
        BgFit::Cover => {
            let s = (w as f32 / iw as f32).max(h as f32 / ih as f32);
            (s, s)
        }
        BgFit::Contain => {
            let s = (w as f32 / iw as f32).min(h as f32 / ih as f32);
            (s, s)
        }
    };

    // 2. Zoom multiplies the drawn size on both axes. Cover must never shrink the
    //    picture inside the frame, or the wallpaper would show an empty edge.
    let zoom = if bg.zoom.is_finite() {
        bg.zoom.clamp(ZOOM_MIN, ZOOM_MAX)
    } else {
        1.0
    };
    let zoom = if matches!(bg.fit, BgFit::Cover) {
        zoom.max(1.0)
    } else {
        zoom
    };
    let draw_w = iw as f32 * base_x * zoom;
    let draw_h = ih as f32 * base_y * zoom;

    // 3. Pan slides across the slack the picture leaves; clamping the slack at zero
    //    means a fitted picture simply cannot expose a gap.
    let slack_x = ((draw_w - w as f32) * 0.5).max(0.0);
    let slack_y = ((draw_h - h as f32) * 0.5).max(0.0);
    let centre_x = w as f32 * 0.5 + bg.pan_x.clamp(-1.0, 1.0) * slack_x;
    let centre_y = h as f32 * 0.5 + bg.pan_y.clamp(-1.0, 1.0) * slack_y;
    let origin_x = centre_x - draw_w * 0.5;
    let origin_y = centre_y - draw_h * 0.5;

    // 4. Canvas starts as the fallback colour.
    let (fr, fg, fb) = rgb(&bg.color_primary, 0x101010);
    let mut canvas = image::RgbaImage::from_pixel(w, h, image::Rgba([fr, fg, fb, 255]));

    // 5. Blit the visible intersection, resampled once.
    let vis_x0 = origin_x.max(0.0);
    let vis_y0 = origin_y.max(0.0);
    let vis_x1 = (origin_x + draw_w).min(w as f32);
    let vis_y1 = (origin_y + draw_h).min(h as f32);
    if draw_w > 0.0 && draw_h > 0.0 && vis_x1 - vis_x0 > 0.5 && vis_y1 - vis_y0 > 0.5 {
        let u0 = ((vis_x0 - origin_x) / draw_w).clamp(0.0, 1.0);
        let u1 = ((vis_x1 - origin_x) / draw_w).clamp(0.0, 1.0);
        let v0 = ((vis_y0 - origin_y) / draw_h).clamp(0.0, 1.0);
        let v1 = ((vis_y1 - origin_y) / draw_h).clamp(0.0, 1.0);
        let sx = ((u0 * iw as f32).floor() as u32).min(iw - 1);
        let sy = ((v0 * ih as f32).floor() as u32).min(ih - 1);
        let sw = (((u1 - u0) * iw as f32).ceil() as u32).clamp(1, iw - sx);
        let sh = (((v1 - v0) * ih as f32).ceil() as u32).clamp(1, ih - sy);
        let crop = image::imageops::crop_imm(&img, sx, sy, sw, sh).to_image();
        let tw = ((vis_x1 - vis_x0).round() as u32).max(1);
        let th = ((vis_y1 - vis_y0).round() as u32).max(1);
        let fitted = image::imageops::resize(&crop, tw, th, FilterType::Triangle);
        alpha_over(
            &mut canvas,
            &fitted,
            vis_x0.round() as i64,
            vis_y0.round() as i64,
        );
    }

    // 6. tiny-skia stores premultiplied pixels; the canvas is opaque by now, so
    //    the copy is a straight one.
    let mut pm = Pixmap::new(w, h)?;
    let dst = pm.data_mut();
    for (i, px) in canvas.as_raw().as_chunks::<4>().0.iter().enumerate() {
        let o = i * 4;
        dst[o..o + 4].copy_from_slice(&[px[0], px[1], px[2], 255]);
    }
    Some(pm)
}

/// Source-over composite of `src` onto `dst` at (ox, oy), clipped to `dst`.
///
/// `image::imageops::replace` would copy transparent source pixels straight over
/// the fallback colour; the wallpaper has to stay opaque.
fn alpha_over(dst: &mut image::RgbaImage, src: &image::RgbaImage, ox: i64, oy: i64) {
    let (dw, dh) = (dst.width() as i64, dst.height() as i64);
    for (x, y, px) in src.enumerate_pixels() {
        let dx = ox + x as i64;
        let dy = oy + y as i64;
        if dx < 0 || dy < 0 || dx >= dw || dy >= dh {
            continue;
        }
        let a = px[3] as u32;
        if a == 255 {
            dst.put_pixel(dx as u32, dy as u32, *px);
        } else if a > 0 {
            let base = dst.get_pixel(dx as u32, dy as u32);
            let mix = |s: u8, d: u8| ((s as u32 * a + d as u32 * (255 - a)) / 255) as u8;
            dst.put_pixel(
                dx as u32,
                dy as u32,
                image::Rgba([
                    mix(px[0], base[0]),
                    mix(px[1], base[1]),
                    mix(px[2], base[2]),
                    255,
                ]),
            );
        }
    }
}

fn empty() -> Pixmap {
    Pixmap::new(1, 1).expect("1x1 pixmap")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Square fixture image, saved once per test.
    fn fixture(name: &str, size: u32, paint: impl Fn(u32, u32) -> [u8; 4]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let img = image::RgbaImage::from_fn(size, size, |x, y| image::Rgba(paint(x, y)));
        img.save(&path).expect("write fixture png");
        path
    }

    fn background(path: &std::path::Path, fit: BgFit) -> Background {
        Background {
            mode: BgMode::Image,
            image_path: path.display().to_string(),
            fit,
            // Red fallback so uncovered pixels are unambiguous.
            color_primary: "#ff0000".into(),
            color_secondary: "#00ff00".into(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    fn rgb_at(pm: &Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
        let px = pm.pixel(x, y).expect("pixel inside the pixmap");
        (px.red(), px.green(), px.blue())
    }

    #[test]
    fn zoom_fills_the_bars_contain_leaves() {
        let path = fixture("gittrack-framing-square.png", 400, |_, _| [0, 0, 255, 255]);
        let mut bg = background(&path, BgFit::Contain);

        // 400x400 into 200x100: contain draws 100x100 centred, so the sides show
        // the fallback colour.
        let scaled = render(&bg, 200, 100);
        assert_eq!(rgb_at(&scaled, 2, 50), (255, 0, 0), "left bar is fallback");
        assert_eq!(
            rgb_at(&scaled, 100, 50),
            (0, 0, 255),
            "centre is the picture"
        );

        bg.zoom = 2.0;
        let zoomed = render(&bg, 200, 100);
        assert_eq!(
            rgb_at(&zoomed, 2, 50),
            (0, 0, 255),
            "zoom covers the left bar"
        );
        assert_eq!(rgb_at(&zoomed, 197, 50), (0, 0, 255), "and the right bar");
    }

    #[test]
    fn cover_never_shrinks_below_the_canvas() {
        let path = fixture("gittrack-framing-cover.png", 400, |_, _| [0, 0, 255, 255]);
        let mut bg = background(&path, BgFit::Cover);
        // Below 1.0 the renderer clamps, and panning to the edge must still not
        // expose the fallback colour: cover means covered.
        bg.zoom = 0.5;
        for pan_y in [-1.0, 0.0, 1.0] {
            bg.pan_y = pan_y;
            let frame = render(&bg, 200, 100);
            for (x, y) in [(1, 1), (198, 1), (1, 98), (198, 98)] {
                assert_eq!(
                    rgb_at(&frame, x, y),
                    (0, 0, 255),
                    "pan_y {pan_y} exposed the backdrop at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn pan_shifts_the_picture() {
        // Left half blue, right half green: a horizontal shift is measurable.
        let path = fixture("gittrack-framing-pan.png", 400, |x, _| {
            if x < 200 {
                [0, 0, 255, 255]
            } else {
                [0, 255, 0, 255]
            }
        });
        let mut bg = background(&path, BgFit::Cover);
        bg.zoom = 2.0;
        let centred: Vec<(u8, u8, u8)> = (0..200u32)
            .map(|x| rgb_at(&render(&bg, 200, 100), x, 50))
            .collect();
        bg.pan_x = -1.0;
        let panned: Vec<(u8, u8, u8)> = (0..200u32)
            .map(|x| rgb_at(&render(&bg, 200, 100), x, 50))
            .collect();

        assert_ne!(centred, panned, "panning must change the rendered frame");
    }
}
