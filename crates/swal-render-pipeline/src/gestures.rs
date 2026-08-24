//! High-precision gesture recognition and physics engine for SWAL Desktop compositor.
//! Provides kinetic scrolling with calibrated friction (0.92 default), multi-touch
//! pinch/zoom/rotate delta calculations, panning, swiping, tap, and long-press recognition.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;
use std::time::{Duration, Instant};

/// Default friction coefficient for kinetic scrolling physics.
pub const DEFAULT_KINETIC_FRICTION: f64 = 0.92;

/// Minimum velocity threshold below which kinetic motion halts (pixels/tick or pixels/second).
pub const DEFAULT_MIN_VELOCITY: f64 = 0.01;

/// Movement slop (distance in pixels) before a touch gesture transitions from stationary to panning.
pub const DEFAULT_PAN_SLOP: f64 = 8.0;

/// Maximum duration for a touch to qualify as a tap.
pub const DEFAULT_TAP_TIMEOUT: Duration = Duration::from_millis(300);

/// Maximum delay between taps to count as a multi/double-tap.
pub const DEFAULT_DOUBLE_TAP_TIMEOUT: Duration = Duration::from_millis(350);

/// Minimum hold duration for a long press.
pub const DEFAULT_LONG_PRESS_DURATION: Duration = Duration::from_millis(500);

/// Minimum velocity (pixels/sec) to qualify as a swipe gesture.
pub const DEFAULT_SWIPE_MIN_VELOCITY: f64 = 250.0;

/// Discrete lifecycle states of recognized gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GestureState {
    Began,
    Updated,
    Ended,
    Cancelled,
}

impl GestureState {
    /// Returns true if the gesture is currently active (Began or Updated).
    pub fn is_active(&self) -> bool {
        matches!(self, GestureState::Began | GestureState::Updated)
    }

    /// Returns true if the gesture has terminated (Ended or Cancelled).
    pub fn is_terminal(&self) -> bool {
        matches!(self, GestureState::Ended | GestureState::Cancelled)
    }
}

/// 2D Cartesian point/vector used for touch coordinates, displacements, and velocities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance between two points.
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Midpoint between two points.
    pub fn midpoint(&self, other: &Point2D) -> Point2D {
        Point2D::new((self.x + other.x) * 0.5, (self.y + other.y) * 0.5)
    }

    /// Direction angle in radians from `self` to `other`.
    pub fn angle_to(&self, other: &Point2D) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    /// Vector length / Euclidean norm.
    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// Returns normalized unit vector if length > 0.
    pub fn normalized(&self) -> Option<Point2D> {
        let len = self.length();
        if len > 1e-9 {
            Some(Point2D::new(self.x / len, self.y / len))
        } else {
            None
        }
    }
}

/// Cardinal directions for swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Computed delta metrics for two-finger pinch, zoom, rotate, and focal panning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PinchDelta {
    /// Cumulative scale relative to the initial touch distance (1.0 = unchanged, >1.0 = zoom in, <1.0 = zoom out).
    pub scale_factor: f64,
    /// Incremental scale delta since previous sample (current_dist / previous_dist).
    pub delta_scale: f64,
    /// Current geometric focal center point between the two touch points.
    pub focal_center: Point2D,
    /// Translation displacement of the focal center since previous sample.
    pub focal_delta: Point2D,
    /// Rotation delta angle in radians normalized within [-PI, PI].
    pub rotation_delta: f64,
    /// Absolute current distance in pixels between the two touch points.
    pub distance: f64,
}

impl PinchDelta {
    /// Computes full pinch delta metrics from initial, previous, and current two-finger touch points.
    pub fn calculate(
        initial_p1: Point2D,
        initial_p2: Point2D,
        prev_p1: Point2D,
        prev_p2: Point2D,
        curr_p1: Point2D,
        curr_p2: Point2D,
    ) -> Self {
        let initial_dist = initial_p1.distance_to(&initial_p2).max(1e-6);
        let prev_dist = prev_p1.distance_to(&prev_p2).max(1e-6);
        let curr_dist = curr_p1.distance_to(&curr_p2);

        let prev_center = prev_p1.midpoint(&prev_p2);
        let curr_center = curr_p1.midpoint(&curr_p2);

        let prev_angle = prev_p1.angle_to(&prev_p2);
        let curr_angle = curr_p1.angle_to(&curr_p2);
        let mut rotation_delta = curr_angle - prev_angle;

        while rotation_delta > PI {
            rotation_delta -= 2.0 * PI;
        }
        while rotation_delta < -PI {
            rotation_delta += 2.0 * PI;
        }

        Self {
            scale_factor: curr_dist / initial_dist,
            delta_scale: curr_dist / prev_dist,
            focal_center: curr_center,
            focal_delta: Point2D::new(curr_center.x - prev_center.x, curr_center.y - prev_center.y),
            rotation_delta,
            distance: curr_dist,
        }
    }
}

