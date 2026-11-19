//! Card material: backdrop blur of the cropped background region, tint at the
//! configured opacity, deterministic grain for acrylic, light border for glass.
//!
//! Blur is a 3-pass box blur: O(N) per pass, no allocation per row beyond two
//! scratch buffers, and visually close enough to StackBlur for a wallpaper.

use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::config::MaterialKind;

/// Clamped index helper for edge-extended blur windows.
fn at(i: isize, len: usize) -> usize {
    i.clamp(0, len as isize - 1) as usize
}

/// Composite the card backdrop into `pm`.
///
/// * `rect` – card bounds in pixels (x, y, w, h)
/// * `blur_px` – already scaled by the preview factor
#[allow(clippy::too_many_arguments)]
pub fn apply_card(
    pm: &mut Pixmap,
    rect: (f32, f32, f32, f32),
    radius: f32,
    kind: MaterialKind,
    tint: (u8, u8, u8),
    opacity: f32,
    blur_px: f32,
    seed: u64,
) {
    if matches!(kind, MaterialKind::None) {
        return;
    }
    let pw = pm.width() as i64;
    let ph = pm.height() as i64;
    let x0 = rect.0.floor().max(0.0) as i64;
    let y0 = rect.1.floor().max(0.0) as i64;
    let x1 = (rect.0 + rect.2).ceil().min(pw as f32) as i64;
    let y1 = (rect.1 + rect.3).ceil().min(ph as f32) as i64;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let (rw, rh) = ((x1 - x0) as usize, (y1 - y0) as usize);

    // Crop the region that sits behind the card.
    let stride = pw as usize * 4;
    let mut region = vec![0u8; rw * rh * 4];
    {
        let src = pm.data();
        for row in 0..rh {
            let s = (y0 as usize + row) * stride + x0 as usize * 4;
            let d = row * rw * 4;
            region[d..d + rw * 4].copy_from_slice(&src[s..s + rw * 4]);
        }
    }

    let blur_amount = match kind {
        MaterialKind::Solid => 0,
        MaterialKind::Mica => (blur_px * 0.45).round() as usize,
        MaterialKind::Acrylic => (blur_px * 1.15).round() as usize,
        _ => blur_px.round() as usize,
    };
    if blur_amount >= 1 {
        box_blur_rgba(&mut region, rw, rh, blur_amount);
    }
    match kind {
        MaterialKind::Acrylic => add_noise(&mut region, seed, 10),
        MaterialKind::Mica => desaturate(&mut region, 0.30),
        _ => {}
    }

    // Composite tint over the (blurred) backdrop, masked by the rounded rect.
    let opacity = opacity.clamp(0.0, 1.0);
    let tint_a = if matches!(kind, MaterialKind::Solid) {
        opacity.max(0.85)
    } else {
        opacity
    };
    {
        let dst = pm.data_mut();
        for row in 0..rh {
            let py = y0 as f32 + row as f32 + 0.5;
            for col in 0..rw {
                let px = x0 as f32 + col as f32 + 0.5;
                let cov = coverage(px, py, rect, radius);
                if cov <= 0.0 {
                    continue;
                }
                let i = (row * rw + col) * 4;
                let (sr, sg, sb) = (region[i] as f32, region[i + 1] as f32, region[i + 2] as f32);
                let out = (
                    sr + (tint.0 as f32 - sr) * tint_a,
                    sg + (tint.1 as f32 - sg) * tint_a,
                    sb + (tint.2 as f32 - sb) * tint_a,
                );
                let o = ((y0 as usize + row) * stride) + (x0 as usize + col) * 4;
                for (k, v) in [out.0, out.1, out.2].iter().enumerate() {
                    let orig = dst[o + k] as f32;
                    dst[o + k] = (orig + (v - orig) * cov).round().clamp(0.0, 255.0) as u8;
                }
                dst[o + 3] = 255;
            }
        }
    }

    if matches!(kind, MaterialKind::Glass) {
        draw_border(pm, rect, radius);
    }
}

