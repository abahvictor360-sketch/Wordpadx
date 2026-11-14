//! Touch on the page (phones, tablets): a finger dragged across the page scrolls it, with momentum
//! after a flick, and a tap places the caret, which is what brings up the on-screen keyboard. A
//! drag never selects text or takes keyboard focus, so scrolling doesn't open the keyboard.
//!
//! egui reports a touch both as `Event::Touch` and as a primary-button press; while a finger is
//! down on the page the canvas leaves the mouse handling alone and asks [`TouchPan::update`] what
//! the finger did.

use egui::{Event, Pos2, Rect, TouchId, TouchPhase, Vec2};

/// How far (points) a finger moves before the touch is a scroll rather than a tap.
const TAP_SLOP: f32 = 10.0;
/// Longest touch (seconds) that is still a tap.
const TAP_MAX_SECONDS: f64 = 0.6;
/// Two taps this close in time (seconds) and space (points) are a double tap.
const DOUBLE_TAP_SECONDS: f64 = 0.4;
const DOUBLE_TAP_DISTANCE: f32 = 30.0;
/// A flick keeps scrolling while faster than this (points per second), slowing by this factor
/// each second.
const MIN_FLICK_SPEED: f32 = 30.0;
const FLICK_DECAY_PER_SECOND: f32 = 0.05;
/// Fastest flick (points per second), so a stray reading can't throw the page a mile.
const MAX_FLICK_SPEED: f32 = 6000.0;

/// One finger on the page.
#[derive(Clone, Copy, Debug)]
struct Finger {
    id: TouchId,
    start: Pos2,
    last: Pos2,
    started: f64,
    last_time: f64,
    /// Moved past [`TAP_SLOP`]: the touch scrolls.
    panning: bool,
    /// A second finger came down (a pinch): neither a tap nor a scroll.
    cancelled: bool,
}

/// What the finger did this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TouchFrame {
    /// How far to move the page with the finger (points; positive moves the content down, as a
    /// wheel scroll up does).
    pub scroll: Vec2,
    /// A tap on the page: where, and whether it was the second of a double tap.
    pub tap: Option<(Pos2, bool)>,
}

/// The page's touch state.
#[derive(Clone, Debug, Default)]
pub struct TouchPan {
    finger: Option<Finger>,
    /// Momentum after a flick (points per second, scroll convention).
    velocity: Vec2,
    /// The last tap, for double taps.
    last_tap: Option<(f64, Pos2)>,
    /// A touch ended this frame: its press and release are touch input, not a mouse click.
    ended_this_frame: bool,
}

impl TouchPan {
    /// Read this frame's touch events. `area` is the page's part of the window; touches that start
    /// outside it (the ribbon, a pane) are left alone. `now` is the input time in seconds.
    pub fn update(&mut self, events: &[Event], area: Rect, now: f64, dt: f32) -> TouchFrame {
        let mut out = TouchFrame::default();
        self.ended_this_frame = false;
        for e in events {
            let Event::Touch { id, phase, pos, .. } = e else { continue };
            let (id, pos) = (*id, *pos);
            match phase {
                TouchPhase::Start => match &mut self.finger {
                    // A second finger: a pinch zoom (egui reports it as zoom), not a tap or scroll.
                    Some(f) if f.id != id => f.cancelled = true,
                    Some(_) => {}
                    None if area.contains(pos) => {
                        self.velocity = Vec2::ZERO;
                        self.finger = Some(Finger { id, start: pos, last: pos, started: now, last_time: now, panning: false, cancelled: false });
                    }
                    None => {}
                },
                TouchPhase::Move => {
                    if let Some(f) = self.finger.as_mut().filter(|f| f.id == id && !f.cancelled) {
                        if !f.panning && (pos - f.start).length() > TAP_SLOP {
                            f.panning = true;
                            // The first move covers the slop too, so the page doesn't lag the finger.
                            f.last = f.start;
                        }
                        if f.panning {
                            let d = pos - f.last;
                            out.scroll += d;
                            let elapsed = (now - f.last_time) as f32;
                            if elapsed > 1e-3 && d.length() > 0.0 {
                                // Smoothed, so one jittery sample doesn't decide the flick.
                                let v = d / elapsed;
                                self.velocity = self.velocity * 0.3 + v * 0.7;
                            }
                            f.last_time = now;
                        }
                        f.last = pos;
                    }
                }
                TouchPhase::End | TouchPhase::Cancel => {
                    if let Some(f) = self.finger.filter(|f| f.id == id) {
                        self.finger = None;
                        self.ended_this_frame = true;
                        let is_tap = *phase == TouchPhase::End && !f.panning && !f.cancelled && now - f.started <= TAP_MAX_SECONDS;
                        if is_tap {
                            let double =
                                self.last_tap.is_some_and(|(t, p)| now - t <= DOUBLE_TAP_SECONDS && (p - f.start).length() <= DOUBLE_TAP_DISTANCE);
                            out.tap = Some((f.start, double));
                            self.last_tap = if double { None } else { Some((now, f.start)) };
                        }
                        // A finger that stopped before lifting doesn't fling the page.
                        if !f.panning || f.cancelled || now - f.last_time > 0.1 {
                            self.velocity = Vec2::ZERO;
                        }
                        let speed = self.velocity.length();
                        if speed > MAX_FLICK_SPEED {
                            self.velocity *= MAX_FLICK_SPEED / speed;
                        }
                    }
                }
            }
        }
        // Momentum after the finger lifts.
        if self.finger.is_none() && self.velocity.length() > MIN_FLICK_SPEED && dt.is_finite() && dt > 0.0 {
            let dt = dt.min(0.1);
            out.scroll += self.velocity * dt;
            self.velocity *= FLICK_DECAY_PER_SECOND.powf(dt);
        } else if self.finger.is_none() {
            self.velocity = Vec2::ZERO;
        }
        if !out.scroll.x.is_finite() || !out.scroll.y.is_finite() {
            out.scroll = Vec2::ZERO;
            self.velocity = Vec2::ZERO;
        }
        out
    }

