/// A single mouse motion event received from the display server.
///
/// This is the raw data that flows from X11 (or Wayland in the future)
/// into the shake detector. Each event captures where the cursor was
/// and when it was there.
pub struct MotionEvent {
    /// Cursor X position in pixels, relative to the root window origin (top-left corner).
    pub x: i16,

    /// Cursor Y position in pixels, relative to the root window origin (top-left corner).
    pub y: i16,

    /// Timestamp in milliseconds, as reported by the X server.
    /// This is a monotonic counter that resets when the server restarts,
    /// not a wall-clock time. Used to calculate velocity and evict old events.
    pub timestamp: u32,
}

/// The contract that any display server backend must fulfill.
///
/// This is the Strategy pattern. main.rs calls these methods without
/// knowing whether it's talking to X11 or Wayland. To add a new backend,
/// implement this trait — no existing code needs to change.
pub trait DisplayBackend {
    /// Connect to the display server and subscribe to mouse motion events.
    /// Returns an error if the connection fails (e.g., no X server running).
    fn connect(&mut self) -> Result<(), String>;

    /// Block until the next mouse motion event arrives.
    /// Returns None if the display server disconnects (e.g., Xorg crashed).
    fn next_motion_event(&mut self) -> Option<MotionEvent>;

    /// Change the cursor to the given size in pixels.
    /// Used both for enlarging (96px) and restoring (original size).
    fn set_cursor_size(&mut self, size: u32) -> Result<(), String>;

    /// Disconnect from the display server and free resources.
    fn disconnect(&mut self);
}
