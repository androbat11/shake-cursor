# Design Patterns

## Overview

Design patterns are **reusable solutions to common problems** in software design. They're not code you copy-paste — they're blueprints for how to structure your code to solve a specific kind of problem.

For shake-cursor, five patterns are relevant:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Pattern               Where it applies                    │
│   ───────────────────   ──────────────────────────────      │
│   State Machine         Cursor states (idle/enlarged)       │
│   Observer              Subscribing to X11 events           │
│   Strategy              Swapping X11/Wayland backends       │
│   Ring Buffer           Rolling event buffer for detection  │
│   Builder               Configuration construction          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 1. State Machine Pattern

### What It Is

A **state machine** models something that can be in one of several states, and transitions between them based on events. At any moment, the system is in **exactly one** state.

```
┌─────────────────────────────────────────────────────────────┐
│                    General Concept                           │
│                                                             │
│   A traffic light is a state machine:                       │
│                                                             │
│   ┌───────┐  timer  ┌────────┐  timer  ┌───────┐           │
│   │ GREEN │────────►│ YELLOW │────────►│  RED  │           │
│   └───────┘         └────────┘         └───┬───┘           │
│       ▲                                    │               │
│       └────────────── timer ───────────────┘               │
│                                                             │
│   • Only one state at a time                                │
│   • Clear transitions (what causes a change)                │
│   • Predictable behavior                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### How It Applies to shake-cursor

The cursor has three states:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│                       ┌──────────┐                                 │
│              start ──►│   IDLE   │                                 │
│                       │          │                                 │
│                       │ cursor:  │                                 │
│                       │ normal   │                                 │
│                       └────┬─────┘                                 │
│                            │                                       │
│                            │ shake detected                        │
│                            ▼                                       │
│                       ┌──────────┐                                 │
│                       │ ENLARGED │                                 │
│                       │          │◄────────┐                       │
│                       │ cursor:  │         │ still shaking         │
│                       │ big      │─────────┘ (reset cooldown)      │
│                       └────┬─────┘                                 │
│                            │                                       │
│                            │ cooldown expired (2s no shake)        │
│                            ▼                                       │
│                       ┌──────────┐                                 │
│                       │   IDLE   │ (cursor restored to normal)     │
│                       └──────────┘                                 │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Rust Implementation

```rust
enum CursorState {
    Idle,
    Enlarged { since: u64 },  // timestamp when enlarged
}

impl CursorState {
    fn handle_event(&self, is_shaking: bool, now: u64, cooldown: u64) -> CursorState {
        match self {
            CursorState::Idle => {
                if is_shaking {
                    // Transition: Idle → Enlarged
                    CursorState::Enlarged { since: now }
                } else {
                    CursorState::Idle
                }
            }
            CursorState::Enlarged { since } => {
                if is_shaking {
                    // Still shaking → reset cooldown timer
                    CursorState::Enlarged { since: now }
                } else if now - since > cooldown {
                    // Cooldown expired → restore
                    CursorState::Idle
                } else {
                    // Waiting for cooldown
                    CursorState::Enlarged { since: *since }
                }
            }
        }
    }
}
```

### Why This Pattern Fits

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Without state machine:                                    │
│                                                             │
│   let mut is_big = false;                                   │
│   let mut last_shake = 0;                                   │
│   // Scattered if/else throughout the code                  │
│   // Easy to forget a case                                  │
│   // Hard to reason about behavior                          │
│                                                             │
│   With state machine:                                       │
│                                                             │
│   let state = CursorState::Idle;                            │
│   state = state.handle_event(is_shaking, now, cooldown);    │
│   // ALL transitions in one place                           │
│   // Every case is explicit                                 │
│   // Impossible to be in an invalid state                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 2. Observer Pattern

### What It Is

The **observer pattern** is when one part of the system (the **subject**) notifies other parts (the **observers**) when something happens. Observers register their interest, then get called when events occur.

```
┌─────────────────────────────────────────────────────────────┐
│                    General Concept                           │
│                                                             │
│   A YouTube channel is the observer pattern:                │
│                                                             │
│   ┌──────────┐     subscribe      ┌───────────┐            │
│   │ Channel  │◄───────────────────│ Viewer A  │            │
│   │ (subject)│◄───────────────────│ Viewer B  │            │
│   │          │◄───────────────────│ Viewer C  │            │
│   └────┬─────┘                    └───────────┘            │
│        │                                                    │
│        │ new video uploaded                                 │
│        │                                                    │
│        ├── notify Viewer A                                  │
│        ├── notify Viewer B                                  │
│        └── notify Viewer C                                  │
│                                                             │
│   Viewers don't check repeatedly. They get notified.        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### How It Applies to shake-cursor

