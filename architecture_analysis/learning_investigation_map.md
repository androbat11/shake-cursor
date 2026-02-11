# Learning Investigation Map

Every concept that must be understood to build shake-cursor with confidence.
Organized from foundational (Layer 1) to application-specific (Layer 4).
Each topic links to the file(s) where it appears.

---

## Layer 1: Linux System Fundamentals

These concepts exist independently of your project. You need them to understand
why anything in shake-cursor works the way it does.

### 1.1 How a Linux Graphical Desktop Works

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 1 | What is a window system? | The full stack: display server + window manager + compositor + protocol working together | `window_system.md` |
| 2 | What is a display server? | The central program that owns the screen and input hardware, routes events to apps | `display_server.md` |
| 3 | What is a window manager? | Positions windows, draws decorations, handles workspaces (i3, Openbox, Mutter) | `window_system.md` |
| 4 | What is a compositor? | Combines all window buffers into a single screen image, applies transparency/shadows | `window_system.md` |
| 5 | How do these three relate on X11? | Three **separate** processes that coordinate through the X11 protocol | `window_system.md` |
| 6 | How do these three relate on Wayland? | One **unified** process (the compositor does everything) | `window_system.md`, `wayland.md` |

### 1.2 Display Server Protocols

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 7 | What is a display server protocol? | A contract/language defining exact message formats between apps and the display server | `display_server_protocol.md` |
| 8 | Protocol message types | Requests (client→server), Replies (server→client), Events (server→client), Errors | `display_server_protocol.md` |
| 9 | Protocol message format | Binary structure: opcode + length + sequence number + payload | `display_server_protocol.md` |
| 10 | Connection lifecycle | Connect → create window → show → receive events → destroy → disconnect | `display_server_protocol.md` |
| 11 | Client libraries | Apps don't write raw protocol bytes — they use libraries (x11rb, Xlib, wayland-client) | `display_server_protocol.md` |

### 1.3 X11 Protocol (Your Primary Target)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 12 | X11 client-server model | Server = Xorg (owns hardware). Client = apps. Naming feels backwards because server "serves" display resources | `X11.md` |
| 13 | The DISPLAY environment variable | `DISPLAY=:0` tells clients where to find the X server (Unix socket `/tmp/.X11-unix/X0`) | `X11.md` |
| 14 | Root window | The desktop background, parent of all windows. Subscribing to events on root = global monitoring | `X11.md` |
| 15 | Window hierarchy | Root → top-level windows → child windows. Everything on screen is a window in a tree | `X11.md` |
| 16 | X11 input handling | X server receives hardware events, converts to X11 events, delivers to the window under cursor | `X11.md` |
| 17 | X11 global state | Any client can query cursor position, window list, keyboard state — no permissions needed | `X11.md` |
| 18 | X11 security model (or lack thereof) | All clients are fully trusted. Any app can spy on any other app. Great for tools, terrible for security | `X11.md` |
| 19 | Key X11 requests | CreateWindow, MapWindow, QueryPointer, ChangeWindowAttributes, XDefineCursor | `X11.md`, `main.md` |
| 20 | Key X11 events | MotionNotify, ButtonPress, KeyPress, Expose, ConfigureNotify | `X11.md`, `display_server_protocol.md` |
| 21 | X11 extensions | XInput2 (advanced input), XFIXES (cursor utilities), XTest (fake input) | `X11.md` |
| 22 | Network transparency | X11 apps can run on machine A and display on machine B — unique to X11 | `X11.md` |

### 1.4 Wayland Protocol (Future Target)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 23 | Why Wayland was created | X11's security flaws, complexity from 40 years of extensions, performance overhead | `wayland.md` |
| 24 | The compositor model | One unified process replaces Xorg + WM + compositor | `wayland.md` |
| 25 | Client-side rendering | Apps render their own pixels into shared memory buffers, compositor just combines them | `wayland.md` |
| 26 | Wayland object protocol | Object-oriented: wl_display → wl_registry → wl_compositor → wl_surface → wl_buffer | `wayland.md` |
| 27 | Wayland security isolation | Each app has its own private channel to the compositor. No app can see another's events | `wayland.md` |
| 28 | No global cursor position | `wl_pointer.motion()` gives coordinates relative to YOUR window only. No global query exists | `wayland.md` |
| 29 | Why shake-cursor is hard on Wayland | Cursor leaves your window → you stop getting events → gaps in data → can't detect shake | `wayland.md` |
| 30 | Wayland workarounds | XDG Portal, compositor extensions, libei, XWayland, invisible overlay | `wayland.md` |
| 31 | wlroots-based compositors | Sway, Hyprland share common extension protocols. GNOME (Mutter) and KDE (KWin) are different | `wayland.md` |

