# Wayland

## Definition

**Wayland** is a modern display server protocol created in 2008 as a replacement for X11. Unlike X11 where the display server, window manager, and compositor are separate programs, Wayland **unifies them into a single program** called a **compositor**.

```
X11:     Protocol → Xorg (server) + i3 (WM) + Picom (compositor) = 3 separate programs
Wayland: Protocol → Mutter / KWin / Sway = 1 unified program (called "compositor")
```

## Why Wayland Was Created

X11 was designed in 1984. Over 40 years, its limitations became clear:

```
┌─────────────────────────────────────────────────────────────┐
│                    X11 Problems                             │
│                                                             │
│   1. SECURITY                                               │
│      Any app can spy on any other app                       │
│      Any app can read your keystrokes                       │
│      Any app can take screenshots of other windows          │
│                                                             │
│   2. COMPLEXITY                                             │
│      The protocol has grown massive over 40 years           │
│      Hundreds of extensions layered on top                  │
│      Server-side rendering is outdated                      │
│                                                             │
│   3. PERFORMANCE                                            │
│      Extra round-trips between client → server → compositor │
│      Every draw command goes through the X server           │
│      Unnecessary overhead for modern GPUs                   │
│                                                             │
│   4. ARCHITECTURE                                           │
│      Display server, WM, and compositor are separate        │
│      They duplicate work and fight over control             │
│      Complex coordination between 3 processes               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Wayland Solutions                          │
│                                                             │
│   1. SECURITY → Apps are fully isolated from each other     │
│   2. SIMPLICITY → Minimal core protocol, clean design       │
│   3. PERFORMANCE → Client renders directly, no middleman    │
│   4. UNIFIED → One program does everything                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Wayland Architecture

### The Compositor Model

In Wayland, there is no separate display server. The **compositor** is everything:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WAYLAND ARCHITECTURE                           │
│                                                                     │
│   CLIENTS (applications)            COMPOSITOR (everything-in-one)  │
│                                                                     │
│   ┌─────────────┐                  ┌───────────────────────────┐    │
│   │  Firefox    │── Wayland ──────►│                           │    │
│   └─────────────┘   protocol       │   Mutter / KWin / Sway   │    │
│                                    │                           │    │
│   ┌─────────────┐                  │   Acts as:                │    │
│   │  Terminal   │── Wayland ──────►│   ✅ Display server       │    │
│   └─────────────┘   protocol       │   ✅ Window manager       │    │
│                                    │   ✅ Compositor           │    │
│   ┌─────────────┐                  │                           │    │
│   │  File Mgr   │── Wayland ──────►│   All in ONE process      │    │
│   └─────────────┘   protocol       └──────────┬────────────────┘    │
│                                               │                    │
│                                               ▼                    │
│                                    ┌───────────────────────┐        │
│                                    │      Hardware         │        │
│                                    │  GPU, Monitor, Mouse  │        │
│                                    │  Keyboard             │        │
│                                    └───────────────────────┘        │
└─────────────────────────────────────────────────────────────────────┘
```

### Compared to X11 Architecture

```
  X11 (3 separate programs)              Wayland (1 unified program)

  ┌─────────┐                            ┌─────────┐
  │  App    │                            │  App    │
  └────┬────┘                            └────┬────┘
       │                                      │
       ▼                                      │
  ┌─────────┐                                 │
  │  Xorg   │ (display server)                │
  └────┬────┘                                 │
       │                                      ▼
       ├────────┐                        ┌─────────────┐
       ▼        ▼                        │  Compositor │
  ┌────────┐ ┌────────┐                  │  (Mutter)   │
  │  i3    │ │ Picom  │                  │             │
  │  (WM)  │ │(comp.) │                  │ = server    │
  └────────┘ └────────┘                  │ + WM        │
       │        │                        │ + compositor│
       ▼        ▼                        └──────┬──────┘
  ┌─────────────────┐                          │
  │    Hardware     │                          ▼
  └─────────────────┘                    ┌───────────┐
                                         │ Hardware  │
                                         └───────────┘
```

## How Wayland Works

### Client-Side Rendering

