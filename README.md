# shake-cursor

A Linux daemon that detects mouse cursor shaking and temporarily enlarges the cursor, making it easy to locate on screen. Inspired by the macOS "shake to find cursor" feature.

Built in Rust. Runs as a user-level systemd service. Currently targets X11 (Wayland support planned).

## How It Works

shake-cursor subscribes to mouse motion events on the X11 root window. When the user shakes the cursor, the daemon detects rapid direction reversals combined with high velocity and enlarges the cursor using the Xcursor theme system. After a cooldown period with no shaking, the cursor returns to its original size.

The daemon uses an event-driven architecture. It sleeps when the mouse is idle, consuming zero CPU. The kernel wakes it only when mouse motion events arrive.

### Shake Detection Algorithm

A shake is detected when both conditions are met within a 500ms rolling window:

- **Direction reversals >= 3** on either the X or Y axis
- **Average velocity >= 500 px/s** across all recorded motion events

This distinguishes deliberate shaking from normal mouse movement (too few reversals) and fast straight-line motion (no reversals).

## Architecture

### Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Protocol | X11 first, Wayland later | X11 exposes global cursor position. Wayland isolates clients from global input state. |
| Input strategy | Event-driven (MotionNotify) | Zero CPU when idle. Catches every mouse movement without polling. |
| Shake detection | Direction reversals + velocity threshold | Filters both slow adjustments and fast straight-line movements. |
| Cursor enlargement | Xcursor | Loads native cursor themes at arbitrary sizes. Simple API, low overhead. |
| Execution model | systemd user daemon | Starts on login, restarts on crash, managed with standard tooling. |

### Module Structure

```
shake-cursor/src/
    main.rs          Entry point, X11 connection, event loop, signal handling
    backend.rs       DisplayBackend trait, MotionEvent struct
    x11_backend.rs   X11 implementation of DisplayBackend
    detector.rs      Shake detection with rolling event buffer (ring buffer)
    config.rs        Configuration with builder pattern
```

**main.rs** connects to the X server, subscribes to MotionNotify on the root window, and runs the event loop. It delegates motion events to the detector and cursor operations to the backend.

**backend.rs** defines the `DisplayBackend` trait, abstracting display server operations behind a common interface. This is the Strategy pattern — adding Wayland support later requires implementing the trait without modifying existing code.

**detector.rs** maintains a `VecDeque`-based ring buffer of recent motion events, evicting entries older than the configured time window. On each new event, it counts direction reversals and calculates average velocity to determine if a shake is occurring.

**config.rs** provides a builder for configuration parameters, with sensible defaults that can be overridden via command-line arguments.


## Configuration

| Parameter | Default | Description |
|---|---|---|
| `time_window` | 500ms | Rolling window for motion event analysis |
| `min_reversals` | 3 | Minimum direction changes to qualify as a shake |
| `min_velocity` | 500 px/s | Minimum average velocity threshold |
| `cooldown` | 2000ms | Time without shaking before restoring cursor size |
| `enlarged_size` | 96px | Cursor size when enlarged |

## Dependencies

| Crate | Purpose |
|---|---|
| `x11rb` | Pure Rust X11 protocol client. Handles connection, events, and window operations. |
| `xcursor` | Cursor theme loading at arbitrary sizes. Pulled in transitively by `x11rb[cursor]`. |
| `signal-hook` | POSIX signal handling (SIGTERM, SIGINT) for clean daemon shutdown. |
| `log` | Logging facade. |
| `env_logger` | Log output configuration via `RUST_LOG` environment variable. |

## Build

```sh
cd shake-cursor
cargo build --release
```

## Install

```sh
# Copy binary
sudo cp target/release/shake-cursor /usr/local/bin/

# Install systemd user service
mkdir -p ~/.config/systemd/user
cp shake-cursor.service ~/.config/systemd/user/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now shake-cursor
```

## Usage

```sh
# Run directly (foreground, for development)
RUST_LOG=info cargo run

# Check daemon status
systemctl --user status shake-cursor

# View logs
journalctl --user -u shake-cursor -f

# Stop
systemctl --user stop shake-cursor

# Disable
systemctl --user disable shake-cursor
```

## Requirements

- Linux with X11 (Xorg)
- Rust 2024 edition (1.85+)
- A cursor theme that supports multiple sizes (most themes do)

## License

MIT