/// Kinetic scrolling physics simulation engine.
/// Simulates smooth momentum deceleration using calibrated friction (default: 0.92).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KineticScroller {
    /// Friction coefficient applied per simulation step (default: 0.92).
    pub friction: f64,
    /// Current velocity along X axis (pixels per tick or pixels/second).
    pub velocity_x: f64,
    /// Current velocity along Y axis (pixels per tick or pixels/second).
    pub velocity_y: f64,
    /// Minimum velocity below which scrolling halts and snaps to zero.
    pub min_velocity: f64,
    /// Accumulated position offset along X axis.
    pub position_x: f64,
    /// Accumulated position offset along Y axis.
    pub position_y: f64,
}

impl Default for KineticScroller {
    fn default() -> Self {
        Self {
            friction: DEFAULT_KINETIC_FRICTION,
            velocity_x: 0.0,
            velocity_y: 0.0,
            min_velocity: DEFAULT_MIN_VELOCITY,
            position_x: 0.0,
            position_y: 0.0,
        }
    }
}

impl KineticScroller {
    /// Creates a new kinetic scroller with default friction 0.92.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom friction coefficient (clamped in [0.0, 0.9999]).
    pub fn with_friction(mut self, friction: f64) -> Self {
        self.friction = friction.clamp(0.0, 0.9999);
        self
    }

    /// Sets minimum velocity cutoff.
    pub fn with_min_velocity(mut self, min_velocity: f64) -> Self {
        self.min_velocity = min_velocity.max(0.0);
        self
    }

    /// Launches kinetic scrolling with an initial velocity fling vector (vx, vy).
    pub fn fling(&mut self, vx: f64, vy: f64) {
        self.velocity_x = vx;
        self.velocity_y = vy;
    }

    /// Returns true if the kinetic scroller is currently in motion.
    pub fn is_active(&self) -> bool {
        self.speed() >= self.min_velocity
    }

    /// Current speed (magnitude of velocity vector).
    pub fn speed(&self) -> f64 {
        (self.velocity_x.powi(2) + self.velocity_y.powi(2)).sqrt()
    }

    /// Current velocity vector (vx, vy).
    pub fn velocity(&self) -> Point2D {
        Point2D::new(self.velocity_x, self.velocity_y)
    }

    /// Current accumulated position (x, y).
    pub fn position(&self) -> Point2D {
        Point2D::new(self.position_x, self.position_y)
    }

    /// Advances the kinetic simulation by one tick.
    /// Returns step displacement `Some(Point2D)` if active, or `None` if halted.
    pub fn tick(&mut self) -> Option<Point2D> {
        if !self.is_active() {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            return None;
        }

        let dx = self.velocity_x;
        let dy = self.velocity_y;

        self.position_x += dx;
        self.position_y += dy;

        // Apply friction deceleration (0.92)
        self.velocity_x *= self.friction;
        self.velocity_y *= self.friction;

        if !self.is_active() {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
        }

        Some(Point2D::new(dx, dy))
    }

    /// Advances the kinetic simulation with time delta `dt_secs` calibrated to a reference frame rate (e.g. 200Hz).
    pub fn tick_dt(&mut self, dt_secs: f64, reference_fps: f64) -> Option<Point2D> {
        if !self.is_active() || dt_secs <= 0.0 {
            if !self.is_active() {
                self.velocity_x = 0.0;
                self.velocity_y = 0.0;
            }
            return None;
        }

        let step_factor = dt_secs * reference_fps;
        let effective_friction = self.friction.powf(step_factor);

        let dx = self.velocity_x * step_factor;
        let dy = self.velocity_y * step_factor;

        self.position_x += dx;
        self.position_y += dy;

        self.velocity_x *= effective_friction;
        self.velocity_y *= effective_friction;

        if !self.is_active() {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
        }

        Some(Point2D::new(dx, dy))
    }

    /// Stops kinetic scrolling immediately.
    pub fn stop(&mut self) {
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
    }

    /// Resets velocity and position accumulator to zero.
    pub fn reset(&mut self) {
        self.stop();
        self.position_x = 0.0;
        self.position_y = 0.0;
    }
}