This is exactly how X11 event handling works:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   shake-cursor                          X Server (Xorg)            │
│   (observer)                            (subject)                  │
│                                                                    │
│   1. SUBSCRIBE                                                     │
│      shake-cursor tells X server:                                  │
│      "Notify me when the mouse moves on the root window"           │
│                                                                    │
│      XSelectInput(root_window, PointerMotionMask)                  │
│                                                                    │
│   2. WAIT                                                          │
│      shake-cursor sleeps (0% CPU)                                  │
│                                                                    │
│   3. NOTIFY                                                        │
│      Mouse moves → X server sends MotionNotify to shake-cursor     │
│                                                                    │
│   4. REACT                                                         │
│      shake-cursor wakes up, processes the event                    │
│                                                                    │
│   This IS the observer pattern:                                    │
│   • Subject: X server (owns the mouse state)                       │
│   • Observer: shake-cursor (wants to know about changes)           │
│   • Event: MotionNotify (the notification)                         │
│   • Subscribe: XSelectInput (register interest)                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Why This Pattern Fits

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Without observer (polling):                               │
│   loop {                                                    │
│       pos = query_pointer()  // ask every 10ms              │
│       sleep(10ms)            // waste CPU when idle         │
│   }                                                         │
│                                                             │
│   With observer (event-driven):                             │
│   subscribe_to_motion_events()                              │
│   loop {                                                    │
│       event = wait_for_event()  // sleep until notified     │
│       handle(event)                                         │
│   }                                                         │
│                                                             │
│   The X server already tracks mouse position.               │
│   We just observe its changes. Zero wasted work.            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 3. Strategy Pattern

### What It Is

The **strategy pattern** defines a family of interchangeable algorithms behind a common interface. The caller doesn't know which algorithm is being used — it just calls the interface.

```
┌─────────────────────────────────────────────────────────────┐
│                    General Concept                           │
│                                                             │
│   A navigation app with multiple route strategies:          │
│                                                             │
│   ┌──────────────────┐                                      │
│   │   Navigator      │                                      │
│   │                  │                                      │
│   │ navigate(A → B)  │                                      │
│   └───────┬──────────┘                                      │
│           │                                                 │
│           │ uses one of:                                     │
│           │                                                 │
│   ┌───────┴───────┬──────────────┬──────────────┐           │
│   │   Fastest     │   Shortest   │   Scenic     │           │
│   │   Route       │   Route      │   Route      │           │
│   └───────────────┴──────────────┴──────────────┘           │
│                                                             │
│   Same interface, different implementations.                │
│   Swap strategies without changing the navigator.           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### How It Applies to shake-cursor

We want to support **both X11 and Wayland** behind a common interface:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   shake-cursor main loop                                           │
│   ┌──────────────────────────────────────────┐                     │
│   │                                          │                     │
│   │   loop {                                 │                     │
│   │       event = backend.next_event()       │                     │
│   │       if is_shaking:                     │                     │
│   │           backend.enlarge_cursor()       │                     │
│   │   }                                      │                     │
│   │                                          │                     │
│   │   "backend" could be X11 or Wayland.     │                     │
│   │   The main loop doesn't care which one.  │                     │
│   │                                          │                     │
│   └──────────────────┬───────────────────────┘                     │
│                      │                                              │
│           ┌──────────┴──────────┐                                   │
│           ▼                     ▼                                   │
│   ┌──────────────────┐  ┌──────────────────┐                       │
│   │  X11 Backend     │  │  Wayland Backend │                       │
│   │                  │  │                  │                       │
│   │  next_event():   │  │  next_event():   │                       │
│   │  XNextEvent()    │  │  wl_display_     │                       │
│   │                  │  │  dispatch()      │                       │
│   │  enlarge():      │  │                  │                       │
│   │  XDefineCursor() │  │  enlarge():      │                       │
│   │                  │  │  (compositor     │                       │
│   │  restore():      │  │   specific)      │                       │
│   │  XDefineCursor() │  │                  │                       │
│   └──────────────────┘  └──────────────────┘                       │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Rust Implementation (Trait)

```rust
/// Common interface for display server backends
trait DisplayBackend {
    /// Wait for the next mouse motion event (blocks)
    fn next_motion_event(&mut self) -> Option<MotionEvent>;

    /// Enlarge the cursor to the configured size
    fn enlarge_cursor(&mut self);

    /// Restore cursor to original size
    fn restore_cursor(&mut self);
}

