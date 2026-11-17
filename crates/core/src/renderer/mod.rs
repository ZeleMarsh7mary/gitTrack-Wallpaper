//! CPU compositor: `Scene` -> `Pixmap`. Pure: no Win32, no file IO unless the
//! configured background is an image.

pub mod background;
pub mod heatmap;
pub mod material;

use tiny_skia::Pixmap;

use crate::config::{Config, Position};
use crate::git_fetcher::DayStat;

pub const COLS: usize = 53;
pub const ROWS: usize = 7;

/// Everything the renderer is allowed to look at.
pub struct Scene<'a> {
    pub config: &'a Config,
    pub days: &'a [DayStat],
    pub size_px: (u32, u32),
    /// 1.0 renders for the real screen; the settings preview passes a fraction.
    pub scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub card_x: f32,
    pub card_y: f32,
    pub card_w: f32,
    pub card_h: f32,
    pub grid_x: f32,
    pub grid_y: f32,
    pub box_size: f32,
    pub gap: f32,
    pub radius: f32,
    pub header_h: f32,
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub total: u32,
    pub best_day: u32,
    pub streak: u32,
}

pub fn stats_of(days: &[DayStat]) -> Stats {
    let total: u32 = days.iter().map(|d| d.count).sum();
    let best_day = days.iter().map(|d| d.count).max().unwrap_or(0);
    // Current streak: walk backwards, allowing today to be empty without
    // resetting a streak that is still alive from yesterday.
    let mut streak = 0;
    for d in days.iter().rev() {
        if d.count > 0 {
            streak += 1;
        } else if streak > 0 || d.date.is_empty() {
            break;
        }
    }
    Stats {
        total,
        best_day,
        streak,
    }
}

pub fn layout(scene: &Scene<'_>) -> Layout {
    let s = scene.scale;
    let cfg = scene.config;
    let (w, h) = (scene.size_px.0 as f32, scene.size_px.1 as f32);

    let box_size = (cfg.track.box_size * s).max(2.0);
    let gap = (cfg.track.box_gap * s).max(0.0);
    let pad = 22.0 * s;
    let header_h = if cfg.track.show_stats_header {
        (box_size * 1.9).max(16.0 * s)
    } else {
        0.0
    };
    let grid_w = COLS as f32 * box_size + (COLS - 1) as f32 * gap;
    let grid_h = ROWS as f32 * box_size + (ROWS - 1) as f32 * gap;
    let card_w = grid_w + 2.0 * pad;
    let card_h = grid_h + 2.0 * pad + header_h;

    let margin = 60.0 * s;
    let (mut x, mut y) = match cfg.card.position {
        Position::Center => ((w - card_w) / 2.0, (h - card_h) / 2.0),
        Position::TopLeft => (margin, margin),
        Position::TopRight => (w - card_w - margin, margin),
        Position::BottomLeft => (margin, h - card_h - margin),
        Position::BottomRight => (w - card_w - margin, h - card_h - margin),
        Position::Custom => (
            (w - card_w) / 2.0 + cfg.card.offset_x as f32 * s,
            (h - card_h) / 2.0 + cfg.card.offset_y as f32 * s,
        ),
    };
    x = x.clamp(0.0, (w - card_w).max(0.0));
    y = y.clamp(0.0, (h - card_h).max(0.0));

    Layout {
        card_x: x,
        card_y: y,
        card_w,
        card_h,
        grid_x: x + pad,
        grid_y: y + pad + header_h,
        box_size,
        gap,
        radius: cfg.card.corner_radius * s,
        header_h,
        font_size: (box_size * 1.25).max(10.0 * s),
    }
}

pub fn render(scene: &Scene<'_>) -> Result<Pixmap, String> {
    let (w, h) = scene.size_px;
    let mut pm = background::render(&scene.config.background, w, h);
    let l = layout(scene);

    let card = &scene.config.card;
    let (tr, tg, tb) = crate::config::rgb(&card.tint_color, 0x10161f);
    material::apply_card(
        &mut pm,
        (l.card_x, l.card_y, l.card_w, l.card_h),
        l.radius,
        card.material,
        (tr, tg, tb),
        card.opacity,
        card.blur_radius * scene.scale,
        0x9e37_79b9_7f4a_7c15,
    );
    heatmap::draw(&mut pm, scene, &l);
    Ok(pm)
}

/// Convenience for the daemon: render the scene, then hand back PNG bytes.
pub fn render_png(scene: &Scene<'_>) -> Result<Vec<u8>, String> {
    let pm = render(scene)?;
    pm.encode_png().map_err(|e| format!("encode_png: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BgMode, Config, MaterialKind};
    use crate::git_fetcher::demo_days;

    fn fixture() -> (Config, Vec<DayStat>) {
        // Built with struct-update so clippy::field_reassign_with_default stays quiet.
        let cfg = Config {
            background: crate::config::Background {
                mode: BgMode::Gradient,
                ..Default::default()
            },
            ..Config::default()
        };
        (cfg, demo_days())
    }

    #[test]
    fn renders_opaque_full_size_wallpaper() {
        let (cfg, days) = fixture();
        let scene = Scene {
            config: &cfg,
            days: &days,
            size_px: (640, 360),
            scale: 640.0 / 1920.0,
        };
        let pm = render(&scene).expect("render");
        assert_eq!(pm.width(), 640);
        assert_eq!(pm.height(), 360);
        assert!(
            pm.data().chunks_exact(4).all(|px| px[3] == 255),
            "wallpaper must be fully opaque"
        );
        let stride = 640 * 4;
        assert_ne!(
            &pm.data()[..4],
            &pm.data()[(359 * stride)..(359 * stride + 4)],
            "gradient must differ top to bottom"
        );
    }

    #[test]
    fn draws_heatmap_boxes_inside_the_card() {
        let (cfg, days) = fixture();
        let scene = Scene {
            config: &cfg,
            days: &days,
            size_px: (640, 360),
            scale: 640.0 / 1920.0,
        };
        let l = layout(&scene);
        let pm = render(&scene).expect("render");
        let (r, g, b) = crate::config::rgb(&cfg.track.colors[4], 0x39d353);
        let mut hits = 0usize;
        let data = pm.data();
        for y in (l.grid_y as u32)..((l.grid_y + 7.0 * (l.box_size + l.gap)) as u32) {
            for x in (l.grid_x as u32)..((l.grid_x + 53.0 * (l.box_size + l.gap)) as u32) {
                if x >= pm.width() || y >= pm.height() {
                    continue;
                }
                let i = ((y * pm.width() + x) * 4) as usize;
                if data[i] == r && data[i + 1] == g && data[i + 2] == b {
                    hits += 1;
                }
            }
        }
        assert!(hits > 20, "expected level-4 boxes in the grid, got {hits}");
    }

    #[test]
    fn card_material_changes_the_backdrop() {
        let (mut cfg, days) = fixture();
        cfg.card.material = MaterialKind::None;
        let without = {
            let scene = Scene {
                config: &cfg,
                days: &days,
                size_px: (320, 180),
                scale: 1.0 / 6.0,
            };
            render(&scene).expect("render")
        };
        cfg.card.material = MaterialKind::Glass;
        cfg.card.blur_radius = 24.0;
        let with = {
            let scene = Scene {
                config: &cfg,
                days: &days,
                size_px: (320, 180),
                scale: 1.0 / 6.0,
            };
            render(&scene).expect("render")
        };
        assert_ne!(without.data(), with.data(), "glass must alter pixels");
    }
}
