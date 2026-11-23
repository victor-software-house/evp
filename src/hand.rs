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
/// averaged over straight strokes captured from a pointer at about 200 Hz.
const HAND: [f32; 41] = [
    0.0, 0.023, 0.042, 0.068, 0.102, 0.145, 0.188, 0.25, 0.316, 0.386, 0.458, 0.525, 0.585, 0.639,
    0.676, 0.704, 0.729, 0.755, 0.773, 0.787, 0.802, 0.821, 0.839, 0.851, 0.857, 0.864, 0.871,
    0.884, 0.897, 0.904, 0.911, 0.916, 0.922, 0.927, 0.933, 0.94, 0.952, 0.964, 0.976, 0.988, 1.0,
];

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
    let steps = (total_ms / step_ms).ceil().max(1.0) as usize;
    (0..=steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            (t * total_ms, at(easing.ease(t)))
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
        assert!(h.ease(0.3) > 0.55, "most of the way covered early");
        assert!(h.ease(0.75) < 0.95, "the last stretch takes its time");
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
