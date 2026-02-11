# Display Server

## Definition

A **display server** (also called a **window server** or **compositor**) is a system program that acts as the central coordinator between graphical applications and the underlying display hardware. It is responsible for:

1. **Window Management** — Creating, positioning, resizing, and destroying application windows
2. **Input Routing** — Receiving raw input from hardware (keyboard, mouse, touchscreen) and delivering it to the appropriate application
3. **Output Composition** — Combining the visual output of all applications into a single image displayed on the screen
4. **Resource Management** — Managing shared graphical resources like fonts, colors, and cursors

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Operating System                              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      Display Server                          │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │    │
│  │  │   Window    │  │   Input     │  │     Compositor      │  │    │
│  │  │  Manager    │  │  Handler    │  │  (Screen Renderer)  │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │    │
│  └──────────▲────────────────▲────────────────▲────────────────┘    │
│             │                │                │                      │
│  ┌──────────┴────────────────┴────────────────┴────────────────┐    │
│  │              Protocol Layer (X11 or Wayland)                 │    │
│  └──────────▲────────────────▲────────────────▲────────────────┘    │
│             │                │                │                      │
│      ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐              │
│      │ Application │  │ Application │  │ Application │              │
│      │  (Firefox)  │  │  (Terminal) │  │   (Game)    │              │
│      └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Hardware Layer                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │     GPU     │  │   Monitor   │  │  Keyboard   │  │   Mouse   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Responsibilities

### 1. Window Management

The display server maintains a hierarchy of windows and surfaces:

- **Root Window** — The background/desktop, covering the entire screen
- **Top-level Windows** — Application main windows
- **Child Windows** — Windows nested within other windows (dialogs, tooltips)
- **Surfaces** — Raw drawing areas that applications render to

For each window, the server tracks:
- Position (x, y coordinates)
- Dimensions (width, height)
- Z-order (stacking order — which window is on top)
- Visibility state (visible, minimized, hidden)
- Focus state (which window receives keyboard input)

### 2. Input Handling

The display server is the single point of entry for all user input:

```
Hardware Input → Display Server → Target Application
```

**Mouse Input:**
- Tracks absolute cursor position on screen
- Determines which window is under the cursor (hit testing)
- Delivers motion, button press, scroll events to the target window
- Manages cursor appearance (arrow, hand, text beam, etc.)

**Keyboard Input:**
- Tracks which window has keyboard focus
- Translates raw keycodes to symbols (handling keyboard layouts)
- Delivers key press/release events to the focused window
- Handles global shortcuts (Alt+Tab, PrintScreen, etc.)

### 3. Composition / Rendering

Modern display servers act as **compositors** — they combine the output of all applications:

1. Each application renders to its own **buffer** (a block of memory representing pixels)
2. The display server reads all visible buffers
3. It combines them according to window positions and z-order
4. The final composed image is sent to the GPU for display

This enables effects like:
- Transparency and shadows
- Smooth window animations
- Damage tracking (only re-rendering changed areas)
- VSync synchronization (preventing screen tearing)

## The Client-Server Model

Display servers follow a **client-server architecture**:

| Component | Role |
|-----------|------|
| **Server** | The display server itself — manages display and input |
| **Clients** | Applications that connect to display and draw windows |
| **Protocol** | The message format for client-server communication |

### Communication Flow

**Application wants to create a window:**
```
Client (App) → "Create window 800x600 at position 100,100" → Server
Server → Creates window, assigns ID → "Window ID: 42" → Client
```

**User moves mouse:**
```
Hardware → Mouse moved to (500, 300) → Server
Server → Determines Firefox is at that position
Server → "Mouse at (150, 80) relative to your window" → Firefox
```

**Application wants to draw:**
```
Client → "Draw rectangle at 10,10 size 100x50 in window 42" → Server
Server → Renders rectangle to window's buffer
Server → Composites all windows → Display
```

## Display Server vs Window Manager vs Compositor

These terms are often confused:

| Component | Responsibility | Examples |
|-----------|---------------|----------|
| **Display Server** | Core protocol handling, input routing | Xorg, Wayland compositor |
| **Window Manager** | Window decorations, positioning policy, workspaces | i3, Openbox, Mutter |
| **Compositor** | Visual composition, effects, transparency | Picom, Mutter, KWin |

**On X11:** These are often separate processes
- Xorg (display server) + i3 (window manager) + Picom (compositor)

**On Wayland:** These are unified into a single program
- Mutter = display server + window manager + compositor

## Why This Matters for shake-cursor

To implement cursor shake detection, we need to:

1. **Query cursor position** — Ask the display server where the mouse currently is
2. **Track movement over time** — Poll position repeatedly or subscribe to motion events
3. **React to patterns** — Detect "shake" motion and trigger actions

This requires communicating with the display server via its protocol:

| Protocol | How to get cursor position |
|----------|---------------------------|
| **X11** | `XQueryPointer()` — Direct query to X server |
| **Wayland** | No global cursor query — must use compositor-specific extensions or `libportal` |

The fundamental difference:
- **X11** — Any client can query global cursor position (less secure, but convenient)
- **Wayland** — Cursor position is only known relative to your own window (more secure, requires workarounds for global tracking)

## Common Display Servers

| Name | Protocol | Notes |
|------|----------|-------|
| **Xorg (X.Org Server)** | X11 | Traditional, widely compatible |
| **XWayland** | X11 | X11 compatibility layer running on Wayland |
| **Mutter** | Wayland | GNOME's compositor |
| **KWin** | Wayland | KDE Plasma's compositor |
| **Sway** | Wayland | i3-compatible tiling compositor |
| **Weston** | Wayland | Reference implementation |

## Summary

The display server is the **gatekeeper** between applications and the screen/input devices. Understanding its role is essential for building any application that needs low-level access to:

- Cursor position and movement
- Keyboard state
- Screen capture
- Window manipulation

For shake-cursor, we must work within the constraints of whichever display server protocol is active — with X11 being more straightforward and Wayland requiring compositor cooperation.
