# Display Server Protocol

## Definition

A **display server protocol** is a formal specification that defines the exact rules, message formats, and procedures for communication between graphical applications (clients) and the display server.

Think of it as a **contract** or **language** that both parties agree to speak:

- The **client** (application) sends requests in a specific format
- The **server** (display server) understands those requests and responds accordingly
- Both sides know exactly what messages exist and what they mean

## Analogy: HTTP vs Web Server

| Concept | Web World | Graphics World |
|---------|-----------|----------------|
| **Protocol** | HTTP | X11 / Wayland |
| **Server** | Nginx, Apache | Xorg, Mutter, KWin |
| **Client** | Browser | Firefox, Terminal, Games |
| **Messages** | GET, POST, PUT | CreateWindow, MapWindow, MotionNotify |

Just as HTTP defines how browsers talk to web servers, X11/Wayland define how applications talk to display servers.

## Why Protocols Matter

Without a standardized protocol:

```
App A: "Make me a window please!"
Server: "I don't understand that format"

App B: "WINDOW_CREATE{800,600}"
Server: "That's not how you ask"
```

With a standardized protocol:

```
App A: XCreateWindow(display, parent, x, y, width, height, ...)
Server: "Understood. Here's window ID 42"

App B: XCreateWindow(display, parent, x, y, width, height, ...)
Server: "Understood. Here's window ID 43"
```

The protocol ensures **any application** can talk to **any compatible server**.

## Protocol Structure

### 1. Message Types

Display server protocols typically define several categories of messages:

| Type | Direction | Purpose | Example |
|------|-----------|---------|---------|
| **Request** | Client → Server | Ask the server to do something | "Create a window" |
| **Reply** | Server → Client | Response to a request | "Window ID is 42" |
| **Event** | Server → Client | Notify client something happened | "Mouse moved to (100, 200)" |
| **Error** | Server → Client | Report a problem | "Invalid window ID" |

### 2. Message Format

Each message has a defined binary structure:

```
┌─────────────────────────────────────────────────────────────┐
│                    Protocol Message                         │
├──────────┬──────────┬──────────┬───────────────────────────┤
│  Opcode  │  Length  │ Sequence │         Payload           │
│ (1 byte) │ (2 bytes)│ (2 bytes)│      (variable size)      │
├──────────┴──────────┴──────────┴───────────────────────────┤
│                                                             │
│  Opcode: Which operation (CreateWindow=1, MapWindow=8, etc) │
│  Length: Total message size                                 │
│  Sequence: Request number for matching replies              │
│  Payload: Operation-specific data                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 3. Connection Lifecycle

```
┌────────┐                                    ┌────────┐
│ Client │                                    │ Server │
└───┬────┘                                    └───┬────┘
    │                                             │
    │ ──── 1. Connection Request ───────────────►│
    │                                             │
    │ ◄─── 2. Connection Accepted ───────────────│
    │      (server info, supported features)     │
    │                                             │
    │ ──── 3. Create Window Request ────────────►│
    │                                             │
    │ ◄─── 4. Window Created (ID: 42) ───────────│
    │                                             │
    │ ──── 5. Map Window (show it) ─────────────►│
    │                                             │
    │ ◄─── 6. Expose Event (need to draw) ───────│
    │                                             │
    │ ──── 7. Drawing Commands ─────────────────►│
    │                                             │
    │ ◄─── 8. Input Events (mouse, keyboard) ────│
    │      (continuous stream while running)     │
    │                                             │
    │ ──── 9. Destroy Window ───────────────────►│
    │                                             │
    │ ──── 10. Disconnect ──────────────────────►│
    │                                             │