/// Gesture events emitted by `GestureRecognizer`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GestureEvent {
    /// Panning / Dragging gesture event.
    Pan {
        state: GestureState,
        translation: Point2D,
        delta: Point2D,
        velocity: Point2D,
        focal_point: Point2D,
    },
    /// Two-finger pinch / zoom / rotation gesture event.
    Pinch {
        state: GestureState,
        delta: PinchDelta,
    },
    /// High-velocity swipe event.
    Swipe {
        direction: SwipeDirection,
        fingers: usize,
        velocity: f64,
        distance: f64,
    },
    /// Tap gesture event (supports single and multi-tap).
    Tap {
        position: Point2D,
        tap_count: u32,
    },
    /// Long press gesture event.
    LongPress {
        position: Point2D,
        duration: Duration,
    },
    /// Kinetic scrolling physics motion event.
    KineticScroll {
        delta: Point2D,
        velocity: Point2D,
        position: Point2D,
    },
}

/// Internal tracking state for an individual touch contact.
#[derive(Debug, Clone)]
struct TouchRecord {
    pub start_pos: Point2D,
    pub start_time: Instant,
    pub current_pos: Point2D,
    pub prev_pos: Point2D,
    pub last_time: Instant,
    pub history: VecDeque<(Point2D, Instant)>,
}

impl TouchRecord {
    pub fn new(pos: Point2D, time: Instant) -> Self {
        let mut history = VecDeque::with_capacity(16);
        history.push_back((pos, time));
        Self {
            start_pos: pos,
            start_time: time,
            current_pos: pos,
            prev_pos: pos,
            last_time: time,
            history,
        }
    }

    pub fn update(&mut self, pos: Point2D, time: Instant) {
        self.prev_pos = self.current_pos;
        self.current_pos = pos;
        self.last_time = time;
        self.history.push_back((pos, time));

        // Retain recent history (last 150ms) for accurate velocity tracking
        while self.history.len() > 1 {
            if let Some((_, first_time)) = self.history.front() {
                if time.duration_since(*first_time).as_millis() > 150 {
                    self.history.pop_front();
                    continue;
                }
            }
            break;
        }
    }

    /// Computes velocity vector in pixels per second using linear sample history.
    pub fn compute_velocity(&self) -> Point2D {
        if self.history.len() < 2 {
            return Point2D::ZERO;
        }

        let (first_pos, first_time) = self.history.front().unwrap();
        let (last_pos, last_time) = self.history.back().unwrap();
        let dt = last_time.duration_since(*first_time).as_secs_f64();

        if dt > 1e-5 {
            Point2D::new((last_pos.x - first_pos.x) / dt, (last_pos.y - first_pos.y) / dt)
        } else {
            Point2D::ZERO
        }
    }
}

/// Multi-touch gesture recognizer and kinetic momentum coordinator.
#[derive(Debug, Clone)]
pub struct GestureRecognizer {
    touches: HashMap<u64, TouchRecord>,
    touch_order: Vec<u64>,
    pub kinetic_scroller: KineticScroller,

    // Configuration parameters
    pub pan_slop: f64,
    pub tap_timeout: Duration,
    pub double_tap_timeout: Duration,
    pub long_press_duration: Duration,
    pub swipe_min_velocity: f64,

    // Internal state tracking
    is_panning: bool,
    pan_start_focal: Point2D,
    pan_prev_focal: Point2D,

    is_pinching: bool,
    pinch_initial_p1: Point2D,
    pinch_initial_p2: Point2D,
    pinch_prev_p1: Point2D,
    pinch_prev_p2: Point2D,

    last_tap_position: Option<Point2D>,
    last_tap_time: Option<Instant>,
    tap_count: u32,
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self {
            touches: HashMap::new(),
            touch_order: Vec::new(),
            kinetic_scroller: KineticScroller::default(),
            pan_slop: DEFAULT_PAN_SLOP,
            tap_timeout: DEFAULT_TAP_TIMEOUT,
            double_tap_timeout: DEFAULT_DOUBLE_TAP_TIMEOUT,
            long_press_duration: DEFAULT_LONG_PRESS_DURATION,
            swipe_min_velocity: DEFAULT_SWIPE_MIN_VELOCITY,

            is_panning: false,
            pan_start_focal: Point2D::ZERO,
            pan_prev_focal: Point2D::ZERO,

            is_pinching: false,
            pinch_initial_p1: Point2D::ZERO,
            pinch_initial_p2: Point2D::ZERO,
            pinch_prev_p1: Point2D::ZERO,
            pinch_prev_p2: Point2D::ZERO,

            last_tap_position: None,
            last_tap_time: None,
            tap_count: 0,
        }
    }
}

impl GestureRecognizer {
    /// Creates a new `GestureRecognizer` with default settings and kinetic friction 0.92.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom kinetic friction coefficient.
    pub fn with_kinetic_friction(mut self, friction: f64) -> Self {
        self.kinetic_scroller = self.kinetic_scroller.with_friction(friction);
        self
    }

