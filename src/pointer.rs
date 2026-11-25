//! Shared macOS-style arrow geometry for raster and vector output.
//! Original artwork: black body, fine white outline, no platform assets.
const POINTS: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 23.0),
    (5.8, 17.5),
    (10.3, 27.0),
    (14.0, 25.2),
    (9.5, 15.8),
    (18.0, 15.8),
];
const OUTLINE: f32 = 0.75;
/// Offset of the soft shadow macOS draws under its arrow, in arrow units.
const SHADOW: (f32, f32) = (0.6, 1.4);
const SHADOW_ALPHA: f32 = 0.3;
pub(crate) const MIN: i32 = -1;
pub(crate) const WIDTH: i32 = 21;
pub(crate) const HEIGHT: i32 = 30;
/// The arrow's tip-to-tail height in `POINTS` units.
const ARROW_HEIGHT: f32 = 27.0;
/// macOS draws its arrow about 17 pt tall beside 13 pt terminal text.
const HEIGHT_PER_FONT_PX: f32 = 17.0 / 13.0;

/// The scale that sizes the arrow to the terminal font, as on screen.
pub(crate) fn scale_for_font(font_px: f32) -> f32 {
    font_px * HEIGHT_PER_FONT_PX / ARROW_HEIGHT
}

pub(crate) fn svg() -> String {
    let points = POINTS
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");
    let (sx, sy) = SHADOW;
    format!(
        r##"<polygon points="{points}" transform="translate({sx},{sy})" fill="#000000" fill-opacity="{SHADOW_ALPHA}" stroke="#000000" stroke-opacity="{SHADOW_ALPHA}" stroke-width="1.5" stroke-linejoin="round"/><polygon points="{points}" fill="#111111" stroke="#ffffff" stroke-width="1.5" stroke-linejoin="round"/>"##
    )
}

fn sample(x: f32, y: f32) -> Option<f32> {
    let mut inside = false;
    let mut distance = f32::INFINITY;
    for i in 0..POINTS.len() {
        let (ax, ay) = POINTS[i];
        let (bx, by) = POINTS[(i + 1) % POINTS.len()];
        if (ay > y) != (by > y) && x < (bx - ax) * (y - ay) / (by - ay) + ax {
            inside = !inside;
        }
        let dx = bx - ax;
        let dy = by - ay;
        let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        distance = distance.min((x - ax - t * dx).hypot(y - ay - t * dy));
    }
    if distance <= OUTLINE {
        Some(255.0)
    } else if inside {
        Some(17.0)
    } else {
        None
    }
}

/// Four-by-four coverage sampling avoids the previous doubled bitmap's jagged edges.
#[cfg(test)]
pub(crate) fn pixel(x: i32, y: i32) -> ([u8; 3], f32) {
    scaled_pixel(x, y, 1.0)
}

pub(crate) fn scaled_pixel(x: i32, y: i32, scale: f32) -> ([u8; 3], f32) {
    let mut count = 0;
    let mut value = 0.0;
    for sy in 0..4 {
        for sx in 0..4 {
            if let Some(v) = sample(
                (x as f32 + (sx as f32 + 0.5) / 4.0) / scale,
                (y as f32 + (sy as f32 + 0.5) / 4.0) / scale,
            ) {
                value += v;
                count += 1;
            }
        }
    }
    if count == 0 {
        return shadow_pixel(x, y, scale);
    }
    (
        [(value / count as f32).round() as u8; 3],
        count as f32 / 16.0,
    )
}

/// Coverage of the arrow's shadow, drawn where the arrow itself is not.
fn shadow_pixel(x: i32, y: i32, scale: f32) -> ([u8; 3], f32) {
    let (ox, oy) = SHADOW;
    let mut count = 0;
    for sy in 0..4 {
        for sx in 0..4 {
            let px = (x as f32 + (sx as f32 + 0.5) / 4.0) / scale - ox;
            let py = (y as f32 + (sy as f32 + 0.5) / 4.0) / scale - oy;
            if sample(px, py).is_some() {
                count += 1;
            }
        }
    }
    ([0; 3], SHADOW_ALPHA * count as f32 / 16.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressed_pointer_is_smaller() {
        let coverage = |scale| {
            (MIN..HEIGHT)
                .flat_map(|y| (MIN..WIDTH).map(move |x| scaled_pixel(x, y, scale).1))
                .sum::<f32>()
        };
        assert!(coverage(0.82) < coverage(1.0) * 0.75);
    }

    #[test]
    fn pointer_has_dark_body_white_outline_and_transparent_exterior() {
        assert_eq!(pixel(3, 10), ([17; 3], 1.0));
        assert!(pixel(0, 10).0[0] > 150);
        assert_eq!(pixel(20, 29).1, 0.0);
        assert!((MIN..HEIGHT).any(|y| (MIN..WIDTH).any(|x| {
            let a = pixel(x, y).1;
            a > 0.0 && a < 1.0
        })));
        assert!(svg().contains("stroke-width=\"1.5\""));
    }

    #[test]
    fn arrow_matches_the_terminal_font() {
        let tall = ARROW_HEIGHT * scale_for_font(13.0);
        assert!((tall - 17.0).abs() < 0.01);
    }

    #[test]
    fn shadow_falls_below_the_arrow() {
        let (color, alpha) = pixel(1, 24);
        assert_eq!(color, [0; 3]);
        assert!(alpha > 0.0 && alpha <= SHADOW_ALPHA);
    }
}