The biggest architectural change from X11: **applications render their own pixels**.

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   X11 Rendering:                    Wayland Rendering:              │
│                                                                     │
│   App: "Draw a red rectangle       App: "Here's a buffer of        │
│         at 10,10 size 100x50"            pixels I already drew.    │
│         ↓                                Please display it."        │
│   X Server: draws the rectangle          ↓                          │
│         ↓                           Compositor: takes the buffer    │
│   Compositor: composites it                composites it            │
│         ↓                                  ↓                        │
│   Screen                            Screen                          │
│                                                                     │
│   3 steps, multiple processes       2 steps, single process         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

In detail:

```
┌────────┐                                         ┌────────────┐
│ Client │                                         │ Compositor │
└───┬────┘                                         └─────┬──────┘
    │                                                    │
    │  1. App renders pixels into a shared memory buffer │
    │     (using OpenGL, Vulkan, or CPU rendering)       │
    │                                                    │
    │  ┌──────────────────┐                              │
    │  │  Shared Memory   │                              │
    │  │  Buffer (pixels) │                              │
    │  │  ┌──┬──┬──┬──┐   │                              │
    │  │  │██│██│░░│░░│   │                              │
    │  │  │██│██│░░│░░│   │                              │
    │  │  └──┴──┴──┴──┘   │                              │
    │  └──────────────────┘                              │
    │                                                    │
    │ ── "attach this buffer to my surface" ───────────► │
    │                                                    │
    │ ── "I changed region (0,0)-(200,100)" ───────────► │
    │     (damage region)                                │
    │                                                    │
    │ ── "commit" (apply all pending changes) ─────────► │
    │                                                    │
    │                       Compositor combines all      │
    │                       client buffers and sends     │
    │                       final image to monitor       │
    │                                                    │
```

### The Object Protocol

Wayland uses an **object-oriented** protocol. Everything is an object with methods and events:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Wayland Object Model                            │
│                                                                     │
│   When a client connects, it gets a wl_display object:              │
│                                                                     │
│   wl_display (connection to compositor)                             │
│       │                                                             │
│       ├── wl_registry (discover available interfaces)               │
│       │       │                                                     │
│       │       ├── wl_compositor (create surfaces)                   │
│       │       │       │                                             │
│       │       │       └── wl_surface (a drawable area)              │
│       │       │               │                                     │
│       │       │               └── wl_buffer (pixel data)            │
│       │       │                                                     │
│       │       ├── wl_seat (group of input devices)                  │
│       │       │       │                                             │
│       │       │       ├── wl_pointer (mouse)                        │
│       │       │       │                                             │
│       │       │       └── wl_keyboard (keyboard)                    │
│       │       │                                                     │
│       │       ├── wl_shm (shared memory for buffers)                │
│       │       │                                                     │
│       │       └── xdg_wm_base (desktop window features)            │
│       │               │                                             │
│       │               └── xdg_surface → xdg_toplevel               │
│       │                   (desktop window with title bar, etc.)     │
│       │                                                             │
│       └── Each object has methods (requests) and events             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

Each object has **requests** (client → compositor) and **events** (compositor → client):

