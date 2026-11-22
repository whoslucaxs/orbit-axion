//! Pure interaction rules shared by the hook and the UI; coordinates are logical pixels.
use std::f64::consts::{FRAC_PI_2, PI, TAU};
pub mod model;

pub const RADIUS: f64 = 112.0;
pub const ICON_RADIUS: f64 = 29.0;
pub const SIZE: f64 = 360.0;

pub fn icon_center(slot: usize) -> (f64, f64) {
    let angle = slot as f64 * TAU / 8.0 - FRAC_PI_2;
    (RADIUS * angle.cos(), RADIUS * angle.sin())
}

pub fn icon_at(x: f64, y: f64) -> Option<usize> {
    (0..8).find(|&slot| {
        let (cx, cy) = icon_center(slot);
        (x - cx).hypot(y - cy) <= ICON_RADIUS
    })
}

pub fn direction(x: f64, y: f64, min_distance: f64) -> Option<usize> {
    if x.hypot(y) < min_distance {
        return None;
    }
    Some((((y.atan2(x) + FRAC_PI_2 + PI / 8.0).rem_euclid(TAU)) / (TAU / 8.0)) as usize % 8)
}

#[derive(Default)]
pub struct Gesture {
    wheel_adjusted: bool,
    remainder: i32,
}

impl Gesture {
    pub fn reset_remainder(&mut self) {
        self.remainder = 0;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Windows reports 120 units per notch. Preserve high-resolution wheel remainders.
    pub fn wheel(&mut self, delta: i16, over_volume: bool) -> i32 {
        if !over_volume {
            self.remainder = 0;
            return 0;
        }
        self.wheel_adjusted = true;
        self.remainder += i32::from(delta);
        let notches = self.remainder / 120;
        self.remainder %= 120;
        notches
    }

    pub fn release(&self, selected: Option<usize>) -> Option<usize> {
        if self.wheel_adjusted { None } else { selected }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_directions_and_actual_icon_hit_regions() {
        for slot in 0..8 {
            let (x, y) = icon_center(slot);
            assert_eq!(direction(x, y, 48.0), Some(slot));
            assert_eq!(icon_at(x, y), Some(slot));
        }
        assert_eq!(direction(0.0, 0.0, 48.0), None);
        assert_eq!(icon_at(0.0, 0.0), None);
        assert_eq!(icon_at(0.0, -170.0), None);
    }

    #[test]
    fn wheel_aggregates_and_consumes_release_even_below_a_notch() {
        let mut gesture = Gesture::default();
        assert_eq!(gesture.wheel(30, true), 0);
        assert_eq!(gesture.release(Some(0)), None);
        assert_eq!(gesture.wheel(90, true), 1);
        assert_eq!(gesture.wheel(-240, true), -2);
        gesture.reset();
        assert_eq!(gesture.release(Some(2)), Some(2));
        assert_eq!(gesture.wheel(120, false), 0);
        assert_eq!(gesture.release(Some(2)), Some(2));
    }

    #[test]
    fn wheel_remainder_does_not_leak_between_hover_regions() {
        let mut gesture = Gesture::default();
        assert_eq!(gesture.wheel(60, true), 0);
        gesture.wheel(120, false);
        assert_eq!(gesture.wheel(60, true), 0);
    }
}
