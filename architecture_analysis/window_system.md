# Window System

## Definition

A **window system** (also called a **windowing system**) is the complete software stack that enables graphical user interfaces on an operating system. It's the entire ecosystem that allows multiple applications to display windows on screen and receive user input.

The window system is **not a single program** — it's the combination of several components working together:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         WINDOW SYSTEM                               │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                        Applications                            │  │
│  │         (Firefox, Terminal, File Manager, Games)               │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│                               │ Protocol (X11 / Wayland)            │
│                               ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                      Display Server                            │  │
│  │              (Xorg, Mutter, KWin, Weston)                      │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                      Window Manager                            │  │
│  │        (i3, Openbox, Mutter, KWin, Sway, dwm)                  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                        Compositor                              │  │
│  │             (Picom, Mutter, KWin, Compton)                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
└───────────────────────────────┼─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           Hardware                                  │
│              (GPU, Monitor, Keyboard, Mouse)                        │
└─────────────────────────────────────────────────────────────────────┘
```

## Components of a Window System

### 1. Display Server

**Role:** Central coordinator between applications and hardware.

**Responsibilities:**
- Receives input events from hardware (mouse, keyboard)
- Routes input to the correct application
- Receives drawing commands from applications
- Manages shared resources (cursors, fonts)

**Examples:** Xorg, Weston, XWayland

```
┌─────────────────────────────────────────────────────────────┐
│                      Display Server                         │
│                                                             │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│   │   Input     │    │   Window    │    │  Resource   │    │
│   │  Handling   │    │  Registry   │    │  Manager    │    │
│   └─────────────┘    └─────────────┘    └─────────────┘    │
│                                                             │
│   • Receives mouse/keyboard events                          │
│   • Tracks all windows                                      │
│   • Manages cursors, fonts, colors                          │
└─────────────────────────────────────────────────────────────┘
```

### 2. Window Manager

**Role:** Controls the placement, appearance, and behavior of windows.

**Responsibilities:**
- Decide where windows appear on screen
- Draw window decorations (title bar, borders, buttons)
- Handle window operations (minimize, maximize, close)
- Manage workspaces/virtual desktops
- Handle keyboard shortcuts (Alt+Tab, etc.)

**Examples:** i3, Openbox, dwm, bspwm, Mutter, KWin

```
┌─────────────────────────────────────────────────────────────┐
│                      Window Manager                         │
│                                                             │
│   User opens Firefox:                                       │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Window Manager decides:                             │   │
│   │  • Position: center of screen                        │   │
│   │  • Size: 1200x800                                    │   │
│   │  • Decorations: title bar with close/min/max buttons │   │
│   │  • Workspace: current workspace                      │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   Types of Window Managers:                                 │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│   │   Stacking   │  │   Tiling     │  │   Dynamic    │     │
│   │  (Openbox)   │  │    (i3)      │  │   (awesome)  │     │
│   │              │  │              │  │              │     │
│   │  ┌───┐ ┌───┐ │  │ ┌────┬────┐ │  │  Both modes  │     │
│   │  │   │ │   │ │  │ │    │    │ │  │   available  │     │
│   │  └───┘ └───┘ │  │ ├────┼────┤ │  │              │     │
│   │   overlapping│  │ │    │    │ │  │              │     │
│   │   windows    │  │ └────┴────┘ │  │              │     │
│   │              │  │  no overlap │  │              │     │
│   └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### 3. Compositor

**Role:** Combines the visual output of all windows into the final screen image.

**Responsibilities:**
- Collect rendered content from all visible windows
- Combine (composite) them according to position and z-order
- Apply visual effects (transparency, shadows, blur)
- Handle VSync to prevent screen tearing
- Manage off-screen buffers

**Examples:** Picom, Compton, Mutter, KWin

```
┌─────────────────────────────────────────────────────────────┐
│                        Compositor                           │
│                                                             │
│   Window Buffers:              Final Composited Image:      │
│                                                             │
│   ┌─────────┐                  ┌───────────────────────┐    │
│   │ Firefox │                  │ ┌─────────────────┐   │    │
│   │ Buffer  │──┐               │ │    Firefox      │   │    │
│   └─────────┘  │               │ │   (on top)      │   │    │
│                │    Combine    │ └───┬─────────────┘   │    │
│   ┌─────────┐  ├──────────────►│ ┌───┴─────────────┐   │    │
│   │Terminal │  │               │ │    Terminal     │   │    │
│   │ Buffer  │──┤               │ │   (behind)      │   │    │
│   └─────────┘  │               │ └───┬─────────────┘   │    │
│                │               │ ┌───┴─────────────┐   │    │
│   ┌─────────┐  │               │ │    Desktop      │   │    │
│   │Desktop  │──┘               │ │  (background)   │   │    │
│   │ Buffer  │                  │ └─────────────────┘   │    │
│   └─────────┘                  └───────────────────────┘    │
│                                                             │
│   Effects applied:                                          │
│   • Transparency (see-through windows)                      │
│   • Shadows (depth perception)                              │
│   • Blur (frosted glass effect)                             │
│   • Animations (smooth transitions)                         │
└─────────────────────────────────────────────────────────────┘
```