```
┌─────────────────────────────────────────────────────────────┐
│                  wl_pointer (mouse object)                   │
│                                                             │
│   Events (Compositor → Client):                             │
│                                                             │
│   • enter(surface, x, y)                                    │
│     "cursor entered YOUR window at position (x, y)"        │
│                                                             │
│   • leave(surface)                                          │
│     "cursor left YOUR window"                               │
│                                                             │
│   • motion(time, x, y)                                      │
│     "cursor moved to (x, y) within YOUR window"             │
│                                                             │
│   • button(time, button, state)                             │
│     "mouse button pressed/released in YOUR window"          │
│                                                             │
│   NOTICE: All coordinates are RELATIVE to your window.      │
│   You NEVER know the global cursor position.                │
│   You NEVER know if the cursor is over another app.         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Wayland Security Model

This is the fundamental difference from X11:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   X11 (Open Trust):                 Wayland (Isolated):             │
│                                                                     │
│   ┌─────────┐  ┌─────────┐         ┌─────────┐  ┌─────────┐       │
│   │  App A  │  │  App B  │         │  App A  │  │  App B  │       │
│   └────┬────┘  └────┬────┘         └────┬────┘  └────┬────┘       │
│        │            │                   │            │             │
│        │   Can spy  │                   │   CANNOT   │             │
│        │◄──────────►│                   │    see     │             │
│        │  on each   │                   │   each     │             │
│        │  other     │                   │   other    │             │
│        │            │                   │            │             │
│        ▼            ▼                   ▼            ▼             │
│   ┌────────────────────┐          ┌────────────────────────┐       │
│   │      X Server      │          │      Compositor        │       │
│   │  (shared state)    │          │  (isolated channels)   │       │
│   └────────────────────┘          └────────────────────────┘       │
│                                                                     │
│   App A can:                        App A can:                      │
│   ✅ Read App B's keystrokes        ❌ NOT read App B's keystrokes  │
│   ✅ Screenshot App B               ❌ NOT screenshot App B         │
│   ✅ Know global cursor position    ❌ NOT know global cursor pos   │
│   ✅ Inject input into App B        ❌ NOT inject input into App B  │
│   ✅ Move/resize App B              ❌ NOT move/resize App B        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

Each application has its own **isolated channel** to the compositor:

```
┌─────────────────────────────────────────────────────────────┐
│                   Isolated Channels                         │
│                                                             │
│   ┌──────────┐     private channel     ┌──────────────┐    │
│   │ Firefox  │◄───────────────────────►│              │    │
│   └──────────┘  only Firefox events    │              │    │
│                                        │              │    │
│   ┌──────────┐     private channel     │  Compositor  │    │
│   │ Terminal │◄───────────────────────►│              │    │
│   └──────────┘  only Terminal events   │              │    │
│                                        │              │    │
│   ┌──────────┐     private channel     │              │    │
│   │  Game    │◄───────────────────────►│              │    │
│   └──────────┘  only Game events       └──────────────┘    │
│                                                             │
│   Firefox cannot see Terminal's events.                      │
│   Terminal cannot see Game's events.                        │
│   No app knows what any other app is doing.                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Wayland Communication In Practice

### Connecting to the Compositor

```
┌────────┐                                         ┌────────────┐
│ Client │                                         │ Compositor │
└───┬────┘                                         └─────┬──────┘
    │                                                    │
    │ ── Connect via $XDG_RUNTIME_DIR/wayland-0 ────────►│
    │    (typically /run/user/1000/wayland-0)             │
    │                                                    │
    │ ◄── wl_display object ────────────────────────────  │
    │                                                    │
    │ ── get_registry() ────────────────────────────────►│
    │                                                    │
    │ ◄── global: wl_compositor v5 ─────────────────────  │
    │ ◄── global: wl_seat v7 ──────────────────────────── │
    │ ◄── global: wl_shm v1 ──────────────────────────── │
    │ ◄── global: xdg_wm_base v3 ─────────────────────── │
    │     (list of available interfaces)                  │
    │                                                    │
```

### Creating and Showing a Window

```
┌────────┐                                         ┌────────────┐
│ Client │                                         │ Compositor │
└───┬────┘                                         └─────┬──────┘
    │                                                    │
    │ ── wl_compositor.create_surface() ────────────────►│
    │                                                    │
    │ ◄── wl_surface created ───────────────────────────  │
    │                                                    │
    │ ── xdg_wm_base.get_xdg_surface(surface) ────────►│
    │                                                    │
    │ ── xdg_surface.get_toplevel() ───────────────────►│
    │                                                    │
    │ ── xdg_toplevel.set_title("My App") ─────────────►│
    │                                                    │
    │ ── wl_surface.commit() ──────────────────────────►│
    │    "I'm ready, show me"                            │
    │                                                    │
    │ ◄── xdg_surface.configure(width, height) ────────  │
    │    "compositor says: use this size"                │
    │                                                    │
    │    (client renders pixels into buffer)             │
    │                                                    │
    │ ── wl_surface.attach(buffer) ────────────────────►│
    │ ── wl_surface.damage(0, 0, width, height) ──────►│
    │ ── wl_surface.commit() ──────────────────────────►│
    │    "here are my pixels, display them"              │
    │                                                    │
```

### Receiving Input Events

