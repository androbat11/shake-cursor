# shake-cursor Architecture

## Overview

**shake-cursor** is a Linux user daemon that detects when the user shakes the mouse cursor and temporarily enlarges it, making it easy to locate on screen. Inspired by the macOS "shake to find cursor" feature.

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   User shakes mouse → shake-cursor detects it → cursor gets bigger  │
│                                                                     │
│   After 2 seconds of no shaking → cursor returns to normal size     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Protocol** | X11 first, Wayland later | Fedora XFCE uses X11. X11 allows global cursor queries. |
| **Input strategy** | Event-driven (MotionNotify) | 0% CPU when idle, catches every movement. |
| **Shake detection** | Direction reversals + velocity | Distinguishes shakes from normal movement. |
| **Cursor enlargement** | Xcursor library | Simple, native look, low CPU. |
| **Execution model** | User daemon (systemd) | Auto-start on login, auto-restart on crash. |

## System Context

Where shake-cursor fits in the system:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Your Machine                                │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  systemd (user)                                              │   │
│   │     │                                                        │   │
│   │     └── shake-cursor (daemon)                                │   │
│   │              │                                               │   │
│   │              │ X11 Protocol                                  │   │
│   │              ▼                                               │   │
│   │         ┌─────────┐                                          │   │
│   │         │  Xorg   │ (X server)                               │   │
│   │         └────┬────┘                                          │   │
│   │              │                                               │   │
│   │         ┌────┴────┐                                          │   │
│   │         ▼         ▼                                          │   │
│   │    ┌────────┐ ┌────────┐                                     │   │
│   │    │ Mouse  │ │Monitor │                                     │   │
│   │    └────────┘ └────────┘                                     │   │
│   │                                                              │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│   shake-cursor talks to Xorg via X11 protocol to:                   │
│   1. Receive mouse motion events                                    │
│   2. Change cursor size when shake is detected                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Application Flow

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   1. STARTUP                                                       │
│      │                                                             │
│      ├── Connect to X server via DISPLAY env var                   │
│      ├── Get root window reference                                 │
│      ├── Subscribe to MotionNotify events on root window           │
│      ├── Load current cursor theme name and size                   │
│      └── Initialize rolling event buffer (empty)                   │
│                                                                    │
│   2. EVENT LOOP (runs forever)                                     │
│      │                                                             │
│      ├── Wait for next X11 event (BLOCKS / SLEEPS)                 │
│      │                                                             │
│      ├── On MotionNotify:                                          │
│      │   ├── Record (x, y, timestamp) in rolling buffer            │
│      │   ├── Remove events older than TIME_WINDOW (500ms)          │
│      │   ├── Run shake detection algorithm                         │
│      │   │   ├── Count direction reversals on X and Y axes         │
│      │   │   ├── Calculate average velocity                        │
│      │   │   └── If reversals >= 3 AND velocity >= 500 px/s:      │
│      │   │       └── SHAKE DETECTED                                │
│      │   │                                                         │
│      │   ├── If SHAKE DETECTED and cursor is normal size:          │
│      │   │   ├── Load cursor theme at enlarged size (96px)         │
│      │   │   ├── Apply enlarged cursor to root window              │
│      │   │   └── Record enlargement timestamp                      │
│      │   │                                                         │
│      │   └── If cursor is enlarged and no shake for COOLDOWN:      │
│      │       ├── Load cursor theme at original size (24px)         │
│      │       └── Apply original cursor to root window              │
│      │                                                             │
│      ├── On SIGTERM (shutdown signal):                             │
│      │   ├── Restore original cursor size                          │
│      │   ├── Disconnect from X server                              │
│      │   └── Exit                                                  │
│      │                                                             │
│      └── On X server disconnect (Xorg died):                      │
│          └── Exit (systemd will restart us)                        │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Module Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   shake-cursor/src/                                                 │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  main.rs                                                     │   │
│   │                                                              │   │
│   │  Entry point. Responsibilities:                              │   │
│   │  • Parse configuration (thresholds)                          │   │
│   │  • Initialize X11 connection                                 │   │
│   │  • Set up signal handlers (SIGTERM)                          │   │
│   │  • Run the event loop                                        │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                         │                                           │
│              ┌──────────┴──────────┐                                 │
│              ▼                     ▼                                 │
│   ┌──────────────────┐  ┌──────────────────┐                        │
│   │  detector.rs     │  │  cursor.rs       │                        │
│   │                  │  │                  │                        │
│   │  Shake detection │  │  Cursor control  │                        │
│   │                  │  │                  │                        │
│   │  • Store motion  │  │  • Load cursor   │                        │
│   │    events in     │  │    theme at      │                        │
│   │    rolling buffer│  │    given size    │                        │
│   │                  │  │                  │                        │
│   │  • Count         │  │  • Apply cursor  │                        │
│   │    direction     │  │    to root       │                        │
│   │    reversals     │  │    window        │                        │
│   │                  │  │                  │                        │
│   │  • Calculate     │  │  • Restore       │                        │
│   │    velocity      │  │    original      │                        │
│   │                  │  │    cursor size   │                        │
│   │  • Return        │  │                  │                        │
│   │    is_shaking    │  │                  │                        │
│   └──────────────────┘  └──────────────────┘                        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### main.rs — Entry Point and Event Loop