### 4. Protocol

**Role:** Defines the communication language between applications and display server.

**Examples:** X11, Wayland

```
┌─────────────────────────────────────────────────────────────┐
│                         Protocol                            │
│                                                             │
│   Application                              Display Server   │
│       │                                          │          │
│       │ ────── "Create window 800x600" ────────► │          │
│       │                                          │          │
│       │ ◄───── "Window ID: 42" ───────────────── │          │
│       │                                          │          │
│       │ ────── "Show window 42" ───────────────► │          │
│       │                                          │          │
│       │ ◄───── "Mouse moved to 100,200" ──────── │          │
│       │                                          │          │
│       │ ◄───── "Key 'A' pressed" ──────────────  │          │
│       │                                          │          │
│                                                             │
│   The protocol defines EXACTLY what messages exist          │
│   and how they are formatted.                               │
└─────────────────────────────────────────────────────────────┘
```

## X11 Window System vs Wayland Window System

The key difference is how components are organized:

### X11 Architecture (Separated Components)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      X11 WINDOW SYSTEM                              │
│                                                                     │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │
│   │  Firefox    │     │  Terminal   │     │   Game      │          │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘          │
│          │                   │                   │                  │
│          └───────────────────┼───────────────────┘                  │
│                              │                                      │
│                              ▼                                      │
│                    ┌─────────────────┐                              │
│                    │   X11 Protocol  │                              │
│                    └────────┬────────┘                              │
│                              │                                      │
│                              ▼                                      │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                        Xorg                                  │  │
│   │                   (Display Server)                           │  │
│   │                                                              │  │
│   │    Handles: connections, input routing, basic rendering      │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│          ┌───────────────────┼───────────────────┐                  │
│          ▼                   │                   ▼                  │
│   ┌─────────────┐            │            ┌─────────────┐          │
│   │   Window    │            │            │ Compositor  │          │
│   │   Manager   │            │            │   (Picom)   │          │
│   │    (i3)     │            │            │             │          │
│   └─────────────┘            │            └─────────────┘          │
│                              │                                      │
│   Note: Window Manager and Compositor are SEPARATE processes       │
│   that also communicate with Xorg via X11 protocol                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Characteristics:**
- Display server (Xorg) is a separate process
- Window manager is a separate process (can be swapped: i3, Openbox, etc.)
- Compositor is optional and separate (Picom, Compton)
- More flexibility but more complexity
- Applications can query global state (cursor position, other windows)