```

## X11 Protocol

### Overview

The **X11 protocol** (X Window System Protocol, version 11) was designed in 1987. It's a **network-transparent** protocol, meaning applications can run on one machine and display on another.

### Key Characteristics

| Aspect | X11 Approach |
|--------|--------------|
| **Architecture** | Client-server with network transparency |
| **Connection** | Unix socket (`/tmp/.X11-unix/X0`) or TCP |
| **State** | Server maintains extensive state about windows |
| **Trust model** | All clients are trusted (can spy on each other) |
| **Drawing** | Server-side rendering (traditionally) |

### Core Concepts

**Display**: The connection to an X server
```
DISPLAY=:0        → Local display 0
DISPLAY=:1        → Local display 1
DISPLAY=host:0    → Remote display on 'host'
```

**Screen**: A physical monitor managed by the display

**Window**: A rectangular area that can receive input and display output

**Atom**: Interned strings for efficient property naming

### Common X11 Requests

| Request | Purpose |
|---------|---------|
| `CreateWindow` | Create a new window |
| `MapWindow` | Make a window visible |
| `UnmapWindow` | Hide a window |
| `DestroyWindow` | Delete a window |
| `ConfigureWindow` | Move/resize a window |
| `QueryPointer` | Get current cursor position |
| `GrabPointer` | Capture all mouse input |
| `ChangeProperty` | Set window properties |

### Common X11 Events

| Event | When it occurs |
|-------|----------------|
| `Expose` | Window needs to be redrawn |
| `KeyPress` / `KeyRelease` | Keyboard input |
| `ButtonPress` / `ButtonRelease` | Mouse button input |
| `MotionNotify` | Mouse movement |
| `EnterNotify` / `LeaveNotify` | Cursor enters/leaves window |
| `FocusIn` / `FocusOut` | Window gains/loses keyboard focus |

### X11 Extensions

The base X11 protocol is extended by various extensions:

| Extension | Purpose |
|-----------|---------|
| **XInput2** | Advanced input handling (multi-touch, tablets) |
| **XFIXES** | Cursor visibility, region management |
| **Composite** | Off-screen rendering for compositing |
| **RANDR** | Monitor configuration (resolution, rotation) |
| **GLX** | OpenGL rendering |

## Wayland Protocol

### Overview

**Wayland** is a modern replacement for X11, designed in 2008+. It simplifies the architecture by combining the display server, compositor, and window manager into one entity.

### Key Characteristics

| Aspect | Wayland Approach |
|--------|------------------|
| **Architecture** | Client talks directly to compositor |
| **Connection** | Unix socket only (no network transparency) |
| **State** | Client maintains its own state |
| **Trust model** | Clients are isolated (can't spy on each other) |
| **Drawing** | Client-side rendering (always) |

### Core Concepts

**Compositor**: The unified display server + window manager + compositor

**Surface**: A rectangular area the client renders to (equivalent to X11 window)

**Buffer**: Shared memory containing pixel data

**Object**: Protocol entities identified by numeric IDs

### Wayland Object Model

Wayland is **object-oriented**. Everything is an object with:
- A unique ID
- An interface (type definition)
- Methods (requests from client)
- Events (notifications to client)

```
┌─────────────────────────────────────────────────────────────┐
│                    wl_surface (object)                      │
├─────────────────────────────────────────────────────────────┤
│ Methods (Client → Server):                                  │
│   • attach(buffer)    - Set the buffer to display           │
│   • damage(x,y,w,h)   - Mark region as changed              │
│   • commit()          - Apply pending changes               │
├─────────────────────────────────────────────────────────────┤
│ Events (Server → Client):                                   │
│   • enter(output)     - Surface now visible on output       │
│   • leave(output)     - Surface no longer on output         │
└─────────────────────────────────────────────────────────────┘
```

### Core Wayland Interfaces

| Interface | Purpose |
|-----------|---------|
| `wl_display` | Core connection to compositor |
| `wl_registry` | Discover available global objects |
| `wl_compositor` | Create surfaces |
| `wl_surface` | A visible area to render to |
| `wl_buffer` | Pixel data storage |
| `wl_seat` | Group of input devices |
| `wl_pointer` | Mouse/touchpad |
| `wl_keyboard` | Keyboard |

### Wayland Extensions

Wayland has a minimal core protocol. Extra functionality comes from extensions:

| Protocol | Purpose |
|----------|---------|
| **xdg-shell** | Desktop window management (minimize, maximize, etc.) |
| **wlr-layer-shell** | Desktop overlays (panels, notifications) |
| **xdg-decoration** | Server-side window decorations |
| **wp-viewporter** | Scaling and cropping surfaces |

## Protocol Comparison: X11 vs Wayland

| Feature | X11 | Wayland |
|---------|-----|---------|
| **Age** | 1987 | 2008+ |
| **Global cursor position** | ✅ `XQueryPointer` | ❌ Not allowed (security) |
| **Screen capture** | ✅ Any app can do it | ❌ Requires portal/permission |
| **Window snooping** | ✅ Possible | ❌ Prevented |
| **Network transparency** | ✅ Built-in | ❌ Not supported |
| **Complexity** | High (many legacy features) | Low (minimal core) |
| **Security** | Weak | Strong |

## How Applications Use Protocols

Applications don't typically write raw protocol messages. Instead, they use **client libraries**:

### X11 Libraries

| Library | Language | Level |
|---------|----------|-------|
| **Xlib** | C | Low-level, direct protocol access |
| **XCB** | C | Lower-level, asynchronous |
| **x11-rb** | Ruby | Bindings to XCB |
| **x11** | Rust | Rust bindings |

### Wayland Libraries

| Library | Language | Level |
|---------|----------|-------|
| **libwayland-client** | C | Core Wayland client |
| **wayland-client** | Rust | Rust bindings |
| **smithay-client-toolkit** | Rust | Higher-level toolkit |

### Example: Query Cursor Position

**X11 (using Xlib in C):**
```c
Display *display = XOpenDisplay(NULL);
Window root = DefaultRootWindow(display);
Window child;
int root_x, root_y, win_x, win_y;
unsigned int mask;

