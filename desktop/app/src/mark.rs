// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Bareq Maher (the Premium Cast mark)
//! The Premium Cast mark (`assets/brand/icon.svg`): a monitor with a violet
//! phone standing in front of its lower-right corner, sending two signal
//! waves up into the screen, on an onyx tile with a soft violet glow. Drawn
//! in code at any pixel size: the tray, window and toast icons at runtime,
//! and the executable's icon at build time (`build.rs` includes this file),
//! so every size is sharp without bitmap assets. At 32 px and below the
//! strokes are heavier and the tile fills more of the square.

/// Onyx tile, its glow, the violet rim, the monitor ink and the signal
/// (docs/design/tokens.json, Premium palette).
const TILE: [f32; 3] = rgb(0x15, 0x12, 0x2E);
const GLOW: [f32; 3] = rgb(0x2E, 0x25, 0x70);
const RIM: [f32; 3] = rgb(0xB4, 0xA6, 0xFF);
const SCREEN: [f32; 3] = rgb(0xF4, 0xF2, 0xFF);
const SIGNAL: [f32; 3] = rgb(0xB4, 0xA6, 0xFF);

/// Boxes in the 100×100 design grid: left, top, right, bottom, corner radius.
type Rect = (f32, f32, f32, f32, f32);
/// The monitor's screen (an outline) and the phone (solid).
const MONITOR: Rect = (16.0, 17.0, 74.0, 61.0, 7.0);
const PHONE: Rect = (60.0, 41.0, 84.0, 81.0, 6.0);
/// The monitor's stand: the neck, then the foot (segments with round ends).
const NECK: ((f32, f32), (f32, f32)) = ((36.0, 61.0), (36.0, 71.0));
const FOOT: ((f32, f32), (f32, f32)) = ((26.0, 71.0), (46.0, 71.0));
/// The phone's home bar, cut out of the phone.
const HOME_BAR: ((f32, f32), (f32, f32)) = ((68.0, 75.0), (76.0, 75.0));
/// Where the waves come from (the phone's top-left corner) and their radii.
const SOURCE: (f32, f32) = (60.0, 41.0);
const WAVES: [f32; 2] = [9.0, 16.0];

/// Supersampling grid per pixel (SS×SS samples) for smooth edges.
const SS: usize = 4;

const fn rgb(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32, g as f32, b as f32]
}

/// Sizes in the design grid: the small sizes are drawn bolder.
struct Geometry {
    inset: f32,
    corner: f32,
    stroke: f32,
    gap: f32,
}

impl Geometry {
    fn for_size(size: usize) -> Self {
        if size <= 32 {
            Geometry { inset: 2.0, corner: 20.0, stroke: 8.4, gap: 5.0 }
        } else {
            Geometry { inset: 4.0, corner: 22.0, stroke: 6.2, gap: 4.0 }
        }
    }
}