### Wayland Architecture (Unified Components)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    WAYLAND WINDOW SYSTEM                            │
│                                                                     │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │
│   │  Firefox    │     │  Terminal   │     │   Game      │          │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘          │
│          │                   │                   │                  │
│          └───────────────────┼───────────────────┘                  │
│                              │                                      │
│                              ▼                                      │
│                   ┌──────────────────┐                              │
│                   │ Wayland Protocol │                              │
│                   └────────┬─────────┘                              │
│                              │                                      │
│                              ▼                                      │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                                                              │  │
│   │                     Mutter / KWin / Sway                     │  │
│   │                                                              │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │  │
│   │  │  Display    │  │   Window    │  │ Compositor  │          │  │
│   │  │  Server     │  │   Manager   │  │             │          │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘          │  │
│   │                                                              │  │
│   │          ALL THREE COMBINED INTO ONE PROCESS                 │  │
│   │                                                              │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
│   Note: Compositor IS the display server IS the window manager      │
│   (that's why they're all called "Wayland compositors")             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Characteristics:**
- Single unified process handles everything
- Simpler architecture, fewer round-trips
- Better security (apps are isolated)
- Less flexibility (can't swap just the window manager)
- Applications cannot query global state

## Component Relationship Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                        WINDOW SYSTEM                                │
│                     (the entire stack)                              │
│                                                                     │
│         ┌───────────────────────────────────────────┐               │
│         │              PROTOCOL                     │               │
│         │    (defines communication language)       │               │
│         │         X11  or  Wayland                  │               │
│         └───────────────────────────────────────────┘               │
│                            │                                        │
│                            │ implemented by                         │
│                            ▼                                        │
│         ┌───────────────────────────────────────────┐               │
│         │          DISPLAY SERVER                   │               │
│         │    (routes input, manages windows)        │               │
│         │       Xorg, Mutter, KWin, Weston          │               │
│         └───────────────────────────────────────────┘               │
│                            │                                        │
│              ┌─────────────┴─────────────┐                          │
│              │                           │                          │
│              ▼                           ▼                          │
│   ┌─────────────────────┐     ┌─────────────────────┐               │
│   │   WINDOW MANAGER    │     │    COMPOSITOR       │               │
│   │ (positions windows) │     │ (combines buffers)  │               │
│   │  i3, Openbox, dwm   │     │  Picom, Mutter      │               │
│   └─────────────────────┘     └─────────────────────┘               │
│                                                                     │
│   On X11: These can be separate programs                            │
│   On Wayland: These are part of the compositor                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Common Window Systems

| OS | Window System | Protocol | Display Server | Notes |
|----|---------------|----------|----------------|-------|
| Linux (traditional) | X Window System | X11 | Xorg | Most compatible |
| Linux (modern) | Wayland | Wayland | Mutter/KWin/Sway | More secure |
| macOS | Quartz | Quartz Compositor | WindowServer | Proprietary |
| Windows | Desktop Window Manager | Win32/WinRT | dwm.exe | Proprietary |

## Input/Output Flow Through the Window System

### Input Flow (User → Application)

```
┌────────────────────────────────────────────────────────────────────┐
│                         INPUT FLOW                                 │
│                                                                    │
│  1. User moves mouse                                               │
│     ┌─────────┐                                                    │
│     │  Mouse  │                                                    │
│     └────┬────┘                                                    │
│          │ raw hardware event                                      │
│          ▼                                                         │
│  2. Kernel receives event                                          │
│     ┌─────────────────┐                                            │
│     │  Linux Kernel   │                                            │
│     │  (evdev/libinput)│                                           │
│     └────────┬────────┘                                            │
│              │ /dev/input/event*                                   │
│              ▼                                                     │
│  3. Display server receives event                                  │
│     ┌─────────────────┐                                            │
│     │ Display Server  │                                            │
│     │ (Xorg/Mutter)   │                                            │
│     └────────┬────────┘                                            │
│              │ determines which window                             │
│              │ is under cursor                                     │
│              ▼                                                     │
│  4. Event delivered to application                                 │
│     ┌─────────────────┐                                            │
│     │   Application   │                                            │
│     │   (Firefox)     │                                            │
│     └─────────────────┘                                            │
│              │                                                     │
│              ▼                                                     │
│  5. Application handles event                                      │
│     "User clicked on link, navigate to URL"                        │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Output Flow (Application → Screen)

```
┌────────────────────────────────────────────────────────────────────┐
│                         OUTPUT FLOW                                │
│                                                                    │
│  1. Application wants to draw                                      │
│     ┌─────────────────┐                                            │
│     │   Application   │                                            │
│     │   (Firefox)     │                                            │
│     └────────┬────────┘                                            │
│              │ renders to buffer                                   │
│              ▼                                                     │
│  2. Buffer ready                                                   │
│     ┌─────────────────┐                                            │
│     │  Shared Memory  │                                            │
│     │     Buffer      │                                            │
│     └────────┬────────┘                                            │
│              │ "buffer ready" message                              │
│              ▼                                                     │
│  3. Compositor receives buffer                                     │
│     ┌─────────────────┐                                            │
│     │   Compositor    │                                            │
│     │ (Picom/Mutter)  │                                            │
│     └────────┬────────┘                                            │
│              │ combines all window buffers                         │
│              ▼                                                     │
│  4. Final image sent to GPU                                        │
│     ┌─────────────────┐                                            │
│     │      GPU        │                                            │
│     └────────┬────────┘                                            │
│              │ scanout                                             │
│              ▼                                                     │
│  5. Image displayed                                                │
│     ┌─────────────────┐                                            │
│     │    Monitor      │                                            │
│     └─────────────────┘                                            │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Relevance to shake-cursor

For the shake-cursor project, we interact with the **window system** at the protocol level:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  shake-cursor                    Window System                      │
│  ┌───────────────┐              ┌────────────────┐                  │
│  │               │   Protocol   │                │                  │
│  │ Query cursor  │◄────────────►│ Display Server │                  │
│  │ position      │   (X11 or    │                │                  │
│  │               │   Wayland)   │                │                  │
│  └───────────────┘              └────────────────┘                  │
│                                                                     │
│  What we need from the window system:                               │
│  • Current cursor position (x, y)                                   │
│  • Ability to query position frequently (polling)                   │
│  • OR ability to receive motion events (event-driven)               │
│                                                                     │
│  X11: XQueryPointer() gives us global cursor position               │
│  Wayland: Must use workarounds (portals, extensions)                │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Summary

| Term | Definition |
|------|------------|
| **Window System** | The complete stack enabling GUIs (display server + WM + compositor + protocol) |
| **Display Server** | Program that manages display hardware and routes input |
| **Window Manager** | Program that positions windows and handles decorations |
| **Compositor** | Program that combines window buffers into final screen image |
| **Protocol** | The language (X11/Wayland) for client-server communication |

The **window system** is the umbrella term for everything that makes graphical applications work on Linux. When we say "X11 window system" or "Wayland window system," we mean the entire stack built around that protocol.