```
┌────────┐                                         ┌────────────┐
│ Client │                                         │ Compositor │
└───┬────┘                                         └─────┬──────┘
    │                                                    │
    │    (user moves cursor over client's window)        │
    │                                                    │
    │ ◄── wl_pointer.enter(surface, x=150, y=230) ─────  │
    │     "cursor entered your window"                   │
    │                                                    │
    │ ◄── wl_pointer.motion(time, x=152, y=228) ──────── │
    │     "cursor moved WITHIN your window"              │
    │                                                    │
    │ ◄── wl_pointer.motion(time, x=155, y=225) ──────── │
    │                                                    │
    │    (user moves cursor OUT of client's window)      │
    │                                                    │
    │ ◄── wl_pointer.leave(surface) ────────────────────  │
    │     "cursor left your window"                      │
    │                                                    │
    │    ❌ NO MORE EVENTS until cursor comes back       │
    │    ❌ You have NO IDEA where the cursor went       │
    │    ❌ You cannot query global cursor position      │
    │                                                    │
```

### Trying to Query Global Cursor Position

```
┌────────┐                                         ┌────────────┐
│ Client │                                         │ Compositor │
└───┬────┘                                         └─────┬──────┘
    │                                                    │
    │ ── "Where is the cursor globally?" ──────────────►│
    │                                                    │
    │ ◄── ❌ NO SUCH REQUEST EXISTS IN WAYLAND ──────── │
    │                                                    │
    │     The protocol simply does not define            │
    │     a way to ask for global cursor position.       │
    │     This is BY DESIGN for security.                │
    │                                                    │
```

## The shake-cursor Problem on Wayland

Since Wayland apps can only see the cursor **when it's over their own window**, shake detection is fundamentally harder:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   User shakes cursor across the screen:                             │
│                                                                     │
│   ┌──────────────────────────────────────────────────────────┐      │
│   │  Desktop                                                  │      │
│   │          ←→←→←→←→←→  (shake motion)                      │      │
│   │                                                          │      │
│   │   ┌──────────┐                    ┌──────────┐           │      │
│   │   │ Firefox  │                    │ Terminal │           │      │
│   │   └──────────┘                    └──────────┘           │      │
│   │                                                          │      │
│   └──────────────────────────────────────────────────────────┘      │
│                                                                     │
│   What X11 sees:     x=100 → x=500 → x=100 → x=500 → x=100       │
│                      (complete picture, easy to detect shake)       │
│                                                                     │
│   What Wayland                                                      │
│   app sees:          enter → motion → leave → ??? → enter → ...    │
│                      (gaps whenever cursor leaves your window)      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Workarounds for Wayland

| Approach | How it works | Pros | Cons |
|----------|-------------|------|------|
| **XDG Desktop Portal** | Standard API for privileged operations | Cross-compositor | Requires user permission, may not support cursor tracking |
| **Compositor extensions** | Custom protocols (e.g., `wlr-foreign-toplevel`) | Native, efficient | Different per compositor (Mutter vs KWin vs Sway) |
| **libei** | Emulated Input protocol | Designed for input capture | Newer, less widespread |
| **XWayland** | Run as X11 app on Wayland | Full X11 access | Not a native solution |
| **Invisible overlay** | Transparent fullscreen surface | Gets all motion events | Hacky, blocks input to other apps |

### Most Practical: Compositor-Specific Extensions

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Each compositor may offer custom protocols:                       │
│                                                                     │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  wlroots-based compositors (Sway, Hyprland, etc.)         │    │
│   │                                                            │    │
│   │  • zwlr_virtual_pointer_v1 — create virtual pointer       │    │
│   │  • wlr_layer_shell — overlay surfaces                     │    │
│   │  • zwp_pointer_constraints — lock/confine pointer         │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  GNOME (Mutter)                                            │    │
│   │                                                            │    │
│   │  • Remote desktop portal for input capture                 │    │
│   │  • GNOME Shell extensions (JavaScript)                     │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  KDE (KWin)                                                │    │
│   │                                                            │    │
│   │  • KWin scripting API                                      │    │
│   │  • Custom Wayland protocols                                │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│   ⚠ No single solution works on ALL Wayland compositors           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Wayland Compositors