    /// Handles pointer/touch contact down event.
    pub fn handle_touch_down(&mut self, id: u64, x: f64, y: f64, time: Instant) -> Vec<GestureEvent> {
        let mut events = Vec::new();

        // Any new touch contact immediately interrupts ongoing kinetic scrolling
        self.kinetic_scroller.stop();

        let pos = Point2D::new(x, y);
        let record = TouchRecord::new(pos, time);

        if !self.touches.contains_key(&id) {
            self.touch_order.push(id);
        }
        self.touches.insert(id, record);

        // If two fingers are now down, initialize pinch gesture tracking
        if self.touch_order.len() == 2 {
            let id1 = self.touch_order[0];
            let id2 = self.touch_order[1];
            if let (Some(t1), Some(t2)) = (self.touches.get(&id1), self.touches.get(&id2)) {
                self.is_pinching = true;
                self.pinch_initial_p1 = t1.current_pos;
                self.pinch_initial_p2 = t2.current_pos;
                self.pinch_prev_p1 = t1.current_pos;
                self.pinch_prev_p2 = t2.current_pos;

                let delta = PinchDelta::calculate(
                    self.pinch_initial_p1,
                    self.pinch_initial_p2,
                    self.pinch_prev_p1,
                    self.pinch_prev_p2,
                    t1.current_pos,
                    t2.current_pos,
                );

                events.push(GestureEvent::Pinch {
                    state: GestureState::Began,
                    delta,
                });
            }
        }

        events
    }

    /// Handles pointer/touch move event.
    pub fn handle_touch_move(&mut self, id: u64, x: f64, y: f64, time: Instant) -> Vec<GestureEvent> {
        let mut events = Vec::new();
        let pos = Point2D::new(x, y);

        if let Some(record) = self.touches.get_mut(&id) {
            record.update(pos, time);
        } else {
            return events;
        }

        // Two-finger Pinch tracking
        if self.touch_order.len() == 2 && self.is_pinching {
            let id1 = self.touch_order[0];
            let id2 = self.touch_order[1];
            if let (Some(t1), Some(t2)) = (self.touches.get(&id1), self.touches.get(&id2)) {
                let delta = PinchDelta::calculate(
                    self.pinch_initial_p1,
                    self.pinch_initial_p2,
                    self.pinch_prev_p1,
                    self.pinch_prev_p2,
                    t1.current_pos,
                    t2.current_pos,
                );

                self.pinch_prev_p1 = t1.current_pos;
                self.pinch_prev_p2 = t2.current_pos;

                events.push(GestureEvent::Pinch {
                    state: GestureState::Updated,
                    delta,
                });
            }
        }
        // Single-finger Pan / Drag tracking
        else if self.touch_order.len() == 1 {
            let touch_id = self.touch_order[0];
            if let Some(record) = self.touches.get(&touch_id) {
                let dist_from_start = record.current_pos.distance_to(&record.start_pos);

                if !self.is_panning && dist_from_start >= self.pan_slop {
                    self.is_panning = true;
                    self.pan_start_focal = record.start_pos;
                    self.pan_prev_focal = record.prev_pos;

                    let translation = Point2D::new(
                        record.current_pos.x - self.pan_start_focal.x,
                        record.current_pos.y - self.pan_start_focal.y,
                    );
                    let delta = Point2D::new(
                        record.current_pos.x - self.pan_prev_focal.x,
                        record.current_pos.y - self.pan_prev_focal.y,
                    );
                    let velocity = record.compute_velocity();

                    events.push(GestureEvent::Pan {
                        state: GestureState::Began,
                        translation,
                        delta,
                        velocity,
                        focal_point: record.current_pos,
                    });

                    self.pan_prev_focal = record.current_pos;
                } else if self.is_panning {
                    let translation = Point2D::new(
                        record.current_pos.x - self.pan_start_focal.x,
                        record.current_pos.y - self.pan_start_focal.y,
                    );
                    let delta = Point2D::new(
                        record.current_pos.x - self.pan_prev_focal.x,
                        record.current_pos.y - self.pan_prev_focal.y,
                    );
                    let velocity = record.compute_velocity();

                    events.push(GestureEvent::Pan {
                        state: GestureState::Updated,
                        translation,
                        delta,
                        velocity,
                        focal_point: record.current_pos,
                    });

                    self.pan_prev_focal = record.current_pos;
                }
            }
        }

        events
    }

