//! Mouse easings: the standard curves, plus one measured from a person's hand.

use easing_function::Easing;
use easing_function::easings::StandardEasing;
use serde::{Deserialize, Serialize};

/// How a move's covered share of its path grows with elapsed time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseEasing {
    Standard(StandardEasing),
    /// A hand's pointing stroke: a fast launch that peaks a quarter of the way
    /// in, then a long homing approach to the target.
    Hand,
}

/// The share of the path a hand has covered at each fortieth of the move,
/// averaged over 84 clean pointing strokes captured at about 200 Hz.
const HAND: [f32; 41] = [
    0.0, 0.023, 0.058, 0.105, 0.155, 0.208, 0.26, 0.313, 0.361, 0.409, 0.453, 0.495, 0.532, 0.569,
    0.606, 0.643, 0.676, 0.707, 0.734, 0.759, 0.783, 0.804, 0.823, 0.84, 0.857, 0.874, 0.887,
    0.901, 0.913, 0.925, 0.935, 0.944, 0.953, 0.961, 0.968, 0.974, 0.98, 0.985, 0.99, 0.995, 1.0,
];

/// A leftward stroke sags downward by about this share of its length at its
/// middle; other directions run straight.
const LEFTWARD_SAG: f32 = 0.04;

/// A terminal cell is about twice as tall as it is wide, so a path in cells
/// is measured in widths.
const CELL_ASPECT: f32 = 2.0;

impl MouseEasing {
    pub fn ease(self, t: f32) -> f32 {
        match self {
            MouseEasing::Standard(e) => e.ease(t),
            MouseEasing::Hand => {
                let x = t.clamp(0.0, 1.0) * (HAND.len() - 1) as f32;
                let i = (x as usize).min(HAND.len() - 2);
                HAND[i] + (HAND[i + 1] - HAND[i]) * (x - i as f32)
            }
        }
    }
}

impl From<StandardEasing> for MouseEasing {
    fn from(e: StandardEasing) -> Self {
        MouseEasing::Standard(e)
    }
}

impl Default for MouseEasing {
    fn default() -> Self {
        MouseEasing::Standard(StandardEasing::InOutCubic)
    }
}

/// Positions along `points` at even steps of `step_ms`, each at the share of
/// the path's length the easing gives for that time. A share past either end
/// continues along the end segment, so an overshooting easing overshoots.
pub fn timed_path(
    points: &[(f32, f32)],
    easing: MouseEasing,
    total_ms: f32,
    step_ms: f32,
) -> Vec<(f32, (f32, f32))> {
    if points.len() < 2 || total_ms <= 0.0 {
        return points.iter().map(|&p| (total_ms.max(0.0), p)).collect();
    }
    let mut lengths = vec![0.0f32];
    for w in points.windows(2) {
        let d = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
        lengths.push(lengths.last().copied().unwrap_or(0.0) + d);
    }
    let total_len = lengths.last().copied().unwrap_or(0.0);
    let at = |share: f32| -> (f32, f32) {
        if total_len == 0.0 {
            return points[0];
        }
        let target = share * total_len;
        let i = match lengths.iter().position(|&l| l >= target) {
            Some(0) => 1,
            Some(i) => i,
            None => lengths.len() - 1,
        };
        let (a, b) = (points[i - 1], points[i]);
        let seg = (lengths[i] - lengths[i - 1]).max(f32::EPSILON);
        let f = (target - lengths[i - 1]) / seg;
        (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
    };
    // A hand's leftward stroke bows downward, most at its middle.
    let (first, last) = (points[0], points[points.len() - 1]);
    let (dx, dy) = (last.0 - first.0, (last.1 - first.1) * CELL_ASPECT);
    let span = (dx * dx + dy * dy).sqrt();
    let sag = if easing == MouseEasing::Hand && dx < 0.0 && span > 0.0 {
        LEFTWARD_SAG * span
    } else {
        0.0
    };
    // The unit normal that points down the screen, in cell units.
    let (nx, ny) = if span > 0.0 {
        let (mut nx, mut ny) = (-dy / span, dx / span);
        if ny < 0.0 {
            (nx, ny) = (-nx, -ny);
        }
        (nx, ny / CELL_ASPECT)
    } else {
        (0.0, 0.0)
    };
    let steps = (total_ms / step_ms).ceil().max(1.0) as usize;
    (0..=steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            let share = easing.ease(t);
            let (x, y) = at(share);
            let bow = sag * 4.0 * share * (1.0 - share);
            (t * total_ms, (x + nx * bow, y + ny * bow))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_launches_fast_and_homes_in_slowly() {
        let h = MouseEasing::Hand;
        assert_eq!(h.ease(0.0), 0.0);
        assert_eq!(h.ease(1.0), 1.0);
        assert!(h.ease(0.3) > 0.5, "half the way covered early");
        assert!(h.ease(0.75) < 0.95, "the last stretch takes its time");
    }

    #[test]
    fn a_leftward_hand_stroke_sags_down_and_lands_on_target() {
        let path = [(40.0, 10.0), (0.0, 10.0)];
        let samples = timed_path(&path, MouseEasing::Hand, 300.0, 5.0);
        let deepest = samples.iter().map(|(_, (_, y))| *y).fold(0.0f32, f32::max);
        // 4% of 40 widths is 1.6 widths, or 0.8 rows.
        assert!(deepest > 10.7, "the middle runs below the line: {deepest}");
        assert_eq!(samples.last().unwrap().1, (0.0, 10.0));
        let right = timed_path(&[(0.0, 10.0), (40.0, 10.0)], MouseEasing::Hand, 300.0, 5.0);
        assert!(right.iter().all(|(_, (_, y))| *y == 10.0));
    }

    #[test]
    fn position_follows_the_easing_in_time() {
        let path = [(0.0, 0.0), (10.0, 0.0)];
        let samples = timed_path(
            &path,
            MouseEasing::Standard(StandardEasing::Linear),
            100.0,
            25.0,
        );
        let xs: Vec<f32> = samples.iter().map(|(_, (x, _))| *x).collect();
        assert_eq!(xs, vec![0.0, 2.5, 5.0, 7.5, 10.0]);
        let quad = timed_path(
            &path,
            MouseEasing::Standard(StandardEasing::InQuadradic),
            100.0,
            50.0,
        );
        assert!(
            (quad[1].1.0 - 2.5).abs() < 1e-4,
            "half the time covers a quarter"
        );
    }
}