/// Renders the mark as straight-alpha RGBA, row by row, `size`×`size`.
pub fn render(size: usize) -> Vec<u8> {
    let g = Geometry::for_size(size);
    let unit = 100.0 / size as f32;
    // The rim is at least about a pixel wide, so the tile keeps an edge on dark taskbars.
    let rim = (1.5f32).max(unit * 0.6);
    let half = g.stroke / 2.0;

    let mut out = vec![0u8; size * size * 4];
    for py in 0..size {
        for px in 0..size {
            let mut color = [0f32; 3];
            let mut covered = 0usize;
            for sy in 0..SS {
                for sx in 0..SS {
                    // The sample in the design grid.
                    let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) * unit;
                    let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) * unit;
                    let edge = rounded_square_distance(x, y, g.inset, g.corner);
                    if edge > 0.0 {
                        continue;
                    }
                    covered += 1;
                    let background = if -edge <= rim {
                        mix(TILE, RIM, 0.34)
                    } else {
                        // The glow fades from the middle to the tile color at 42 units.
                        let (dx, dy) = (x - 50.0, y - 48.0);
                        let t = ((dx * dx + dy * dy).sqrt() / 42.0).min(1.0);
                        mix(GLOW, TILE, t)
                    };
                    let phone = box_distance(x, y, PHONE);
                    let paint = if phone <= 0.0 {
                        if segment_distance((x, y), HOME_BAR.0, HOME_BAR.1) <= half * 0.55 {
                            background
                        } else {
                            SIGNAL
                        }
                    } else if phone <= g.gap {
                        // A clear gap around the phone, so it stands in front of the monitor.
                        background
                    } else if WAVES.iter().any(|&r| wave_distance(x, y, r) <= half) {
                        SIGNAL
                    } else if box_distance(x, y, MONITOR).abs() <= half
                        || segment_distance((x, y), NECK.0, NECK.1) <= half
                        || segment_distance((x, y), FOOT.0, FOOT.1) <= half
                    {
                        SCREEN
                    } else {
                        background
                    };
                    for c in 0..3 {
                        color[c] += paint[c];
                    }
                }
            }
            if covered == 0 {
                continue;
            }
            let i = (py * size + px) * 4;
            for c in 0..3 {
                out[i + c] = (color[c] / covered as f32).round() as u8;
            }
            out[i + 3] = ((covered as f32 / (SS * SS) as f32) * 255.0).round() as u8;
        }
    }
    out
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// How far (x, y) is outside the rounded square from `inset` to
/// `100 - inset` with corner radius `r` (negative inside).
fn rounded_square_distance(x: f32, y: f32, inset: f32, r: f32) -> f32 {
    box_distance(x, y, (inset, inset, 100.0 - inset, 100.0 - inset, r))
}

/// Signed distance to a rounded box (negative inside).
fn box_distance(x: f32, y: f32, (l, t, r, b, k): Rect) -> f32 {
    let (cx, cy) = ((l + r) / 2.0, (t + b) / 2.0);
    let (hx, hy) = ((r - l) / 2.0 - k, (b - t) / 2.0 - k);
    let (qx, qy) = ((x - cx).abs() - hx, (y - cy).abs() - hy);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - k
}

/// Distance to a quarter wave of `radius` around the source, opening up and
/// to the left (into the screen), with round ends.
fn wave_distance(x: f32, y: f32, radius: f32) -> f32 {
    let (dx, dy) = (x - SOURCE.0, y - SOURCE.1);
    if dx <= 0.0 && dy <= 0.0 {
        return ((dx * dx + dy * dy).sqrt() - radius).abs();
    }
    // Nearest end: straight left, or straight up.
    let left = ((dx + radius).powi(2) + dy * dy).sqrt();
    let up = (dx * dx + (dy + radius).powi(2)).sqrt();
    left.min(up)
}

fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(img: &[u8], size: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * size + x) * 4;
        [img[i], img[i + 1], img[i + 2], img[i + 3]]
    }

    const VIOLET: [u8; 4] = [0xB4, 0xA6, 0xFF, 255];

    #[test]
    fn corners_are_transparent_and_the_phone_is_violet() {
        let size = 100;
        let img = render(size);
        assert_eq!(img.len(), size * size * 4);
        assert_eq!(pixel(&img, size, 0, 0)[3], 0, "rounded corner");
        assert_eq!(pixel(&img, size, 72, 60), VIOLET, "the phone");
        // Inside the screen, away from every stroke: the dark tile.
        let tile = pixel(&img, size, 30, 32);
        assert!(tile[0] < 0x80 && tile[3] == 255, "{tile:?}");
    }

    #[test]
    fn monitor_and_waves_are_drawn() {
        let size = 100;
        let img = render(size);
        let ink = |p: [u8; 4]| p[0] > 0xF0 && p[3] == 255;
        // The monitor's left edge and its foot.
        assert!(ink(pixel(&img, size, 16, 40)));
        assert!(ink(pixel(&img, size, 36, 71)));
        // The inner wave, up and to the left of the phone.
        assert_eq!(pixel(&img, size, 53, 34), VIOLET);
    }

    #[test]
    fn renders_every_tray_size() {
        for size in [16, 20, 24, 32, 48, 64, 256] {
            let img = render(size);
            assert!(img.chunks(4).any(|p| p[3] == 255), "{size}px has an opaque tile");
        }
    }
}