    /// Handles pointer/touch up / release event.
    pub fn handle_touch_up(&mut self, id: u64, x: f64, y: f64, time: Instant) -> Vec<GestureEvent> {
        let mut events = Vec::new();
        let pos = Point2D::new(x, y);

        if let Some(record) = self.touches.get_mut(&id) {
            record.update(pos, time);
        }

        // Pinch completion
        if self.is_pinching && self.touch_order.contains(&id) {
            let id1 = self.touch_order[0];
            let id2 = self.touch_order[1];
            if let (Some(t1), Some(t2)) = (self.touches.get(&id1), self.touches.get(&id2)) {
                let delta = PinchDelta::calculate(
                    self.pinch_initial_p1,
                    self.pinch_initial_p2,
                    self.pinch_prev_p1,
                    self.pinch_prev_p2,
                    t1.current_pos,
                    t2.current_pos,
                );

                events.push(GestureEvent::Pinch {
                    state: GestureState::Ended,
                    delta,
                });
            }
            self.is_pinching = false;
        }

        // Pan completion, swipe detection, and kinetic fling trigger
        if self.is_panning && self.touch_order.len() == 1 && self.touch_order[0] == id {
            if let Some(record) = self.touches.get(&id) {
                let translation = Point2D::new(
                    record.current_pos.x - self.pan_start_focal.x,
                    record.current_pos.y - self.pan_start_focal.y,
                );
                let delta = Point2D::new(
                    record.current_pos.x - self.pan_prev_focal.x,
                    record.current_pos.y - self.pan_prev_focal.y,
                );
                let velocity = record.compute_velocity();

                events.push(GestureEvent::Pan {
                    state: GestureState::Ended,
                    translation,
                    delta,
                    velocity,
                    focal_point: record.current_pos,
                });

                let speed = velocity.length();
                let total_dist = translation.length();

                // Swipe evaluation
                if speed >= self.swipe_min_velocity && total_dist >= self.pan_slop {
                    let dir = if translation.x.abs() > translation.y.abs() {
                        if translation.x > 0.0 {
                            SwipeDirection::Right
                        } else {
                            SwipeDirection::Left
                        }
                    } else if translation.y > 0.0 {
                        SwipeDirection::Down
                    } else {
                        SwipeDirection::Up
                    };

                    events.push(GestureEvent::Swipe {
                        direction: dir,
                        fingers: 1,
                        velocity: speed,
                        distance: total_dist,
                    });
                }

                // Launch kinetic scrolling momentum if released with velocity
                if speed > self.kinetic_scroller.min_velocity {
                    // Normalize velocity to tick units (assuming ~60-200 ticks/sec reference frame)
                    self.kinetic_scroller.fling(velocity.x / 60.0, velocity.y / 60.0);
                }
            }
            self.is_panning = false;
        }
        // Tap / Double Tap / Long Press detection when not panning
        else if !self.is_panning && self.touch_order.len() == 1 && self.touch_order[0] == id {
            if let Some(record) = self.touches.get(&id) {
                let duration = time.duration_since(record.start_time);
                let dist = record.current_pos.distance_to(&record.start_pos);

                if dist < self.pan_slop {
                    if duration >= self.long_press_duration {
                        events.push(GestureEvent::LongPress {
                            position: record.current_pos,
                            duration,
                        });
                    } else if duration <= self.tap_timeout {
                        let is_double_tap = if let (Some(last_pos), Some(last_time)) =
                            (self.last_tap_position, self.last_tap_time)
                        {
                            let time_diff = time.duration_since(last_time);
                            let pos_diff = record.current_pos.distance_to(&last_pos);
                            time_diff <= self.double_tap_timeout && pos_diff < 20.0
                        } else {
                            false
                        };

                        if is_double_tap {
                            self.tap_count += 1;
                        } else {
                            self.tap_count = 1;
                        }

                        self.last_tap_position = Some(record.current_pos);
                        self.last_tap_time = Some(time);

                        events.push(GestureEvent::Tap {
                            position: record.current_pos,
                            tap_count: self.tap_count,
                        });
                    }
                }
            }
        }

        // Clean up touch tracking
        self.touches.remove(&id);
        self.touch_order.retain(|&tid| tid != id);

        events
    }

    /// Handles pointer/touch cancellation event.
    pub fn handle_touch_cancel(&mut self, id: u64, _time: Instant) -> Vec<GestureEvent> {
        let mut events = Vec::new();

        if self.is_pinching && self.touch_order.contains(&id) {
            let delta = PinchDelta {
                scale_factor: 1.0,
                delta_scale: 1.0,
                focal_center: self.pinch_prev_p1.midpoint(&self.pinch_prev_p2),
                focal_delta: Point2D::ZERO,
                rotation_delta: 0.0,
                distance: self.pinch_prev_p1.distance_to(&self.pinch_prev_p2),
            };

            events.push(GestureEvent::Pinch {
                state: GestureState::Cancelled,
                delta,
            });
            self.is_pinching = false;
        }

        if self.is_panning && self.touch_order.contains(&id) {
            events.push(GestureEvent::Pan {
                state: GestureState::Cancelled,
                translation: Point2D::ZERO,
                delta: Point2D::ZERO,
                velocity: Point2D::ZERO,
                focal_point: self.pan_prev_focal,
            });
            self.is_panning = false;
        }

        self.touches.remove(&id);
        self.touch_order.retain(|&tid| tid != id);

        events
    }