struct X11Backend { /* X11 connection, cursor handles */ }
struct WaylandBackend { /* Wayland connection */ }

impl DisplayBackend for X11Backend {
    fn next_motion_event(&mut self) -> Option<MotionEvent> {
        // Use x11rb: conn.wait_for_event()
    }
    fn enlarge_cursor(&mut self) {
        // Use Xcursor to load big cursor, XDefineCursor to apply
    }
    fn restore_cursor(&mut self) {
        // Load original cursor, XDefineCursor to apply
    }
}

impl DisplayBackend for WaylandBackend {
    // Future implementation
}
```

### Why This Pattern Fits

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Without strategy:                                         │
│                                                             │
│   if using_x11 {                                            │
│       x11_next_event()                                      │
│   } else if using_wayland {                                 │
│       wayland_next_event()                                  │
│   }                                                         │
│   // if/else scattered throughout ALL code                  │
│   // Every function needs to know about both protocols      │
│                                                             │
│   With strategy:                                            │
│                                                             │
│   backend.next_motion_event()                               │
│   // One line. Don't care which protocol.                   │
│   // Add new backends without touching existing code.       │
│                                                             │
│   The main loop, detector, and state machine                │
│   are completely protocol-independent.                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 4. Ring Buffer (Circular Buffer)

### What It Is

A **ring buffer** is a fixed-size data structure that overwrites the oldest data when full. It's like a conveyor belt — new items push in, old items fall off.

```
┌─────────────────────────────────────────────────────────────┐
│                    General Concept                           │
│                                                             │
│   A security camera that records on a loop:                 │
│   • Records over the oldest footage when disk is full       │
│   • Always has the MOST RECENT footage available            │
│   • Fixed storage size, never grows                         │
│                                                             │
│   Ring buffer with capacity 5:                              │
│                                                             │
│   Step 1: Add A         [A, _, _, _, _]                     │
│   Step 2: Add B         [A, B, _, _, _]                     │
│   Step 3: Add C         [A, B, C, _, _]                     │
│   Step 4: Add D         [A, B, C, D, _]                     │
│   Step 5: Add E         [A, B, C, D, E]  ← full            │
│   Step 6: Add F         [F, B, C, D, E]  ← A overwritten   │
│   Step 7: Add G         [F, G, C, D, E]  ← B overwritten   │
│                                                             │
│   Newest data always available.                             │
│   Oldest data automatically discarded.                      │
│   Memory usage never grows.                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### How It Applies to shake-cursor

The shake detector needs a **rolling window** of recent mouse events:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   We only care about events from the last 500ms.                   │
│   Older events are irrelevant for shake detection.                 │
│                                                                    │
│   Ring buffer of motion events:                                    │
│                                                                    │
│   ┌───────────────────────────────────────────────────────────┐    │
│   │ (500,300,t=100) (540,300,t=110) (520,305,t=120) ...      │    │
│   │    oldest ──────────────────────────────── newest         │    │
│   └───────────────────────────────────────────────────────────┘    │
│                                                                    │
│   On each new MotionNotify:                                        │
│   1. Add new event to buffer                                       │
│   2. Remove events older than 500ms                                │
│   3. Analyze remaining events for shake                            │
│                                                                    │
│   Benefits:                                                        │
│   • Fixed memory: never allocates more than N events               │
│   • Fast: O(1) add, no shifting elements                           │
│   • Always has the freshest data for analysis                      │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Rust Implementation

```rust
use std::collections::VecDeque;

struct MotionEvent {
    x: i32,
    y: i32,
    timestamp: u64,
}

struct EventBuffer {
    events: VecDeque<MotionEvent>,
    time_window_ms: u64,
}

impl EventBuffer {
    fn new(time_window_ms: u64) -> Self {
        Self {
            events: VecDeque::new(),
            time_window_ms,
        }
    }

    fn push(&mut self, event: MotionEvent) {
        // Remove events outside the time window
        let cutoff = event.timestamp.saturating_sub(self.time_window_ms);
        while let Some(front) = self.events.front() {
            if front.timestamp < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
        // Add new event
        self.events.push_back(event);
    }

    fn events(&self) -> &VecDeque<MotionEvent> {
        &self.events
    }
}
```

### Why This Pattern Fits

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Without ring buffer:                                      │
│                                                             │
│   let mut events = Vec::new();                              │
│   // Grows forever! After hours of use:                     │
│   // events.len() == 1,000,000+                             │
│   // Memory usage keeps growing                             │
│   // Must scan entire vec to find recent events             │
│                                                             │
│   With ring buffer:                                         │
│                                                             │
│   // Fixed size, only recent events                         │
│   // After hours of use: still only ~50 events              │
│   // Memory usage constant                                  │
│   // All events are relevant (within time window)           │
│                                                             │
│   Perfect for a daemon that runs for days/weeks.            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 5. Builder Pattern