XQueryPointer(display, root, &root, &child,
              &root_x, &root_y,  // ← Global cursor position!
              &win_x, &win_y, &mask);

printf("Cursor at: %d, %d\n", root_x, root_y);
```

**Wayland:**
```
// Not directly possible!
// Must use:
// 1. Compositor-specific extension (wlr-foreign-toplevel)
// 2. XDG Desktop Portal (requires user permission)
// 3. Only track cursor when over your own window
```

## Relevance to shake-cursor

For detecting cursor shake, we need **continuous access to cursor position**.

### X11 Strategy (Straightforward)

```
1. Connect to X display
2. In a loop:
   a. Call XQueryPointer() to get (x, y)
   b. Store position with timestamp
   c. Analyze recent positions for shake pattern
   d. Sleep briefly (e.g., 10ms)
3. If shake detected → trigger action
```

### Wayland Strategy (Complex)

Since global cursor position is hidden, options are:

| Approach | Pros | Cons |
|----------|------|------|
| **XWayland** | Works like X11 | Only for X11 apps, not native |
| **libportal / xdg-desktop-portal** | Standard API | May require user permission |
| **Compositor extension** | Native access | Different per compositor |
| **Invisible fullscreen window** | Gets all motion events | Hacky, may have focus issues |

## Summary

A **display server protocol** defines:

1. **What messages exist** — CreateWindow, QueryPointer, MotionNotify, etc.
2. **How messages are formatted** — Binary structure with opcodes and payloads
3. **How communication flows** — Request → Reply, and async Events
4. **What guarantees are made** — Ordering, atomicity, error handling

The protocol is the **contract** between applications and the display server. For shake-cursor:

- **X11 protocol** allows direct cursor queries (`XQueryPointer`)
- **Wayland protocol** deliberately omits this for security — requiring workarounds

Understanding the protocol helps us know what's **possible** and what's **restricted** when building our cursor tracking driver.