    /// Advances kinetic scrolling physics simulation by one tick.
    /// Returns `Some(GestureEvent::KineticScroll)` if active momentum motion occurred.
    pub fn tick_kinetic(&mut self) -> Option<GestureEvent> {
        if let Some(delta) = self.kinetic_scroller.tick() {
            Some(GestureEvent::KineticScroll {
                delta,
                velocity: self.kinetic_scroller.velocity(),
                position: self.kinetic_scroller.position(),
            })
        } else {
            None
        }
    }

    /// Advances kinetic scrolling simulation with time delta `dt_secs` and reference FPS (e.g. 200Hz).
    pub fn tick_kinetic_dt(&mut self, dt_secs: f64, reference_fps: f64) -> Option<GestureEvent> {
        if let Some(delta) = self.kinetic_scroller.tick_dt(dt_secs, reference_fps) {
            Some(GestureEvent::KineticScroll {
                delta,
                velocity: self.kinetic_scroller.velocity(),
                position: self.kinetic_scroller.position(),
            })
        } else {
            None
        }
    }

    /// Convenience helper for touch down at current timestamp.
    pub fn on_touch_down(&mut self, id: u64, x: f64, y: f64) -> Vec<GestureEvent> {
        self.handle_touch_down(id, x, y, Instant::now())
    }

    /// Convenience helper for touch move at current timestamp.
    pub fn on_touch_move(&mut self, id: u64, x: f64, y: f64) -> Vec<GestureEvent> {
        self.handle_touch_move(id, x, y, Instant::now())
    }

    /// Convenience helper for touch up at current timestamp.
    pub fn on_touch_up(&mut self, id: u64, x: f64, y: f64) -> Vec<GestureEvent> {
        self.handle_touch_up(id, x, y, Instant::now())
    }

    /// Convenience helper for touch cancel at current timestamp.
    pub fn on_touch_cancel(&mut self, id: u64) -> Vec<GestureEvent> {
        self.handle_touch_cancel(id, Instant::now())
    }

    /// Returns the number of currently active touch points.
    pub fn active_touch_count(&self) -> usize {
        self.touches.len()
    }

    /// Returns true if a specific touch point ID is actively tracked.
    pub fn is_touch_active(&self, id: u64) -> bool {
        self.touches.contains_key(&id)
    }

    /// Resets all internal recognizer state, active touches, and stops kinetic scrolling.
    pub fn reset(&mut self) {
        self.touches.clear();
        self.touch_order.clear();
        self.kinetic_scroller.reset();
        self.is_panning = false;
        self.is_pinching = false;
        self.last_tap_position = None;
        self.last_tap_time = None;
        self.tap_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_state_properties() {
        assert!(GestureState::Began.is_active());
        assert!(GestureState::Updated.is_active());
        assert!(!GestureState::Ended.is_active());
        assert!(!GestureState::Cancelled.is_active());

        assert!(!GestureState::Began.is_terminal());
        assert!(!GestureState::Updated.is_terminal());
        assert!(GestureState::Ended.is_terminal());
        assert!(GestureState::Cancelled.is_terminal());
    }

    #[test]
    fn test_point2d_math() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.midpoint(&p2), Point2D::new(1.5, 2.0));
        assert_eq!(p2.length(), 5.0);

