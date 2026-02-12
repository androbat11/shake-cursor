use std::collections::VecDeque;

use crate::backend::MotionEvent;
use crate::config::Config;

/// State Machine pattern: tracks the current cursor state.
///
/// Transitions:
///   Idle → Enlarged        (shake detected)
///   Enlarged → Enlarged    (still shaking, reset cooldown)
///   Enlarged → Restoring   (no shake for cooldown_ms)
///   Restoring → Idle       (cursor restored)
pub enum CursorState {
    /// Cursor is at its normal size. Waiting for a shake.
    Idle,

    /// Cursor has been enlarged. `since` is the timestamp (ms) of
    /// the last detected shake, used to know when to start restoring.
    Enlarged { since: u32 },

    /// Cursor is being restored to normal size.
    Restoring,
}

/// Analyzes mouse motion events to detect shake gestures.
///
/// Stores recent motion events in a ring buffer (VecDeque) and evicts
/// entries older than the configured time window. On each new event,
/// it counts direction reversals and calculates average velocity to
/// determine if the user is shaking the cursor.
pub struct ShakeDetector {
    /// Ring Buffer pattern: rolling buffer of recent motion events.
    /// Old events (outside the time window) are removed from the front.
    /// New events are pushed to the back. This gives constant memory usage.
    pub events: VecDeque<MotionEvent>,

    /// State Machine: current cursor state and transitions.
    pub state: CursorState,

    /// Configuration thresholds that control detection sensitivity.
    pub config: Config,
}

impl ShakeDetector {
    /// Create a new detector with an empty event buffer and idle state.
    pub fn new(config: Config) -> Self {
        Self {
            events: VecDeque::new(),
            state: CursorState::Idle,
            config,
        }
    }

    /// Ring Buffer pattern: record a new motion event and evict stale ones.
    ///
    /// 1. Push the new event to the back of the buffer
    /// 2. Calculate the cutoff timestamp (newest event - time_window)
    /// 3. Pop events from the front that are older than the cutoff
    ///
    /// This keeps memory bounded: no matter how long the daemon runs,
    /// the buffer only holds events within the last time_window_ms.
    pub fn record_motion(&mut self, event: MotionEvent) {
        let cutoff = event.timestamp.saturating_sub(self.config.time_window_ms);

        self.events.push_back(event);

        while let Some(front) = self.events.front() {
            if front.timestamp < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Analyze the buffered events to determine if a shake is occurring.
    ///
    /// Walks through consecutive event pairs to:
    /// 1. Calculate direction (dx, dy) between each pair
    /// 2. Count direction reversals on X and Y axes independently
    /// 3. Sum the total distance traveled
    /// 4. Compute average velocity over the time span
    ///
    /// Returns true when: reversals >= min_reversals AND velocity >= min_velocity
    pub fn is_shaking(&self) -> bool {
        // Need at least 3 events to detect 1 reversal
        if self.events.len() < 3 {
            return false;
        }

        let mut x_reversals: u32 = 0;
        let mut y_reversals: u32 = 0;
        let mut total_distance: f64 = 0.0;
        let mut prev_dx: i16 = 0;
        let mut prev_dy: i16 = 0;

        for event_index in 1..self.events.len() {
            let prev = &self.events[event_index - 1];
            let curr = &self.events[event_index];

            let dx = curr.x - prev.x;
            let dy = curr.y - prev.y;

            // Euclidean distance between consecutive events
            // <Remove by myself the usage of "as">
            let distance = ((dx as f64).powi(2) + (dy as f64).powi(2)).sqrt();
            total_distance += distance;

            // Count sign changes starting from the second pair
            if event_index > 1 {
                if (prev_dx > 0 && dx < 0) || (prev_dx < 0 && dx > 0) {
                    x_reversals += 1;
                }
                if (prev_dy > 0 && dy < 0) || (prev_dy < 0 && dy > 0) {
                    y_reversals += 1;
                }
            }

            prev_dx = dx;
            prev_dy = dy;
        }

        // Time span between oldest and newest event
        let first_time = self.events.front().unwrap().timestamp;
        let last_time = self.events.back().unwrap().timestamp;
        let time_span = last_time - first_time;

        if time_span == 0 {
            return false;
        }

        // Average velocity in pixels per second
        let avg_velocity = (total_distance / time_span as f64) * 1000.0;

        // Use the axis with more reversals (shakes happen on one axis)
        let reversals = x_reversals.max(y_reversals);

        reversals >= self.config.min_reversals && avg_velocity >= self.config.min_velocity
    }
}