### 1.5 Processes and Daemons

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 32 | What is a daemon? | Background process that runs continuously without user interaction (like a security guard) | `daemon.md` |
| 33 | Regular program vs daemon | Terminal-attached + user-interactive vs background + auto-managed + survives logout | `daemon.md` |
| 34 | Common Linux daemons | sshd, NetworkManager, bluetoothd, pipewire, cupsd, Xorg — the "d" suffix convention | `daemon.md` |
| 35 | System daemons vs user daemons | System = run as root, start on boot. User = run as your account, start on login | `daemon.md` |
| 36 | Why shake-cursor is a user daemon | Only needs X server access (user-level), only makes sense with a graphical session | `daemon.md` |

### 1.6 systemd

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 37 | What systemd does | Starts, stops, restarts, and monitors daemons. Collects logs | `daemon.md` |
| 38 | Service file anatomy | `[Unit]` (description, ordering), `[Service]` (command, restart policy), `[Install]` (when to start) | `daemon.md`, `main.md` |
| 39 | After=graphical-session.target | Don't start shake-cursor before Xorg is running | `daemon.md` |
| 40 | Restart=on-failure + RestartSec=3 | If daemon crashes, systemd automatically restarts it after 3 seconds | `daemon.md` |
| 41 | systemctl commands | enable, start, stop, status, daemon-reload — managing your daemon | `daemon.md` |
| 42 | journalctl | Where daemon logs go. `journalctl --user -u shake-cursor -f` for live tail | `daemon.md`, `dependencies.md` |

### 1.7 Events and CPU Efficiency

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 43 | What is an event? | A notification that something happened: hardware events, software events, window system events | `cpu_on_event.md` |
| 44 | The event lifecycle | Mouse hardware → kernel (evdev) → display server (X11 event) → application | `cpu_on_event.md` |
| 45 | The event loop pattern | `loop { event = wait(); handle(event); }` — sleep until notified, handle, repeat | `cpu_on_event.md` |
| 46 | Blocking/sleeping | When a program calls `wait_for_event()`, the kernel removes it from the CPU run queue. Zero CPU used | `cpu_on_event.md` |
| 47 | Polling vs event-driven | Polling: you ask repeatedly (wastes CPU). Event-driven: kernel wakes you (efficient) | `cpu_on_event.md` |
| 48 | Why event-driven uses less CPU | The kernel is already watching hardware for ALL programs. Your sleep adds zero extra work | `cpu_on_event.md` |
| 49 | System call: `read()` on a socket | `wait_for_event()` translates to a blocking `read()` on the X11 Unix socket. Kernel manages the sleep/wake | `cpu_on_event.md` |
| 50 | CPU usage comparison | Polling: 100% of iterations run, ~3% useful. Event-driven: 100% of wakeups are useful | `cpu_on_event.md` |

---

## Layer 2: Design Patterns

These are general software engineering patterns. Understanding them is
independent of this project, but this project uses all five.

### 2.1 State Machine Pattern

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 51 | What is a state machine? | A system that can be in exactly one state at a time, with explicit transitions between states | `design_patterns.md` |
| 52 | States in shake-cursor | IDLE (normal cursor) → ENLARGED (big cursor) → IDLE (restored) | `design_patterns.md`, `main.md` |
| 53 | Transitions | shake detected → enlarge. Cooldown expired → restore. Still shaking → reset cooldown | `design_patterns.md` |
| 54 | Rust enum for states | `enum CursorState { Idle, Enlarged { since: u64 } }` — data embedded in variants | `design_patterns.md` |
| 55 | Why not scattered booleans? | `let mut is_big = false; let mut last_shake = 0;` is error-prone. Enum makes invalid states unrepresentable | `design_patterns.md` |

### 2.2 Observer Pattern

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 56 | What is the observer pattern? | Subject notifies observers when something happens. Observers register interest, then react | `design_patterns.md` |
| 57 | X11 as observer pattern | Subject = X server. Observer = shake-cursor. Subscribe = `XSelectInput(PointerMotionMask)`. Notification = MotionNotify | `design_patterns.md` |
| 58 | Subscribe vs poll | Subscribe once, then sleep. vs. ask repeatedly in a loop | `design_patterns.md`, `cpu_on_event.md` |

