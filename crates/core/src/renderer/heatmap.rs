//! 53x7 contribution grid, plus the optional stats header text.
//!
//! Text is rasterised from the system font (`C:\Windows\Fonts\segoeui.ttf`) so
//! no font blob is shipped; if it is missing the header is simply skipped.

use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use tiny_skia::{Color, FillRule, Paint, Pixmap, Transform};

use super::material::rounded_rect_path;
use super::{stats_of, Layout, Scene, COLS, ROWS};
use crate::config::rgb;

const FONT_CANDIDATES: &[&str] = &[
    r"C:\Windows\Fonts\segoeui.ttf",
    r"C:\Windows\Fonts\segoeuib.ttf",
    r"C:\Windows\Fonts\arial.ttf",
];

static FONT_DATA: OnceLock<Option<Vec<u8>>> = OnceLock::new();

fn font_bytes() -> Option<&'static [u8]> {
    FONT_DATA
        .get_or_init(|| FONT_CANDIDATES.iter().find_map(|p| std::fs::read(p).ok()))
        .as_deref()
}

pub fn draw(pm: &mut Pixmap, scene: &Scene<'_>, l: &Layout) {
    let colors: Vec<Color> = (0..5)
        .map(|i| {
            let (r, g, b) = rgb(&scene.config.track.colors[i], 0x161b22);
            Color::from_rgba8(r, g, b, 255)
        })
        .collect();

    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };

    let days = scene.days;
    let slots = COLS * ROWS;
    for i in 0..slots {
        let level = days.get(i).map(|d| d.level.min(4) as usize).unwrap_or(0);
        let col = (i / ROWS) as f32;
        let row = (i % ROWS) as f32;
        let x = l.grid_x + col * (l.box_size + l.gap);
        let y = l.grid_y + row * (l.box_size + l.gap);
        let radius = scene.config.track.box_radius * scene.scale;
        let Some(path) = rounded_rect_path(x, y, l.box_size, l.box_size, radius) else {
            continue;
        };
        paint.set_color(colors[level]);
        pm.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    if scene.config.track.show_stats_header && l.header_h > 0.0 {
        let stats = stats_of(days);
        let (r, g, b) = rgb(&scene.config.track.text_color, 0xe6edf3);
        let baseline = l.card_y + 22.0 * scene.scale + l.font_size;
        let label = format!(
            "{} contributions in the last year   -   {} day streak   -   best {} in a day",
            stats.total, stats.streak, stats.best_day
        );
        draw_text(pm, l.grid_x, baseline, l.font_size, (r, g, b), 0.92, &label);
    }
}

/// Blend one anti-aliased glyph run into an opaque pixmap.
fn draw_text(
    pm: &mut Pixmap,
    x: f32,
    baseline: f32,
    size: f32,
    color: (u8, u8, u8),
    alpha: f32,
    text: &str,
) {
    let Some(data) = font_bytes() else {
        return;
    };
    let Ok(font) = FontRef::try_from_slice(data) else {
        return;
    };
    let scaled = font.as_scaled(PxScale::from(size));
    let mut caret = x;
    let w = pm.width() as i32;
    let h = pm.height() as i32;
    let px = pm.data_mut();

    for ch in text.chars() {
        let gid = font.glyph_id(ch);
        if let Some(outline) = font.outline_glyph(scaled.scaled_glyph(ch)) {
            let bounds = outline.px_bounds();
            outline.draw(|gx, gy, cov| {
                let a = cov * alpha;
                if a <= 0.002 {
                    return;
                }
                let dx = (caret + bounds.min.x) as i32 + gx as i32;
                let dy = (baseline + bounds.min.y) as i32 + gy as i32;
                if dx < 0 || dy < 0 || dx >= w || dy >= h {
                    return;
                }
                let i = ((dy as usize) * (w as usize) + dx as usize) * 4;
                for (k, c) in [color.0, color.1, color.2].iter().enumerate() {
                    let dst = px[i + k] as f32;
                    px[i + k] = (dst + (*c as f32 - dst) * a).clamp(0.0, 255.0) as u8;
                }
            });
        }
        caret += scaled.h_advance(gid);
    }
}
