/// Configuration for shake detection and cursor enlargement.
///
/// These parameters control how sensitive the shake detection is
/// and how the cursor responds when a shake is detected.
pub struct Config {
    /// How far back in time (ms) to analyze mouse motion events.
    /// Only events within this window are considered for shake detection.
    /// A shorter window requires faster shaking. A longer window is more forgiving.
    pub time_window_ms: u32,

    /// Minimum number of direction reversals required to count as a shake.
    /// A reversal is when the cursor changes direction on either the X or Y axis.
    /// Normal mouse movement rarely produces more than 1-2 reversals in 500ms.
    pub min_reversals: u32,

    /// Minimum average velocity (pixels per second) required to count as a shake.
    /// This filters out slow, deliberate cursor adjustments that happen to reverse
    /// direction multiple times. Only fast, intentional shaking passes this threshold.
    pub min_velocity: f64,

    /// How long (ms) to wait after the last detected shake before restoring
    /// the cursor to its original size. Resets every time a new shake is detected,
    /// so continuous shaking keeps the cursor enlarged.
    pub cooldown_ms: u32,

    /// Cursor size in pixels when enlarged. The original size is read from the
    /// user's cursor theme settings on startup. Most cursor themes ship assets
    /// at 24, 32, 48, 64, and 96 pixels.
    pub enlarged_size: u32,
}

// Default ass the <Default> Values 
impl Default for Config {
    fn default() -> Self {
        Self {
            time_window_ms: 500,
            min_reversals: 3,
            min_velocity: 500.0,
            cooldown_ms: 2000,
            enlarged_size: 96,
        }
    }
}