| Compositor | Desktop Environment | Based On |
|-----------|--------------------| ---------|
| **Mutter** | GNOME | Custom |
| **KWin** | KDE Plasma | Custom |
| **Sway** | Standalone (i3-like) | wlroots |
| **Hyprland** | Standalone (animated) | wlroots |
| **Weston** | Reference/testing | Custom |
| **Wayfire** | Standalone (3D effects) | wlroots |

Note: **wlroots** is a library that provides common compositor building blocks. Compositors built on wlroots share many of the same extension protocols.

## Common Wayland Interfaces

### Core Protocol

| Interface | Purpose |
|-----------|---------|
| `wl_display` | Root object, connection to compositor |
| `wl_registry` | Discover global objects |
| `wl_compositor` | Create surfaces |
| `wl_surface` | Drawable area (like an X11 window) |
| `wl_buffer` | Pixel data container |
| `wl_shm` | Shared memory for CPU-rendered buffers |
| `wl_seat` | Input device group |
| `wl_pointer` | Mouse/trackpad |
| `wl_keyboard` | Keyboard |
| `wl_output` | A monitor |

### XDG Shell (Desktop Integration)

| Interface | Purpose |
|-----------|---------|
| `xdg_wm_base` | Desktop window management entry point |
| `xdg_surface` | Desktop-aware surface |
| `xdg_toplevel` | Top-level window (title, min/max/close) |
| `xdg_popup` | Popup menus and tooltips |

## X11 vs Wayland Summary

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Feature              X11                   Wayland               │
│   ─────────────────────────────────────────────────────────────    │
│   Architecture         Separate processes    Unified compositor    │
│   Rendering            Server-side           Client-side           │
│   Security             Open (all trusted)    Isolated (per-app)    │
│   Global cursor pos    ✅ XQueryPointer      ❌ Not available      │
│   Screen capture       ✅ Any client         ❌ Needs permission   │
│   Input injection      ✅ XTest extension    ❌ Needs permission   │
│   Network transparent  ✅ Built-in           ❌ Not supported      │
│   Protocol age         1984                  2008+                 │
│   Complexity           High (legacy)         Low (minimal core)    │
│   Tearing prevention   Optional compositor   Built-in              │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Relevance to shake-cursor

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   shake-cursor needs:                                               │
│   • Continuous global cursor position                               │
│   • Works regardless of which window is focused                     │
│   • Low latency tracking                                            │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────┐       │
│   │  X11: STRAIGHTFORWARD                                    │       │
│   │                                                         │       │
│   │  loop {                                                 │       │
│   │      (x, y) = XQueryPointer(root_window)                │       │
│   │      analyze_for_shake(x, y)                            │       │
│   │  }                                                      │       │
│   │                                                         │       │
│   │  OR subscribe to MotionNotify on root window            │       │
│   └─────────────────────────────────────────────────────────┘       │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────┐       │
│   │  Wayland: REQUIRES WORKAROUNDS                           │       │
│   │                                                         │       │
│   │  Option A: Use XWayland (run as X11 app)                │       │
│   │  Option B: Use compositor-specific extensions           │       │
│   │  Option C: Use XDG Desktop Portal (if supported)        │       │
│   │  Option D: Use libei for input capture                  │       │
│   │                                                         │       │
│   │  No single approach works everywhere.                   │       │
│   └─────────────────────────────────────────────────────────┘       │
│                                                                     │
│   RECOMMENDATION: Start with X11 implementation first.              │
│   Your Fedora XFCE uses X11, so it will work directly.              │
│   Add Wayland support later as a separate backend.                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Summary

| Concept | Definition |
|---------|------------|
| **Wayland** | Modern display server protocol replacing X11 |
| **Compositor** | The unified program (server + WM + compositor) |
| **wl_surface** | Equivalent of an X11 window |
| **wl_pointer** | Mouse interface, only reports within your window |
| **Client-side rendering** | Apps draw their own pixels, compositor combines them |
| **Isolation** | Apps cannot see each other's state or input |

Wayland is more secure and efficient than X11, but its security model makes tools like shake-cursor harder to implement. Global cursor tracking — trivial on X11 — requires compositor cooperation on Wayland.