    /// A finger is on the page, or lifted this frame: the pointer input is touch, so the canvas
    /// leaves selection and focus to [`TouchPan`].
    pub fn in_gesture(&self) -> bool {
        self.finger.is_some() || self.ended_this_frame
    }

    /// The page is still moving after a flick (keep drawing frames).
    pub fn is_flinging(&self) -> bool {
        self.finger.is_none() && self.velocity.length() > MIN_FLICK_SPEED
    }

    /// Stop a flick (the page hit its end).
    pub fn stop(&mut self) {
        self.velocity = Vec2::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, phase: TouchPhase, x: f32, y: f32) -> Event {
        Event::Touch { device_id: egui::TouchDeviceId(1), id: TouchId(id), phase, pos: Pos2::new(x, y), force: None }
    }

    fn area() -> Rect {
        Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(400.0, 800.0))
    }

    #[test]
    fn a_drag_scrolls_and_is_not_a_tap() {
        let mut t = TouchPan::default();
        assert_eq!(t.update(&[touch(1, TouchPhase::Start, 200.0, 500.0)], area(), 0.0, 0.016).scroll, Vec2::ZERO);
        assert!(t.in_gesture());
        // Within the slop: nothing moves yet.
        assert_eq!(t.update(&[touch(1, TouchPhase::Move, 200.0, 495.0)], area(), 0.016, 0.016).scroll, Vec2::ZERO);
        // Past it: the page follows the finger from where it went down.
        let f = t.update(&[touch(1, TouchPhase::Move, 200.0, 450.0)], area(), 0.032, 0.016);
        assert_eq!(f.scroll, Vec2::new(0.0, -50.0));
        let f = t.update(&[touch(1, TouchPhase::Move, 200.0, 400.0)], area(), 0.048, 0.016);
        assert_eq!(f.scroll, Vec2::new(0.0, -50.0));
        let f = t.update(&[touch(1, TouchPhase::End, 200.0, 400.0)], area(), 0.05, 0.016);
        assert_eq!(f.tap, None, "a scroll is not a tap");
        assert!(t.in_gesture(), "the release frame is still touch input");
        // A flick keeps going, slowing down, then stops.
        assert!(t.is_flinging());
        let mut moved = 0.0;
        for i in 0..400 {
            let f = t.update(&[], area(), 0.05 + f64::from(i) * 0.016, 0.016);
            assert!(f.scroll.y <= 0.0, "the flick carries on the same way");
            moved += f.scroll.y;
        }
        assert!(moved < -50.0, "momentum moved the page: {moved}");
        assert!(!t.is_flinging(), "and stopped");
        assert!(!t.in_gesture());
    }

    #[test]
    fn a_tap_places_the_caret_and_two_make_a_double_tap() {
        let mut t = TouchPan::default();
        t.update(&[touch(1, TouchPhase::Start, 120.0, 300.0)], area(), 0.0, 0.016);
        t.update(&[touch(1, TouchPhase::Move, 123.0, 302.0)], area(), 0.05, 0.016);
        let f = t.update(&[touch(1, TouchPhase::End, 123.0, 302.0)], area(), 0.1, 0.016);
        assert_eq!(f.tap, Some((Pos2::new(120.0, 300.0), false)));
        assert_eq!(f.scroll, Vec2::ZERO);
        t.update(&[touch(2, TouchPhase::Start, 125.0, 305.0)], area(), 0.3, 0.016);
        let f = t.update(&[touch(2, TouchPhase::End, 125.0, 305.0)], area(), 0.35, 0.016);
        assert_eq!(f.tap, Some((Pos2::new(125.0, 305.0), true)));
        // A slow third tap is a fresh single tap.
        t.update(&[touch(3, TouchPhase::Start, 125.0, 305.0)], area(), 1.5, 0.016);
        let f = t.update(&[touch(3, TouchPhase::End, 125.0, 305.0)], area(), 1.55, 0.016);
        assert_eq!(f.tap, Some((Pos2::new(125.0, 305.0), false)));
    }

    #[test]
    fn long_presses_pinches_and_touches_elsewhere_are_not_taps() {
        let mut t = TouchPan::default();
        t.update(&[touch(1, TouchPhase::Start, 120.0, 300.0)], area(), 0.0, 0.016);
        assert_eq!(t.update(&[touch(1, TouchPhase::End, 120.0, 300.0)], area(), 2.0, 0.016).tap, None, "long press");
        t.update(&[touch(1, TouchPhase::Start, 120.0, 300.0), touch(2, TouchPhase::Start, 220.0, 400.0)], area(), 3.0, 0.016);
        let f = t.update(&[touch(1, TouchPhase::Move, 100.0, 280.0)], area(), 3.05, 0.016);
        assert_eq!(f.scroll, Vec2::ZERO, "a pinch doesn't scroll");
        assert_eq!(t.update(&[touch(1, TouchPhase::End, 100.0, 280.0)], area(), 3.1, 0.016).tap, None, "nor tap");
        t.update(&[touch(2, TouchPhase::End, 220.0, 400.0)], area(), 3.1, 0.016);
        // On the ribbon (above the page): not ours.
        t.update(&[touch(5, TouchPhase::Start, 100.0, 50.0)], area(), 4.0, 0.016);
        assert!(!t.in_gesture());
        assert_eq!(t.update(&[touch(5, TouchPhase::End, 100.0, 50.0)], area(), 4.05, 0.016).tap, None);
        // A cancelled touch (the system took it) is no tap either.
        t.update(&[touch(6, TouchPhase::Start, 120.0, 300.0)], area(), 5.0, 0.016);
        assert_eq!(t.update(&[touch(6, TouchPhase::Cancel, 120.0, 300.0)], area(), 5.05, 0.016).tap, None);
    }

    #[test]
    fn a_finger_that_stops_before_lifting_does_not_fling() {
        let mut t = TouchPan::default();
        t.update(&[touch(1, TouchPhase::Start, 200.0, 600.0)], area(), 0.0, 0.016);
        t.update(&[touch(1, TouchPhase::Move, 200.0, 500.0)], area(), 0.05, 0.016);
        t.update(&[touch(1, TouchPhase::End, 200.0, 500.0)], area(), 0.5, 0.016);
        assert!(!t.is_flinging());
        assert_eq!(t.update(&[], area(), 0.52, 0.016).scroll, Vec2::ZERO);
    }

    #[test]
    fn hostile_input_stays_finite() {
        let mut t = TouchPan::default();
        t.update(&[touch(1, TouchPhase::Start, 200.0, 600.0)], area(), 0.0, 0.016);
        t.update(&[touch(1, TouchPhase::Move, 200.0, f32::MAX)], area(), 1e-9, 0.016);
        let f = t.update(&[touch(1, TouchPhase::End, 200.0, f32::MAX)], area(), 2e-9, f32::NAN);
        assert!(f.scroll.x.is_finite() && f.scroll.y.is_finite());
        let f = t.update(&[], area(), 1.0, 0.016);
        assert!(f.scroll.x.is_finite() && f.scroll.y.is_finite());
    }
}
