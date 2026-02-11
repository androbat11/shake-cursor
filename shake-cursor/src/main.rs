mod backend;
mod config;
mod detector;
mod x11_backend;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use backend::DisplayBackend;
use config::Config;
use detector::{CursorState, ShakeDetector};
use x11_backend::X11Backend;

fn main() {
    // Initialize logging (controlled by RUST_LOG env var)
    env_logger::init();

    // Build configuration with default thresholds
    let config = Config::default();
    let enlarged_size = config.enlarged_size;
    let cooldown_ms = config.cooldown_ms;

    // Create shake detector with the config
    let mut detector = ShakeDetector::new(config);

    // Create and connect the X11 backend
    let mut backend = X11Backend::new();
    if let Err(err) = backend.connect() {
        log::error!("Failed to start: {}", err);
        return;
    }
    log::info!("Connected to X server. Listening for mouse motion.");

    // Set up signal handler for clean shutdown (SIGTERM, SIGINT).
    // AtomicBool is checked each iteration of the event loop.
    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .expect("Failed to register SIGTERM handler");
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))
        .expect("Failed to register SIGINT handler");

    // Event loop: runs until shutdown signal or X server disconnect
    loop {
        // Check if a shutdown signal was received
        if shutdown.load(Ordering::Relaxed) {
            log::info!("Shutdown signal received.");
            break;
        }

        // Block until next mouse motion event (sleeps at 0% CPU)
        let event = match backend.next_motion_event() {
            Some(event) => event,
            None => {
                log::warn!("X server connection lost.");
                break;
            }
        };

        let timestamp = event.timestamp;
        log::debug!("Motion: x={}, y={}, t={}", event.x, event.y, timestamp);

        // Feed the event into the shake detector's ring buffer
        detector.record_motion(event);

        // State machine transitions based on shake detection
        match detector.state {
            CursorState::Idle => {
                if detector.is_shaking() {
                    log::info!("Shake detected, enlarging cursor.");
                    if let Err(err) = backend.set_cursor_size(enlarged_size) {
                        log::error!("Failed to enlarge cursor: {}", err);
                    }
                    detector.state = CursorState::Enlarged { since: timestamp };
                }
            }
            CursorState::Enlarged { since } => {
                if detector.is_shaking() {
                    // Still shaking — reset the cooldown timer
                    detector.state = CursorState::Enlarged { since: timestamp };
                } else if timestamp.saturating_sub(since) >= cooldown_ms {
                    // Cooldown expired — begin restoring
                    log::info!("Cooldown expired, restoring cursor.");
                    detector.state = CursorState::Restoring;
                }
            }
            CursorState::Restoring => {
                if let Err(err) = backend.set_cursor_size(24) {
                    log::error!("Failed to restore cursor: {}", err);
                }
                detector.state = CursorState::Idle;
            }
        }
    }

    // Clean shutdown: restore original cursor and disconnect
    log::info!("Restoring cursor and disconnecting.");
    let _ = backend.set_cursor_size(24);
    backend.disconnect();
}