### What It Is

The **builder pattern** constructs complex objects step by step. Instead of a constructor with many parameters, you chain method calls to set each option.

```
┌─────────────────────────────────────────────────────────────┐
│                    General Concept                           │
│                                                             │
│   Ordering a coffee:                                        │
│                                                             │
│   Without builder (one giant order):                        │
│   order_coffee("latte", "large", "oat milk",               │
│                "extra shot", "no sugar", "hot")             │
│   // What is each parameter? Hard to read.                  │
│                                                             │
│   With builder (step by step):                              │
│   Coffee::builder()                                         │
│       .kind("latte")                                        │
│       .size("large")                                        │
│       .milk("oat")                                          │
│       .extra_shot()                                         │
│       .no_sugar()                                           │
│       .hot()                                                │
│       .build()                                              │
│   // Each step is clear and self-documenting.               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### How It Applies to shake-cursor

Configuration has multiple optional parameters with defaults:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Without builder:                                                 │
│                                                                    │
│   let config = Config::new(500, 3, 500.0, 2000, 96);              │
│   // What do these numbers mean?!                                  │
│                                                                    │
│                                                                    │
│   With builder:                                                    │
│                                                                    │
│   let config = Config::builder()                                   │
│       .time_window_ms(500)                                         │
│       .min_reversals(3)                                            │
│       .min_velocity(500.0)                                         │
│       .cooldown_ms(2000)                                           │
│       .enlarged_size(96)                                           │
│       .build();                                                    │
│                                                                    │
│   Or with all defaults:                                            │
│                                                                    │
│   let config = Config::builder().build();  // all defaults         │
│                                                                    │
│   Or override just one:                                            │
│                                                                    │
│   let config = Config::builder()                                   │
│       .min_reversals(4)  // more sensitive                         │
│       .build();                                                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Rust Implementation

```rust
struct Config {
    time_window_ms: u64,
    min_reversals: u32,
    min_velocity: f64,
    cooldown_ms: u64,
    enlarged_size: u32,
}

impl Config {
    fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

struct ConfigBuilder {
    time_window_ms: u64,
    min_reversals: u32,
    min_velocity: f64,
    cooldown_ms: u64,
    enlarged_size: u32,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            time_window_ms: 500,
            min_reversals: 3,
            min_velocity: 500.0,
            cooldown_ms: 2000,
            enlarged_size: 96,
        }
    }
}

impl ConfigBuilder {
    fn time_window_ms(mut self, ms: u64) -> Self {
        self.time_window_ms = ms;
        self
    }
    fn min_reversals(mut self, n: u32) -> Self {
        self.min_reversals = n;
        self
    }
    // ... other setters ...
    fn build(self) -> Config {
        Config {
            time_window_ms: self.time_window_ms,
            min_reversals: self.min_reversals,
            min_velocity: self.min_velocity,
            cooldown_ms: self.cooldown_ms,
            enlarged_size: self.enlarged_size,
        }
    }
}
```

## Pattern Map: Where Each Pattern Lives

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   shake-cursor/src/                                                 │
│                                                                     │
│   main.rs                                                           │
│   ├── Observer Pattern      (subscribe to X11 events)               │
│   ├── State Machine Pattern (manage cursor state transitions)       │
│   └── Builder Pattern       (construct Config)                      │
│                                                                     │
│   detector.rs                                                       │
│   └── Ring Buffer Pattern   (rolling window of motion events)       │
│                                                                     │
│   cursor.rs                                                         │
│   └── Strategy Pattern      (DisplayBackend trait)                  │
│       └── X11Backend        (first implementation)                  │
│       └── WaylandBackend    (future implementation)                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Summary

| Pattern | Problem It Solves | Where In shake-cursor |
|---------|------------------|-----------------------|
| **State Machine** | Managing cursor state without scattered if/else | `main.rs` — IDLE/ENLARGED transitions |
| **Observer** | Reacting to events without polling | `main.rs` — X11 MotionNotify subscription |
| **Strategy** | Supporting multiple backends behind one interface | `cursor.rs` — DisplayBackend trait for X11/Wayland |
| **Ring Buffer** | Fixed-memory rolling window of recent events | `detector.rs` — VecDeque of motion events |
| **Builder** | Constructing config with optional params + defaults | `main.rs` — Config::builder().build() |
