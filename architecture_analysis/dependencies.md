# Dependencies

## Overview

shake-cursor needs four capabilities from external crates:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Capability              Crate                             │
│   ─────────────────────   ────────────────                  │
│   Talk to X server        x11rb                             │
│   Load/change cursors     x11rb (cursor feature) + xcursor  │
│   Handle shutdown signal  signal-hook                       │
│   Logging for daemon      log + env_logger                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 1. x11rb — X11 Protocol Client

**What it does:** Provides Rust bindings to the X11 protocol. This is how shake-cursor talks to the X server (Xorg) — connecting, subscribing to events, and manipulating cursors.

**Version:** 0.13.2 (August 2025)
**License:** MIT / Apache-2.0
**Repository:** https://github.com/psychon/x11rb

### Key Features

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   x11rb provides:                                                   │
│                                                                     │
│   • Pure Rust X11 connection (no C libraries needed)                │
│   • Full X11 core protocol support                                  │
│   • 20+ extension support (XFixes, XInput, Render, etc.)            │
│   • Built-in cursor loading (cursor feature flag)                   │
│   • Resource database reading (cursor theme detection)              │
│                                                                     │
│   Two connection backends:                                          │
│   ┌──────────────────┐  ┌──────────────────┐                        │
│   │ rust_connection   │  │ xcb_ffi          │                        
│   │ (pure Rust)       │  │ (wraps libxcb)   │                        │
│   │                   │  │                   │                        │
│   │ No C deps         │  │ Needs libxcb     │                        │
│   │ Slightly slower   │  │ Faster           │                        │
│   │ Simpler build     │  │ More complex     │                        │
│   └──────────────────┘  └──────────────────┘                        │
│                                                                     │
│   We'll use rust_connection (pure Rust, simpler).                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### What We Use From x11rb

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Operation                     x11rb API                           │
│   ───────────────────────────   ──────────────────────────────      │
│   Connect to X server           RustConnection::connect()           │
│   Get root window               conn.setup().roots[screen].root     │
│   Subscribe to mouse events     conn.change_window_attributes(      │
│                                     root, PointerMotionMask)        │
│   Wait for next event           conn.wait_for_event()  (blocks)     │
│   Load cursor from theme        cursor::Handle::load_cursor()       │
│   Apply cursor to root window   conn.change_window_attributes(      │
│                                     root, cursor=big_cursor)        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Feature Flags We Need

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Feature                Purpose                                    │
│   ────────────────────   ──────────────────────────────────         │
│   cursor                 Built-in cursor loading from Xcursor       │
│                          themes. Automatically enables:              │
│                          • render (X Render extension)               │
│                          • resource_manager (reads Xrdb settings)    │
│                          • xcursor (parses cursor theme files)       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Important Discovery: x11rb Has Built-in Cursor Support

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   x11rb's cursor module (enabled via "cursor" feature):             │
│                                                                     │
│   1. Reads X resource database to find:                             │
│      • Current cursor theme name (e.g., "Adwaita")                  │
│      • Current cursor size (e.g., 24px)                             │
│                                                                     │
│   2. Loads cursor from theme files:                                 │
│      handle.load_cursor(conn, "left_ptr") → Cursor ID              │
│                                                                     │
│   3. Returns an X11 Cursor ID ready to use with                     │
│      ChangeWindowAttributes                                         │
│                                                                     │
│   This means we DON'T need xcursor as a separate dependency.        │
│   x11rb pulls it in automatically.                                  │
│                                                                     │
│   However, to load a cursor at a DIFFERENT size than the default,   │
│   we may need to use the xcursor crate directly to:                 │
│   • Load cursor image data at a specific size                       │
│   • Access raw pixel data (RGBA/ARGB)                               │
│   • Create a cursor from that pixel data                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 2. xcursor — Cursor Theme File Parser

**What it does:** Loads XCursor theme files from the filesystem and parses them into pixel data. This is how we get cursor images at a specific size.

**Version:** 0.3.10 (June 2025)
**License:** MIT
**Repository:** https://github.com/esposm03/xcursor-rs

### Why We Need It

x11rb's cursor module loads cursors at the **default** system size. We need cursors at an **enlarged** size. xcursor gives us access to raw cursor image data so we can create a cursor at any size.