### 2.3 Strategy Pattern

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 59 | What is the strategy pattern? | Interchangeable implementations behind a common interface. Caller doesn't know which one is running | `design_patterns.md`, `design_patterns/strategy.md` |
| 60 | Rust traits as strategies | `trait DisplayBackend { fn next_motion_event(); fn enlarge_cursor(); fn restore_cursor(); }` | `design_patterns/strategy.md` |
| 61 | Static vs dynamic dispatch | Generics (`<B: DisplayBackend>`) = compile-time choice, faster. `dyn DisplayBackend` = runtime choice, flexible | `design_patterns/strategy.md` |
| 62 | Why dynamic dispatch here | Backend chosen at runtime based on `DISPLAY` env var. Overhead is negligible (one pointer lookup per event) | `design_patterns/strategy.md` |
| 63 | Open/Closed principle | Adding Wayland = new file. Zero changes to existing X11 code, detector, or main loop | `design_patterns/strategy.md` |

### 2.4 Ring Buffer Pattern

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 64 | What is a ring buffer? | Fixed-size structure where new data overwrites the oldest. Memory never grows | `design_patterns.md`, `design_patterns/ring_buffer.md` |
| 65 | The modulo trick | `write_pos = (write_pos + 1) % capacity` wraps around: 0,1,2,3,0,1,2,3... | `design_patterns/ring_buffer.md` |
| 66 | O(1) operations | Push, read newest, read oldest — all constant time. No searching or shifting | `design_patterns/ring_buffer.md` |
| 67 | VecDeque as ring buffer | `pop_front()` (remove oldest) + `push_back()` (add newest) = ring buffer behavior | `design_patterns/ring_buffer.md` |
| 68 | Time-based pruning | Instead of fixed capacity, prune events older than 500ms. Bounded by time, not count | `design_patterns/ring_buffer.md`, `shake_algorithms.md` |
| 69 | Why critical for daemons | A 24/7 daemon with unbounded Vec would leak memory. Ring buffer keeps it constant forever | `design_patterns/ring_buffer.md` |

### 2.5 Builder Pattern

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 70 | What is the builder pattern? | Construct complex objects step by step with method chaining. Clear, self-documenting parameters | `design_patterns.md` |
| 71 | Why not `Config::new(500, 3, 500.0, 2000, 96)` | Positional args are unreadable. Builder names every parameter explicitly | `design_patterns.md` |
| 72 | Default values | `ConfigBuilder::default()` fills all fields. Override only what you need | `design_patterns.md` |
| 73 | Method chaining in Rust | `fn time_window_ms(mut self, ms: u64) -> Self` takes ownership and returns self | `design_patterns.md` |

---

## Layer 3: Rust Crates and APIs

The specific libraries shake-cursor depends on.

### 3.1 x11rb (X11 Protocol Client)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 74 | What x11rb provides | Pure Rust X11 client. Connects, subscribes to events, manipulates windows/cursors | `dependencies.md` |
| 75 | RustConnection vs xcb_ffi | Pure Rust (no C deps, simpler build) vs wrapping libxcb (faster, needs C lib) | `dependencies.md` |
| 76 | `RustConnection::connect()` | Connects to X server via DISPLAY env var. Returns connection + screen number | `dependencies.md` |
| 77 | `conn.setup().roots[screen].root` | How to get the root window ID after connecting | `dependencies.md` |
| 78 | `conn.change_window_attributes()` | Subscribe to events (PointerMotionMask) and change cursor (set cursor attribute) | `dependencies.md` |
| 79 | `conn.wait_for_event()` | Blocks until next X11 event. This is the sleep point. 0% CPU when idle | `dependencies.md` |
| 80 | The "cursor" feature flag | Enables built-in cursor loading. Auto-pulls xcursor, render, resource_manager | `dependencies.md` |
| 81 | `cursor::Handle::load_cursor()` | Loads cursor from theme at default size. Returns X11 Cursor ID | `dependencies.md` |

### 3.2 xcursor (Cursor Theme Parser)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 82 | Why xcursor is needed separately | x11rb loads cursors at default size. We need enlarged size. xcursor gives raw pixel data at any size | `dependencies.md` |
| 83 | `CursorTheme::load("Adwaita")` | Searches ~/.icons/, /usr/share/icons/ for cursor theme files | `dependencies.md` |
| 84 | `theme.load_icon("left_ptr")` | Finds the cursor file path for a specific cursor name | `dependencies.md` |
| 85 | `parse_xcursor(&file_data)` | Parses binary cursor file into Vec<Image> with size, width, height, hotspot, pixel data | `dependencies.md` |
| 86 | Multiple sizes in one file | A single cursor file contains 24px, 32px, 48px, 64px etc. Pick closest to desired size | `dependencies.md` |
| 87 | xcursor + x11rb pipeline | xcursor reads files → gets pixels → x11rb sends pixels to X server → X server creates cursor | `dependencies.md` |