```
Responsibilities:
  1. Connect to X server
  2. Subscribe to MotionNotify on root window
  3. Run event loop:
     - Receive events
     - Pass motion events to detector
     - If shake detected → tell cursor module to enlarge
     - If cooldown expired → tell cursor module to restore
  4. Handle SIGTERM → restore cursor → exit
```

### detector.rs — Shake Detection

```
Responsibilities:
  1. Maintain rolling buffer of MotionEvent { x, y, timestamp }
  2. On each new event:
     a. Add to buffer
     b. Remove events older than TIME_WINDOW
     c. Calculate direction reversals on X and Y axes
     d. Calculate average velocity
     e. Return whether shake is detected

Public interface:
  ShakeDetector::new(config) → ShakeDetector
  ShakeDetector::record_motion(x, y, timestamp) → ()
  ShakeDetector::is_shaking() → bool
```

### cursor.rs — Cursor Size Control

```
Responsibilities:
  1. Read current cursor theme and size from X server
  2. Load cursor at enlarged size using Xcursor
  3. Apply cursor to root window using XDefineCursor
  4. Restore original cursor

Public interface:
  CursorManager::new(display, root_window) → CursorManager
  CursorManager::enlarge() → ()
  CursorManager::restore() → ()
  CursorManager::is_enlarged() → bool
```

## Data Flow

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   X Server                                                         │
│      │                                                             │
│      │  MotionNotify { x: 540, y: 300, time: 12345 }              │
│      │                                                             │
│      ▼                                                             │
│   main.rs (event loop)                                             │
│      │                                                             │
│      │  detector.record_motion(540, 300, 12345)                    │
│      │                                                             │
│      ▼                                                             │
│   detector.rs                                                      │
│      │                                                             │
│      ├── Adds event to rolling buffer                              │
│      ├── Removes old events (> 500ms ago)                          │
│      ├── Counts reversals: 4                                       │
│      ├── Calculates velocity: 2700 px/sec                          │
│      └── Returns: is_shaking = true                                │
│                                                                    │
│      ▼                                                             │
│   main.rs                                                          │
│      │                                                             │
│      │  if is_shaking && !cursor.is_enlarged():                    │
│      │      cursor.enlarge()                                       │
│      │                                                             │
│      ▼                                                             │
│   cursor.rs                                                        │
│      │                                                             │
│      ├── Loads "Adwaita" cursor at 96px                            │
│      ├── Calls XDefineCursor(display, root, big_cursor)            │
│      └── Cursor is now enlarged on screen                          │
│                                                                    │
│                                                                    │
│   ... 2 seconds later, no more shaking ...                         │
│                                                                    │
│   main.rs                                                          │
│      │                                                             │
│      │  if !is_shaking && cursor.is_enlarged()                     │
│      │     && time_since_last_shake > COOLDOWN:                    │
│      │      cursor.restore()                                       │
│      │                                                             │
│      ▼                                                             │
│   cursor.rs                                                        │
│      │                                                             │
│      ├── Loads "Adwaita" cursor at 24px (original)                 │
│      ├── Calls XDefineCursor(display, root, normal_cursor)         │
│      └── Cursor is back to normal                                  │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## State Machine