### Key API

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Step 1: Load the theme                                            │
│   ──────────────────────────────────────────────                    │
│   let theme = CursorTheme::load("Adwaita");                        │
│                                                                     │
│   Searches standard paths for cursor theme files:                   │
│   • ~/.icons/                                                       │
│   • /usr/share/icons/                                               │
│   • /usr/share/pixmaps/                                             │
│   Follows theme inheritance (child → parent → "default")            │
│                                                                     │
│                                                                     │
│   Step 2: Find cursor file path                                     │
│   ──────────────────────────────────────────────                    │
│   let path = theme.load_icon("left_ptr");                           │
│   // → Some("/usr/share/icons/Adwaita/cursors/left_ptr")            │
│                                                                     │
│                                                                     │
│   Step 3: Parse cursor file into images                             │
│   ──────────────────────────────────────────────                    │
│   let file_data = std::fs::read(path)?;                             │
│   let images = parse_xcursor(&file_data);                           │
│                                                                     │
│   Returns Vec<Image>, where each Image has:                         │
│   ┌─────────────────────────────────────────────┐                   │
│   │  Image {                                     │                   │
│   │      size: u32,          // nominal size      │                   │
│   │      width: u32,         // actual width      │                   │
│   │      height: u32,        // actual height     │                   │
│   │      xhot: u32,          // hotspot X         │                   │
│   │      yhot: u32,          // hotspot Y         │                   │
│   │      delay: u32,         // animation delay   │                   │
│   │      pixels_rgba: Vec<u8>, // RGBA pixel data │                   │
│   │      pixels_argb: Vec<u8>, // ARGB pixel data │                   │
│   │  }                                           │                   │
│   └─────────────────────────────────────────────┘                   │
│                                                                     │
│   A single cursor file contains MULTIPLE sizes (24, 32, 48, 64...) │
│   We pick the one closest to our desired enlarged size.             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### How xcursor and x11rb Work Together

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Normal cursor (x11rb handles everything):                        │
│                                                                    │
│   x11rb cursor::Handle → loads "left_ptr" at default size → done   │
│                                                                    │
│                                                                    │
│   Enlarged cursor (xcursor + x11rb together):                      │
│                                                                    │
│   xcursor                              x11rb                       │
│   ┌──────────────────────┐             ┌──────────────────────┐    │
│   │ 1. Load theme        │             │ 4. Create X cursor   │    │
│   │ 2. Find cursor file  │────────────►│    from pixel data   │    │
│   │ 3. Parse to pixels   │  pixel data │ 5. Apply to root     │    │
│   │    at large size     │             │    window             │    │
│   └──────────────────────┘             └──────────────────────┘    │
│                                                                    │
│   xcursor reads the files.                                         │
│   x11rb talks to the X server with the result.                     │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## 3. signal-hook — Unix Signal Handling

**What it does:** Provides safe handling of Unix signals (like SIGTERM) in Rust. This is how shake-cursor knows when systemd wants it to shut down.

**Version:** 0.4.3 (January 2026)
**License:** MIT / Apache-2.0
**Repository:** https://github.com/vorner/signal-hook

### Why We Need It

When systemd stops the daemon, it sends **SIGTERM**. We need to:
1. Catch SIGTERM
2. Restore the original cursor size
3. Disconnect from X server
4. Exit cleanly

Without signal handling, the daemon would die immediately and the cursor could be stuck at the enlarged size.

### Key API

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Two approaches:                                                   │
│                                                                     │
│   APPROACH A: Atomic flag (simple, good for us)                     │
│   ──────────────────────────────────────────────                    │
│   let shutdown = Arc::new(AtomicBool::new(false));                  │
│   signal_hook::flag::register(SIGTERM, Arc::clone(&shutdown));      │
│   signal_hook::flag::register(SIGINT, Arc::clone(&shutdown));       │
│                                                                     │
│   // In event loop:                                                 │
│   loop {                                                            │
│       if shutdown.load(Ordering::Relaxed) {                         │
│           // restore cursor, disconnect, exit                       │
│           break;                                                    │
│       }                                                             │
│       // handle events...                                           │
│   }                                                                 │
│                                                                     │
│   When SIGTERM arrives → flag becomes true → loop exits cleanly     │
│                                                                     │
│                                                                     │
│   APPROACH B: Iterator (dedicated thread)                           │
│   ──────────────────────────────────────────────                    │
│   let mut signals = Signals::new(&[SIGTERM, SIGINT])?;              │
│   for signal in signals.forever() {                                 │
│       match signal {                                                │
│           SIGTERM | SIGINT => break,                                │
│       }                                                             │
│   }                                                                 │
│                                                                     │
│   We'll use Approach A (atomic flag) — simpler, no extra thread.    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Signal Flow

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   systemd                    signal-hook              shake-cursor │
│      │                          │                          │       │
│      │ ── SIGTERM ─────────────►│                          │       │
│      │                          │                          │       │
│      │                          │ sets AtomicBool = true   │       │
│      │                          │                          │       │
│      │                          │         ┌────────────────┤       │
│      │                          │         │ Event loop     │       │
│      │                          │         │ checks flag    │       │
│      │                          │         │ → true!        │       │
│      │                          │         │ → restore      │       │
│      │                          │         │   cursor       │       │
│      │                          │         │ → disconnect   │       │
│      │                          │         │ → exit(0)      │       │
│      │                          │         └────────────────┘       │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## 4. log — Logging Facade