### 3.3 signal-hook (Unix Signal Handling)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 88 | Why signal handling is needed | systemd sends SIGTERM to stop the daemon. Without handling it, cursor stays enlarged | `dependencies.md` |
| 89 | Atomic flag approach | `Arc<AtomicBool>` set to true on SIGTERM. Event loop checks the flag each iteration | `dependencies.md` |
| 90 | `signal_hook::flag::register()` | Registers a signal handler that sets the atomic bool. Safe, no extra thread needed | `dependencies.md` |
| 91 | Signal flow | systemd → SIGTERM → signal-hook sets flag → event loop sees flag → restore cursor → exit | `dependencies.md` |

### 3.4 log + env_logger (Logging)

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 92 | log as a facade | Defines macros (error!, warn!, info!, debug!, trace!) but doesn't output anything alone | `dependencies.md` |
| 93 | env_logger as implementation | Makes log macros actually print to stderr. Controlled via RUST_LOG env var | `dependencies.md` |
| 94 | stderr → journald | Daemon's stderr is captured by systemd → stored in journald → viewed with journalctl | `dependencies.md` |
| 95 | RUST_LOG filtering | `RUST_LOG=info` shows info+warn+error. `RUST_LOG=shake_cursor=debug` for crate-specific debug | `dependencies.md` |

---

## Layer 4: The Shake Detection Algorithm

The core logic that makes the project work.

### 4.1 Input Data

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 96 | MotionEvent struct | `{ x: i32, y: i32, timestamp: u64 }` — position in pixels, time in milliseconds | `shake_algorithms.md` |
| 97 | Where events come from | X server sends MotionNotify on every mouse movement. Contains root_x, root_y, time | `shake_algorithms.md`, `X11.md` |
| 98 | Rolling buffer of events | VecDeque storing the last 500ms of MotionEvents. Pruned on every new event | `shake_algorithms.md` |

### 4.2 Direction Reversals

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 99 | Computing deltas | `dx = current.x - previous.x`. Positive = right, negative = left, zero = no movement | `shake_algorithms.md` |
| 100 | What is a reversal | The sign of dx flips: `(prev_dx > 0 && dx < 0) OR (prev_dx < 0 && dx > 0)` | `shake_algorithms.md` |
| 101 | Independent axes | Count x_reversals and y_reversals separately | `shake_algorithms.md` |
| 102 | Why max() not sum() | Shaking is typically on one axis. Horizontal shake has high X reversals, ~0 Y reversals. Max captures dominant axis | `shake_algorithms.md` |
| 103 | Minimum 3 events needed | Need at least 2 deltas to detect 1 reversal. 3 events = 2 deltas = minimum for a sign change | `shake_algorithms.md` |

### 4.3 Velocity

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 104 | Per-segment distance | `sqrt(dx^2 + dy^2)` — Euclidean distance between consecutive events | `shake_algorithms.md` |
| 105 | Total distance | Sum of all per-segment distances across the time window | `shake_algorithms.md` |
| 106 | Average velocity | `(total_distance / time_span_ms) * 1000` → pixels per second | `shake_algorithms.md` |
| 107 | Why velocity matters | Filters out slow careful adjustments that happen to have many reversals | `shake_algorithms.md` |

### 4.4 The Complete Detection

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 108 | The two-condition formula | `is_shaking = (reversals >= 3) AND (velocity >= 500 px/s)` | `shake_algorithms.md` |
| 109 | Why BOTH conditions | Velocity alone = false positive on fast straight lines. Reversals alone = false positive on slow nudging | `shake_algorithms.md` |
| 110 | The four quadrants | High velocity + high reversals = shake. All other combinations = not shake | `shake_algorithms.md` |
| 111 | Time window pruning | Remove events older than 500ms before running detection. Keeps data current | `shake_algorithms.md` |

---

## Layer 5: Application Architecture

How all the pieces wire together into a working daemon.

### 5.1 Module Structure

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 112 | config.rs responsibility | Define Config struct + ConfigBuilder with defaults. No I/O, no dependencies | `main.md` |
| 113 | detector.rs responsibility | ShakeDetector with EventBuffer (VecDeque). `record_motion()` + `is_shaking()`. Pure algorithm | `main.md` |
| 114 | backend.rs responsibility | Define `DisplayBackend` trait + `MotionEvent` struct. The interface contract | `main.md`, `design_patterns/strategy.md` |
| 115 | x11_backend.rs responsibility | `X11Backend` implements `DisplayBackend`. All x11rb code lives here and only here | `main.md`, `design_patterns/strategy.md` |
| 116 | main.rs responsibility | Wire everything: init → connect → event loop → state machine → signal handling → cleanup | `main.md` |