pub(crate) fn draw_border(pm: &mut Pixmap, rect: (f32, f32, f32, f32), radius: f32) {
    let Some(path) = rounded_rect_path(rect.0, rect.1, rect.2, rect.3, radius) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(255, 255, 255, 52));
    paint.anti_alias = true;
    let stroke = Stroke {
        width: 1.2,
        ..Default::default()
    };
    pm.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

pub fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let r = r.clamp(0.0, w.min(h) / 2.0);
    let mut pb = PathBuilder::new();
    if r < 0.01 {
        let rect = tiny_skia::Rect::from_xywh(x, y, w, h)?;
        pb.push_rect(rect);
    } else {
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
    }
    pb.finish()
}

/// Anti-aliased coverage of a rounded rectangle at a pixel centre.
pub fn coverage(px: f32, py: f32, rect: (f32, f32, f32, f32), radius: f32) -> f32 {
    let (x, y, w, h) = rect;
    let r = radius.clamp(0.0, w.min(h) / 2.0);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let qx = (px - cx).abs() - (w / 2.0 - r);
    let qy = (py - cy).abs() - (h / 2.0 - r);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let sdf = outside + qx.max(qy).min(0.0) - r;
    (0.5 - sdf).clamp(0.0, 1.0)
}

fn box_blur_rgba(data: &mut [u8], w: usize, h: usize, radius: usize) {
    if w == 0 || h == 0 || radius == 0 {
        return;
    }
    // Three box passes approximate a Gaussian of sigma ~= radius / 2.
    let passes = 3;
    let r = radius.max(1);
    let mut scratch = vec![0u8; data.len()];
    for _ in 0..passes {
        blur_rows(data, &mut scratch, w, h, r);
        blur_cols(&scratch, data, w, h, r);
    }
}

fn blur_rows(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize) {
    let win = (2 * r + 1) as u32;
    let r = r as isize;
    for row in 0..h {
        let base = row * w * 4;
        for ch in 0..3 {
            let mut sum: u32 = 0;
            for k in -r..=r {
                let x = at(k, w);
                sum += src[base + x * 4 + ch] as u32;
            }
            for x in 0..w {
                dst[base + x * 4 + ch] = (sum / win) as u8;
                let sub = at(x as isize - r, w);
                let add = at(x as isize + r + 1, w);
                sum -= src[base + sub * 4 + ch] as u32;
                sum += src[base + add * 4 + ch] as u32;
            }
        }
        for x in 0..w {
            dst[base + x * 4 + 3] = src[base + x * 4 + 3];
        }
    }
}

fn blur_cols(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize) {
    let win = (2 * r + 1) as u32;
    let r = r as isize;
    for col in 0..w {
        for ch in 0..3 {
            let mut sum: u32 = 0;
            for k in -r..=r {
                let y = at(k, h);
                sum += src[(y * w + col) * 4 + ch] as u32;
            }
            for y in 0..h {
                dst[(y * w + col) * 4 + ch] = (sum / win) as u8;
                let sub = at(y as isize - r, h);
                let add = at(y as isize + r + 1, h);
                sum -= src[(sub * w + col) * 4 + ch] as u32;
                sum += src[(add * w + col) * 4 + ch] as u32;
            }
        }
        for y in 0..h {
            dst[(y * w + col) * 4 + 3] = src[(y * w + col) * 4 + 3];
        }
    }
}

/// Deterministic monochrome grain, +-`amount`/255 around the base colour.
fn add_noise(data: &mut [u8], seed: u64, amount: i32) {
    let mut state = seed | 1;
    for px in data.as_chunks_mut::<4>().0 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let n = ((state >> 24) as i32 % (amount * 2 + 1)) - amount;
        for c in px.iter_mut().take(3) {
            *c = (*c as i32 + n).clamp(0, 255) as u8;
        }
    }
}

fn desaturate(data: &mut [u8], factor: f32) {
    for px in data.as_chunks_mut::<4>().0 {
        let l = 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
        for c in px.iter_mut().take(3) {
            let v = *c as f32;
            *c = (v + (l - v) * factor).clamp(0.0, 255.0) as u8;
        }
    }
}