The daemon has three states:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│                       ┌──────────┐                                 │
│              start ──►│  IDLE    │                                 │
│                       │          │                                 │
│                       │ Cursor:  │                                 │
│                       │ normal   │                                 │
│                       │ size     │                                 │
│                       └────┬─────┘                                 │
│                            │                                       │
│                            │ shake detected                        │
│                            ▼                                       │
│                       ┌──────────┐                                 │
│                       │ ENLARGED │                                 │
│                       │          │◄────────┐                       │
│                       │ Cursor:  │         │ still shaking         │
│                       │ big size │─────────┘ (reset cooldown)      │
│                       └────┬─────┘                                 │
│                            │                                       │
│                            │ no shake for COOLDOWN (2s)            │
│                            ▼                                       │
│                       ┌──────────┐                                 │
│                       │RESTORING │                                 │
│                       │          │                                 │
│                       │ Cursor:  │                                 │
│                       │ back to  │                                 │
│                       │ normal   │                                 │
│                       └────┬─────┘                                 │
│                            │                                       │
│                            │ done                                  │
│                            ▼                                       │
│                       ┌──────────┐                                 │
│                       │  IDLE    │ (back to waiting)               │
│                       └──────────┘                                 │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Configuration

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Parameter          Default     Description                │
│   ────────────────   ────────    ──────────────────────     │
│   time_window        500ms       Event analysis window      │
│   min_reversals      3           Direction changes needed   │
│   min_velocity       500 px/s    Speed threshold            │
│   cooldown           2000ms      Time before restoring size │
│   enlarged_size      96px        Cursor size when enlarged  │
│                                                             │
│   These can be passed as command-line arguments              │
│   or read from a config file later.                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Rust Dependencies

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Crate              Purpose                                        │
│   ─────────────────  ────────────────────────────────────           │
│   x11rb              X11 protocol client (connect, events,          │
│                      window operations). Pure Rust.                  │
│                                                                     │
│   xcursor            Load cursor themes at different sizes.         │
│                      Works with Xcursor theme format.               │
│                                                                     │
│   signal-hook        Handle SIGTERM for clean shutdown.             │
│                                                                     │
│   log + env_logger   Logging for daemon (goes to journald).        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## X11 Operations Used

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Operation                  Purpose                                │
│   ────────────────────────   ───────────────────────────────        │
│   XOpenDisplay               Connect to X server                    │
│   DefaultRootWindow          Get root window                        │
│   XSelectInput               Subscribe to MotionNotify on root      │
│     (PointerMotionMask)                                             │
│   XNextEvent                 Wait for next event (blocks/sleeps)    │
│   XcursorLibraryLoadCursor   Load cursor theme at specific size     │
│   XDefineCursor              Apply cursor to root window            │
│   XFreeCursor                Free old cursor resource               │
│   XCloseDisplay              Disconnect from X server               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Systemd Service

```
File: ~/.config/systemd/user/shake-cursor.service

[Unit]
Description=Shake cursor to enlarge it
After=graphical-session.target

[Service]
ExecStart=/usr/local/bin/shake-cursor
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
```

```
Commands:
  systemctl --user enable --now shake-cursor   # Enable and start
  systemctl --user status shake-cursor         # Check status
  systemctl --user stop shake-cursor           # Stop
  journalctl --user -u shake-cursor -f         # View live logs
```

## Build and Install

```
# Build
cd shake-cursor
cargo build --release

# Install binary
cp target/release/shake-cursor /usr/local/bin/

# Install service file
mkdir -p ~/.config/systemd/user
cp shake-cursor.service ~/.config/systemd/user/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now shake-cursor
```

## Error Handling

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Error                           Response                          │
│   ──────────────────────────────  ─────────────────────────         │
│   Cannot connect to X server      Log error, exit.                  │
│                                   systemd restarts after 3s.        │
│                                                                     │
│   X server disconnects            Exit cleanly.                     │
│   (Xorg crashed/restarted)        systemd restarts after 3s.        │
│                                                                     │
│   Cannot load cursor theme        Log warning, use fallback         │
│                                   cursor or skip enlargement.       │
│                                                                     │
│   SIGTERM received                Restore original cursor,          │
│                                   disconnect, exit.                 │
│                                                                     │
│   SIGINT received (Ctrl+C)        Same as SIGTERM.                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Project File Structure

```
shake-cursor/
├── architecture_analysis/
│   ├── main.md                 ← This file (architecture)
│   ├── display_server.md       ← What is a display server
│   ├── display_server_protocol.md ← What is a protocol
│   ├── window_system.md        ← What is a window system
│   ├── X11.md                  ← X11 protocol details
│   ├── wayland.md              ← Wayland protocol details
│   ├── daemon.md               ← What is a daemon
│   ├── cpu_on_event.md         ← Events and CPU efficiency
│   └── shake_algorithms.md     ← Shake detection algorithm
│
├── shake-cursor/
│   ├── Cargo.toml              ← Rust dependencies
│   ├── Cargo.lock
│   └── src/
│       ├── main.rs             ← Entry point, event loop
│       ├── detector.rs         ← Shake detection logic
│       └── cursor.rs           ← Cursor enlargement via Xcursor
│
└── shake-cursor.service        ← systemd service file
```

## Summary

shake-cursor is a minimal, focused daemon:

1. **Connects** to X server on startup
2. **Listens** for mouse motion events (sleeps when idle, 0% CPU)
3. **Analyzes** motion for shake pattern (direction reversals + velocity)
4. **Enlarges** cursor via Xcursor when shake detected
5. **Restores** cursor after cooldown period
6. **Shuts down** cleanly on SIGTERM

Three source files. One daemon. One job.