**What it does:** Provides standard logging macros for Rust. It's a **facade** — it defines the interface but doesn't do the actual logging. Think of it as the "language" for log messages.

**Version:** 0.4.29 (December 2025)
**License:** MIT / Apache-2.0
**Repository:** https://github.com/rust-lang/log (official Rust project)

### Key API

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Five log levels (from most to least severe):                      │
│                                                                     │
│   error!("Failed to connect to X server: {}", err);                 │
│   warn!("Cursor theme not found, using fallback");                  │
│   info!("shake-cursor started, listening for events");              │
│   debug!("MotionNotify at ({}, {})", x, y);                        │
│   trace!("Buffer has {} events, {} reversals", count, rev);         │
│                                                                     │
│                                                                     │
│   log is just the interface. It does nothing alone.                 │
│   You need an implementation (env_logger) to actually               │
│   print the messages somewhere.                                     │
│                                                                     │
│   Analogy:                                                          │
│   log        = the language (English)                               │
│   env_logger = the speaker (someone who talks in English)           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 5. env_logger — Logging Implementation

**What it does:** An implementation of the `log` facade that outputs to stderr and is configured via the `RUST_LOG` environment variable.

**Version:** 0.11.8 (April 2025)
**License:** MIT / Apache-2.0
**Repository:** https://github.com/rust-cli/env_logger

### Why We Need It

`log` defines the macros. `env_logger` makes them actually print somewhere. For a daemon, logs go to **journald** (systemd's log collector) via stderr.

### Key API

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Initialization (one line in main.rs):                             │
│                                                                     │
│   env_logger::init();                                               │
│                                                                     │
│   Or with a default level:                                          │
│                                                                     │
│   env_logger::Builder::from_env(                                    │
│       env_logger::Env::default().default_filter_or("info")          │
│   ).init();                                                         │
│                                                                     │
│                                                                     │
│   Controlled via RUST_LOG environment variable:                     │
│                                                                     │
│   RUST_LOG=info     → show info, warn, error                        │
│   RUST_LOG=debug    → show debug and above                          │
│   RUST_LOG=trace    → show everything                               │
│   RUST_LOG=shake_cursor=debug → debug only for our crate            │
│                                                                     │
│                                                                     │
│   Viewing daemon logs:                                              │
│                                                                     │
│   journalctl --user -u shake-cursor         → all logs              │
│   journalctl --user -u shake-cursor -f      → live tail             │
│                                                                     │
│   env_logger writes to stderr → systemd captures stderr             │
│   → journald stores it → journalctl reads it                        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   shake-cursor                                                      │
│       │                                                             │
│       ├── x11rb (with "cursor" feature)                             │
│       │     │                                                       │
│       │     ├── xcursor (pulled in automatically)                   │
│       │     ├── x11rb-protocol                                      │
│       │     └── gethostname                                         │
│       │                                                             │
│       ├── signal-hook                                               │
│       │     └── signal-hook-registry                                │
│       │         └── libc                                            │
│       │                                                             │
│       ├── log                                                       │
│       │                                                             │
│       └── env_logger                                                │
│             ├── log                                                 │
│             ├── humantime                                           │
│             └── regex (optional, for log filtering)                 │
│                                                                     │
│   Total: ~10-15 transitive dependencies                             │
│   All are well-maintained, widely used Rust crates.                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Cargo.toml

```toml
[package]
name = "shake-cursor"
version = "0.1.0"
edition = "2021"

[dependencies]
x11rb = { version = "0.13", features = ["cursor"] }
signal-hook = "0.4"
log = "0.4"
env_logger = "0.11"
```

Note: `xcursor` is **not** listed as a direct dependency. The `x11rb` "cursor" feature pulls it in automatically. If we need direct access to cursor pixel data for custom size loading, we can add it later:

```toml
xcursor = "0.3"   # only if needed for manual cursor size control
```

## Summary

| Crate | Purpose | Why this crate |
|-------|---------|---------------|
| **x11rb** | X11 connection, events, cursor | Pure Rust, no C deps, built-in cursor support |
| **xcursor** | (transitive) Cursor theme parsing | Pulled in by x11rb's cursor feature |
| **signal-hook** | Catch SIGTERM for clean shutdown | Safe, simple atomic flag API |
| **log** | Logging macros | Rust standard, zero-cost when disabled |
| **env_logger** | Log output to stderr/journald | Simple, configurable via RUST_LOG |