### 5.2 Application Flow

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 117 | Startup sequence | Connect to X server → get root window → subscribe to MotionNotify → init event buffer | `main.md` |
| 118 | Event loop | `wait_for_event()` → if MotionNotify → `detector.record_motion()` → check `is_shaking()` | `main.md` |
| 119 | State transitions | IDLE + shake → enlarge cursor. ENLARGED + still shaking → reset cooldown. ENLARGED + cooldown expired → restore | `main.md` |
| 120 | Shutdown sequence | SIGTERM → restore cursor → disconnect from X server → exit cleanly | `main.md` |
| 121 | Crash recovery | If daemon dies uncleanly → systemd restarts after 3s → daemon reconnects → cursor is back to normal | `main.md`, `daemon.md` |

### 5.3 Data Flow

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 122 | Hardware to daemon | Mouse → kernel (evdev) → X server → MotionNotify → shake-cursor | `cpu_on_event.md`, `main.md` |
| 123 | Daemon to screen | shake-cursor → x11rb ChangeWindowAttributes(cursor=big) → X server → cursor changes on screen | `main.md`, `dependencies.md` |
| 124 | Signal to daemon | systemd → SIGTERM → signal-hook → AtomicBool → event loop breaks → cleanup → exit | `dependencies.md`, `main.md` |

### 5.4 Error Handling

| # | Topic | Investigate | Source File |
|---|-------|-------------|-------------|
| 125 | Cannot connect to X server | Log error, exit. systemd restarts in 3s (X server might not be ready yet) | `main.md` |
| 126 | X server disconnects | Xorg crashed or restarted. Exit cleanly. systemd restarts | `main.md` |
| 127 | Cannot load cursor theme | Log warning, use fallback or skip enlargement | `main.md` |
| 128 | SIGTERM / SIGINT received | Restore original cursor, disconnect, exit cleanly | `main.md` |

---

## Empty Files (Need Content)

These files exist but are empty. They represent gaps to fill:

| File | Expected Content |
|------|-----------------|
| `Polling.md` | Detailed comparison of polling vs event-driven. CPU cost analysis. When polling is acceptable |
| `common_OS_on_display_servers.md` | Which display servers each OS uses: Linux (Xorg/Wayland), macOS (Quartz), Windows (DWM) |

---

## Investigation Order

Suggested sequence for working through these 128 topics:

### Phase 1: Foundation (Topics 1-50)

Understand the system your code runs on. No Rust code needed.

```
1.1 Window system stack          → Topics 1-6
1.2 Display server protocols     → Topics 7-11
1.3 X11 deep dive               → Topics 12-22
1.5 Daemons and systemd          → Topics 32-42
1.7 Events and CPU               → Topics 43-50
1.4 Wayland (skim for contrast)  → Topics 23-31
```

### Phase 2: Patterns (Topics 51-73)

Understand the design patterns before implementing them.

```
2.5 Builder pattern    → Topics 70-73   (simplest, implement first)
2.4 Ring buffer        → Topics 64-69   (core of the algorithm)
2.1 State machine      → Topics 51-55   (drives the main loop)
2.2 Observer           → Topics 56-58   (how events arrive)
2.3 Strategy           → Topics 59-63   (how backends plug in)
```

### Phase 3: Libraries (Topics 74-95)

Understand the tools before using them.

```
3.1 x11rb             → Topics 74-81   (your primary library)
3.2 xcursor           → Topics 82-87   (cursor enlargement)
3.3 signal-hook       → Topics 88-91   (clean shutdown)
3.4 log + env_logger  → Topics 92-95   (daemon logging)
```

### Phase 4: Algorithm (Topics 96-111)

Understand the math before coding it.

```
4.1 Input data         → Topics 96-98
4.2 Direction reversals → Topics 99-103
4.3 Velocity           → Topics 104-107
4.4 Complete detection → Topics 108-111
```

### Phase 5: Wiring (Topics 112-128)

Understand how pieces connect before writing main.rs.

```
5.1 Module structure   → Topics 112-116
5.2 Application flow   → Topics 117-121
5.3 Data flow          → Topics 122-124
5.4 Error handling     → Topics 125-128
```

---

## How to Use This Map

1. Work through one section at a time
2. For each topic, read the source file listed
3. Close the file and explain the topic in your own words
4. If you can't explain it clearly, re-read
5. Mark topics as understood by adding a checkmark: `[x]`
6. After completing a phase, implement the corresponding module(s)