        let norm = p2.normalized().unwrap();
        assert!((norm.x - 0.6).abs() < 1e-6);
        assert!((norm.y - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_scroller_friction_0_92_physics() {
        let mut scroller = KineticScroller::new();
        assert_eq!(scroller.friction, 0.92);

        // Fling with velocity (100.0, 50.0)
        scroller.fling(100.0, 50.0);
        assert!(scroller.is_active());
        assert_eq!(scroller.velocity(), Point2D::new(100.0, 50.0));

        // Tick 1
        let delta1 = scroller.tick().expect("Scroller must be active on tick 1");
        assert_eq!(delta1, Point2D::new(100.0, 50.0));
        assert_eq!(scroller.position(), Point2D::new(100.0, 50.0));
        // Velocity after friction 0.92
        assert!((scroller.velocity_x - 92.0).abs() < 1e-6);
        assert!((scroller.velocity_y - 46.0).abs() < 1e-6);

        // Tick 2
        let delta2 = scroller.tick().expect("Scroller must be active on tick 2");
        assert!((delta2.x - 92.0).abs() < 1e-6);
        assert!((delta2.y - 46.0).abs() < 1e-6);
        assert!((scroller.position_x - 192.0).abs() < 1e-6);
        assert!((scroller.position_y - 96.0).abs() < 1e-6);

        // Velocity after tick 2 friction: 92.0 * 0.92 = 84.64
        assert!((scroller.velocity_x - 84.64).abs() < 1e-6);
        assert!((scroller.velocity_y - 42.32).abs() < 1e-6);

        // Advance until stopped
        let mut ticks = 0;
        while scroller.tick().is_some() {
            ticks += 1;
            assert!(ticks < 1000, "Kinetic scroll must eventually halt");
        }

        assert!(!scroller.is_active());
        assert_eq!(scroller.velocity(), Point2D::ZERO);
    }

    #[test]
    fn test_kinetic_scroller_stop_and_reset() {
        let mut scroller = KineticScroller::new();
        scroller.fling(50.0, -30.0);
        scroller.tick();
        assert!(scroller.is_active());

        scroller.stop();
        assert!(!scroller.is_active());
        assert_eq!(scroller.velocity(), Point2D::ZERO);

        scroller.reset();
        assert_eq!(scroller.position(), Point2D::ZERO);
    }

    #[test]
    fn test_pinch_delta_calculations() {
        let init_p1 = Point2D::new(100.0, 100.0);
        let init_p2 = Point2D::new(200.0, 100.0); // distance = 100.0

        let prev_p1 = Point2D::new(100.0, 100.0);
        let prev_p2 = Point2D::new(200.0, 100.0);

        // Current points scaled out by 2x (distance = 200.0) and shifted by (10, 20)
        let curr_p1 = Point2D::new(60.0, 120.0);
        let curr_p2 = Point2D::new(260.0, 120.0); // midpoint = (160, 120)

        let pinch = PinchDelta::calculate(init_p1, init_p2, prev_p1, prev_p2, curr_p1, curr_p2);

        assert_eq!(pinch.distance, 200.0);
        assert_eq!(pinch.scale_factor, 2.0);
        assert_eq!(pinch.delta_scale, 2.0);
        assert_eq!(pinch.focal_center, Point2D::new(160.0, 120.0));
        assert_eq!(pinch.focal_delta, Point2D::new(10.0, 20.0)); // (160 - 150, 120 - 100)
        assert_eq!(pinch.rotation_delta, 0.0);
    }

    #[test]
    fn test_pinch_rotation_delta() {
        let init_p1 = Point2D::new(100.0, 100.0);
        let init_p2 = Point2D::new(200.0, 100.0); // angle = 0.0

        let prev_p1 = Point2D::new(100.0, 100.0);
        let prev_p2 = Point2D::new(200.0, 100.0);

        // Rotated 90 degrees (PI / 2)
        let curr_p1 = Point2D::new(100.0, 100.0);
        let curr_p2 = Point2D::new(100.0, 200.0);

        let pinch = PinchDelta::calculate(init_p1, init_p2, prev_p1, prev_p2, curr_p1, curr_p2);
        assert!((pinch.rotation_delta - (PI * 0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_gesture_recognizer_pan_and_kinetic_fling() {
        let mut recognizer = GestureRecognizer::new();
        let t0 = Instant::now();

        // Touch Down
        let ev1 = recognizer.handle_touch_down(1, 100.0, 100.0, t0);
        assert!(ev1.is_empty());
        assert_eq!(recognizer.active_touch_count(), 1);

        // Small motion within pan slop (8.0px) -> no pan event yet
        let t1 = t0 + Duration::from_millis(16);
        let ev2 = recognizer.handle_touch_move(1, 104.0, 100.0, t1);
        assert!(ev2.is_empty());

        // Motion exceeding pan slop -> Pan Began
        let t2 = t1 + Duration::from_millis(16);
        let ev3 = recognizer.handle_touch_move(1, 120.0, 100.0, t2);
        assert_eq!(ev3.len(), 1);
        match &ev3[0] {
            GestureEvent::Pan { state, translation, .. } => {
                assert_eq!(*state, GestureState::Began);
                assert_eq!(translation.x, 20.0);
            }
            _ => panic!("Expected Pan Began event"),
        }

        // Further motion -> Pan Updated
        let t3 = t2 + Duration::from_millis(16);
        let ev4 = recognizer.handle_touch_move(1, 160.0, 100.0, t3);
        assert_eq!(ev4.len(), 1);
        match &ev4[0] {
            GestureEvent::Pan { state, translation, delta, .. } => {
                assert_eq!(*state, GestureState::Updated);
                assert_eq!(translation.x, 60.0);
                assert_eq!(delta.x, 40.0);
            }
            _ => panic!("Expected Pan Updated event"),
        }

        // Touch Up -> Pan Ended + Swipe + Kinetic Scroller activated
        let t4 = t3 + Duration::from_millis(16);
        let ev5 = recognizer.handle_touch_up(1, 200.0, 100.0, t4);
        assert!(ev5.iter().any(|e| matches!(e, GestureEvent::Pan { state: GestureState::Ended, .. })));
        assert!(ev5.iter().any(|e| matches!(e, GestureEvent::Swipe { direction: SwipeDirection::Right, .. })));

        // Kinetic scrolling should be triggered
        assert!(recognizer.kinetic_scroller.is_active());
        let kinetic_event = recognizer.tick_kinetic().expect("Kinetic scroll event expected");
        match kinetic_event {
            GestureEvent::KineticScroll { delta, .. } => {
                assert!(delta.x > 0.0);
            }
            _ => panic!("Expected KineticScroll event"),
        }
    }

    #[test]
    fn test_gesture_recognizer_pinch_flow() {
        let mut recognizer = GestureRecognizer::new();
        let t0 = Instant::now();

        // Finger 1 down
        recognizer.handle_touch_down(1, 100.0, 100.0, t0);
        // Finger 2 down -> Pinch Began
        let ev2 = recognizer.handle_touch_down(2, 200.0, 100.0, t0);
        assert_eq!(ev2.len(), 1);
        match &ev2[0] {
            GestureEvent::Pinch { state, delta } => {
                assert_eq!(*state, GestureState::Began);
                assert_eq!(delta.scale_factor, 1.0);
                assert_eq!(delta.focal_center, Point2D::new(150.0, 100.0));
            }
            _ => panic!("Expected Pinch Began event"),
        }

        // Finger 2 moves outwards -> Pinch Updated
        let t1 = t0 + Duration::from_millis(16);
        let ev3 = recognizer.handle_touch_move(2, 300.0, 100.0, t1);
        assert_eq!(ev3.len(), 1);
        match &ev3[0] {
            GestureEvent::Pinch { state, delta } => {
                assert_eq!(*state, GestureState::Updated);
                assert_eq!(delta.scale_factor, 2.0); // 200px distance vs 100px initial
                assert_eq!(delta.focal_center, Point2D::new(200.0, 100.0));
            }
            _ => panic!("Expected Pinch Updated event"),
        }

        // Finger 1 lifts -> Pinch Ended
        let t2 = t1 + Duration::from_millis(16);
        let ev4 = recognizer.handle_touch_up(1, 100.0, 100.0, t2);
        assert!(ev4.iter().any(|e| matches!(e, GestureEvent::Pinch { state: GestureState::Ended, .. })));
    }

    #[test]
    fn test_gesture_recognizer_tap_and_double_tap() {
        let mut recognizer = GestureRecognizer::new();
        let t0 = Instant::now();

        // Tap 1
        recognizer.handle_touch_down(1, 50.0, 50.0, t0);
        let ev1 = recognizer.handle_touch_up(1, 50.0, 50.0, t0 + Duration::from_millis(50));
        assert_eq!(ev1.len(), 1);
        match &ev1[0] {
            GestureEvent::Tap { position, tap_count } => {
                assert_eq!(*position, Point2D::new(50.0, 50.0));
                assert_eq!(*tap_count, 1);
            }
            _ => panic!("Expected Tap event"),
        }

        // Tap 2 within double tap window (100ms later)
        let t1 = t0 + Duration::from_millis(150);
        recognizer.handle_touch_down(1, 51.0, 50.0, t1);
        let ev2 = recognizer.handle_touch_up(1, 51.0, 50.0, t1 + Duration::from_millis(50));
        assert_eq!(ev2.len(), 1);
        match &ev2[0] {
            GestureEvent::Tap { tap_count, .. } => {
                assert_eq!(*tap_count, 2);
            }
            _ => panic!("Expected Double Tap event"),
        }
    }

    #[test]
    fn test_gesture_recognizer_touch_cancel() {
        let mut recognizer = GestureRecognizer::new();
        let t0 = Instant::now();

        recognizer.handle_touch_down(1, 100.0, 100.0, t0);
        recognizer.handle_touch_move(1, 150.0, 100.0, t0 + Duration::from_millis(20));

        let cancel_ev = recognizer.handle_touch_cancel(1, t0 + Duration::from_millis(40));
        assert_eq!(cancel_ev.len(), 1);
        match &cancel_ev[0] {
            GestureEvent::Pan { state, .. } => {
                assert_eq!(*state, GestureState::Cancelled);
            }
            _ => panic!("Expected Pan Cancelled event"),
        }
        assert_eq!(recognizer.active_touch_count(), 0);
    }
}
