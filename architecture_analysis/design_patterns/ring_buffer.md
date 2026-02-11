# Ring Buffer (Circular Buffer)

## Definition

A **ring buffer** is a fixed-size data structure where the end connects back to the beginning, forming a circle. When the buffer is full and a new element arrives, it **overwrites the oldest element**. This means:

- It **never grows** beyond its fixed size
- It always contains the **most recent** data
- Old data is **automatically discarded**

```
┌─────────────────────────────────────────────────────────────┐
│                    Real World Analogy                        │
│                                                             │
│   A revolving sushi belt:                                   │
│                                                             │
│   ┌─────────────────────────────────┐                       │
│   │                                 │                       │
│   │    ┌──┐  ┌──┐  ┌──┐  ┌──┐      │                       │
│   │ ──►│E │──│D │──│C │──│B │──┐   │                       │
│   │    └──┘  └──┘  └──┘  └──┘  │   │                       │
│   │    newest              oldest│   │                       │
│   │                             │   │                       │
│   │    ┌──┐                     │   │                       │
│   │    │F │ new plate arrives   │   │                       │
│   │    └──┘                     │   │                       │
│   │      │                      │   │                       │
│   │      ▼                      ▼   │                       │
│   │    F pushes onto belt,          │                       │
│   │    B (oldest) falls off         │                       │
│   │                                 │                       │
│   │    Belt has fixed capacity.     │                       │
│   │    Always has freshest plates.  │                       │
│   │    Old plates gone forever.     │                       │
│   │                                 │                       │
│   └─────────────────────────────────┘                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Why "Ring"?

It's called a ring because the buffer wraps around. When you reach the end, the next write goes back to the beginning:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Linear buffer (normal array):                             │
│                                                             │
│   [A] [B] [C] [D] [E]                                      │
│    0   1   2   3   4   ← can't go further, buffer is full  │
│                                                             │
│                                                             │
│   Ring buffer (wraps around):                               │
│                                                             │
│              ┌──[C]──┐                                      │
│           [D]        [B]                                    │
│              │       │                                      │
│           [E]        [A]                                    │
│              └───────┘                                      │
│                                                             │
│   After adding F:                                           │
│                                                             │
│              ┌──[C]──┐                                      │
│           [D]        [B]                                    │
│              │       │                                      │
│           [E]        [F]  ← overwrote A (oldest)            │
│              └───────┘                                      │
│                                                             │
│   The write position wraps from end back to beginning.      │
│   It's a circle — no beginning or end.                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## How It Works: Step by Step

Let's trace through a ring buffer with **capacity 4**:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Capacity: 4 slots                                                │
│   W = write position (where next element goes)                     │
│                                                                    │
│                                                                    │
│   STEP 1: Empty buffer                                             │
│   ┌────┬────┬────┬────┐                                            │
│   │    │    │    │    │     count = 0                               │
│   └────┴────┴────┴────┘                                            │
│    W                                                               │
│                                                                    │
│                                                                    │
│   STEP 2: Push "A"                                                 │
│   ┌────┬────┬────┬────┐                                            │
│   │ A  │    │    │    │     count = 1                               │
│   └────┴────┴────┴────┘                                            │
│         W                                                          │
│                                                                    │
│                                                                    │
│   STEP 3: Push "B"                                                 │
│   ┌────┬────┬────┬────┐                                            │
│   │ A  │ B  │    │    │     count = 2                               │
│   └────┴────┴────┴────┘                                            │
│              W                                                     │
│                                                                    │
│                                                                    │
│   STEP 4: Push "C"                                                 │
│   ┌────┬────┬────┬────┐                                            │
│   │ A  │ B  │ C  │    │     count = 3                               │
│   └────┴────┴────┴────┘                                            │
│                   W                                                │
│                                                                    │
│                                                                    │
│   STEP 5: Push "D" → buffer is now FULL                            │
│   ┌────┬────┬────┬────┐                                            │
│   │ A  │ B  │ C  │ D  │     count = 4 (full!)                      │
│   └────┴────┴────┴────┘                                            │
│    W                        write wraps back to start              │
│                                                                    │
│                                                                    │
│   STEP 6: Push "E" → A is overwritten!                             │
│   ┌────┬────┬────┬────┐                                            │
│   │ E  │ B  │ C  │ D  │     count = 4 (still 4, not 5)            │
│   └────┴────┴────┴────┘                                            │
│         W                   A is gone. E took its slot.            │
│                                                                    │
│   Reading order (oldest → newest): B, C, D, E                      │
│                                                                    │
│                                                                    │
│   STEP 7: Push "F" → B is overwritten!                             │
│   ┌────┬────┬────┬────┐                                            │
│   │ E  │ F  │ C  │ D  │     count = 4                              │
│   └────┴────┴────┴────┘                                            │
│              W                                                     │
│                                                                    │
│   Reading order (oldest → newest): C, D, E, F                      │
│                                                                    │
│                                                                    │
│   STEP 8: Push "G" → C is overwritten!                             │
│   ┌────┬────┬────┬────┐                                            │
│   │ E  │ F  │ G  │ D  │     count = 4                              │
│   └────┴────┴────┴────┘                                            │
│                   W                                                │
│                                                                    │
│   Reading order (oldest → newest): D, E, F, G                      │
│                                                                    │
│                                                                    │
│   No matter how many elements we push:                             │
│   • Buffer NEVER exceeds 4 elements                                │
│   • We ALWAYS have the 4 most recent                               │
│   • Memory usage is CONSTANT                                       │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Ring Buffer vs Normal Array

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   NORMAL ARRAY (Vec):                                               │
│                                                                     │
│   Push A:  [A]                           memory: 1                  │
│   Push B:  [A, B]                        memory: 2                  │
│   Push C:  [A, B, C]                     memory: 3                  │
│   ...                                                               │
│   Push 1000: [A, B, C, ... 1000]         memory: 1000              │
│                                                                     │
│   Memory GROWS forever.                                             │
│   After 1 hour of mouse events at 100/sec = 360,000 entries!       │
│   After 24 hours = 8,640,000 entries!                               │
│                                                                     │
│   ──────────────────────────────────────────────────────────        │
│                                                                     │
│   RING BUFFER (capacity 50):                                        │
│                                                                     │
│   Push A:    [A]                         memory: 1                  │
│   Push B:    [A, B]                      memory: 2                  │
│   ...                                                               │
│   Push 50:   [A, B, C, ... 50]           memory: 50                │
│   Push 51:   [51, B, C, ... 50]          memory: 50 (A gone)       │
│   Push 1000: [...last 50 events...]      memory: 50 (still!)       │
│                                                                     │
│   Memory is CONSTANT at 50 entries.                                 │
│   After 1 hour = still 50 entries.                                  │
│   After 24 hours = still 50 entries.                                │
│   After 1 year = still 50 entries.                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Performance

Every operation is **O(1)** — constant time, no matter the size:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Operation     Time       Why                              │
│   ───────────   ────────   ───────────────────────────      │
│   Push          O(1)       Write at position, increment     │
│   Read newest   O(1)       Read at write position - 1       │
│   Read oldest   O(1)       Read at write position           │
│   Read by index O(1)       Calculate offset, read           │
│   Is full?      O(1)       Compare count to capacity        │
│                                                             │
│   No searching. No shifting. No resizing.                   │
│   Just math (modulo) and one array access.                  │
│                                                             │
│   The secret: write_pos = (write_pos + 1) % capacity        │
│   This wraps around: 0, 1, 2, 3, 0, 1, 2, 3, 0, ...       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### The Modulo Trick

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   capacity = 4                                              │
│                                                             │
│   write_pos = 0    →  0 % 4 = 0   (slot 0)                 │
│   write_pos = 1    →  1 % 4 = 1   (slot 1)                 │
│   write_pos = 2    →  2 % 4 = 2   (slot 2)                 │
│   write_pos = 3    →  3 % 4 = 3   (slot 3)                 │
│   write_pos = 4    →  4 % 4 = 0   (back to slot 0!)        │
│   write_pos = 5    →  5 % 4 = 1   (slot 1 again)           │
│   write_pos = 6    →  6 % 4 = 2   (slot 2 again)           │
│                                                             │
│   The modulo operator (%) makes it wrap around.             │
│   No if/else needed. Pure math.                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Rust Implementation From Scratch

```rust
struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    write_pos: usize,  // where the next element will be written
    count: usize,      // how many elements are currently stored
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with given capacity
    fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
            count: 0,
        }
    }

    /// Add an element. If full, overwrites the oldest.
    fn push(&mut self, item: T) {
        if self.count < self.capacity {
            // Buffer not full yet, just append
            self.data.push(item);
            self.count += 1;
        } else {
            // Buffer full, overwrite oldest
            self.data[self.write_pos] = item;
        }
        // Move write position forward, wrapping around
        self.write_pos = (self.write_pos + 1) % self.capacity;
    }

    /// Get the number of elements currently stored
    fn len(&self) -> usize {
        self.count
    }

    /// Is the buffer full?
    fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    /// Iterate from oldest to newest
    fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.is_full() {
            self.write_pos  // oldest is right after write position
        } else {
            0  // not full, oldest is at the beginning
        };

        (0..self.count).map(move |i| {
            let index = (start + i) % self.capacity;
            &self.data[index]
        })
    }
}
```

### Example Usage

```rust
fn main() {
    let mut buf = RingBuffer::new(3);  // capacity 3

    buf.push("A");
    buf.push("B");
    buf.push("C");
    // Buffer: [A, B, C]  (full)
    // Iter:   A → B → C

    buf.push("D");
    // Buffer: [D, B, C]  (A overwritten)
    // Iter:   B → C → D

    buf.push("E");
    // Buffer: [D, E, C]  (B overwritten)
    // Iter:   C → D → E

    buf.push("F");
    // Buffer: [D, E, F]  (C overwritten)
    // Iter:   D → E → F

    // Always the 3 most recent elements!
    for item in buf.iter() {
        print!("{} ", item);
    }
    // Output: D E F
}
```

## Rust's Built-in: VecDeque

In practice, Rust provides `VecDeque` from the standard library which works as a double-ended queue and can be used as a ring buffer:

```rust
use std::collections::VecDeque;

fn main() {
    let mut buf: VecDeque<&str> = VecDeque::new();
    let capacity = 3;

    // Helper: push and maintain max size
    let mut push = |buf: &mut VecDeque<&str>, item| {
        if buf.len() >= capacity {
            buf.pop_front();  // remove oldest
        }
        buf.push_back(item);  // add newest
    };

    push(&mut buf, "A");  // [A]
    push(&mut buf, "B");  // [A, B]
    push(&mut buf, "C");  // [A, B, C]
    push(&mut buf, "D");  // [B, C, D]     ← A removed
    push(&mut buf, "E");  // [C, D, E]     ← B removed
    push(&mut buf, "F");  // [D, E, F]     ← C removed

    // Iterate oldest → newest
    for item in &buf {
        print!("{} ", item);
    }
    // Output: D E F
}
```

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   VecDeque vs custom RingBuffer:                            │
│                                                             │
│   VecDeque:                                                 │
│   ✅ Built into Rust standard library                       │
│   ✅ Well tested, optimized                                 │
│   ✅ pop_front() + push_back() = ring buffer behavior       │
│   ✅ Implements Iterator, Index, and many other traits       │
│   ⚠  Slightly more overhead (can grow dynamically)         │
│                                                             │
│   Custom RingBuffer:                                        │
│   ✅ Fixed capacity, never allocates after creation          │
│   ✅ Slightly more efficient for fixed-size use case         │
│   ⚠  Must implement everything yourself                    │
│                                                             │
│   For shake-cursor: VecDeque is the pragmatic choice.       │
│   It's already there, well tested, and good enough.         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## How It Applies to shake-cursor

The shake detector needs a **rolling window** of recent mouse events:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Every MotionNotify event gets stored:                            │
│                                                                    │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  { x: 500, y: 300, time: 1000 }                            │   │
│   │  { x: 540, y: 302, time: 1010 }                            │   │
│   │  { x: 580, y: 301, time: 1020 }                            │   │
│   │  { x: 550, y: 300, time: 1030 }   ← direction reversed!   │   │
│   │  { x: 510, y: 299, time: 1040 }                            │   │
│   │  { x: 560, y: 301, time: 1050 }   ← reversed again!       │   │
│   └────────────────────────────────────────────────────────────┘   │
│     oldest                                              newest     │
│                                                                    │
│   We only keep events from the last 500ms.                         │
│   When a new event arrives:                                        │
│   1. Remove events older than 500ms (pop_front)                    │
│   2. Add new event (push_back)                                     │
│   3. Analyze remaining events for shake pattern                    │
│                                                                    │
│   Without ring buffer: events grow forever (memory leak)           │
│   With ring buffer: always bounded, daemon-safe                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### shake-cursor Implementation

```rust
use std::collections::VecDeque;

struct MotionEvent {
    x: i32,
    y: i32,
    timestamp: u64,  // milliseconds
}

struct EventBuffer {
    events: VecDeque<MotionEvent>,
    time_window_ms: u64,  // e.g., 500ms
}

impl EventBuffer {
    fn new(time_window_ms: u64) -> Self {
        Self {
            events: VecDeque::new(),
            time_window_ms,
        }
    }

    /// Add a new motion event and discard old ones
    fn push(&mut self, event: MotionEvent) {
        // Remove events outside the time window
        let cutoff = event.timestamp.saturating_sub(self.time_window_ms);
        while let Some(front) = self.events.front() {
            if front.timestamp < cutoff {
                self.events.pop_front();  // remove oldest
            } else {
                break;  // remaining events are within window
            }
        }

        // Add new event
        self.events.push_back(event);
    }

    /// Get all events in the buffer (oldest → newest)
    fn events(&self) -> &VecDeque<MotionEvent> {
        &self.events
    }

    /// How many events are currently stored
    fn len(&self) -> usize {
        self.events.len()
    }
}
```

### Example Trace

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   time_window_ms = 500                                             │
│                                                                    │
│   t=1000  push(500, 300, 1000)                                     │
│           buffer: [(500,300,1000)]                                  │
│           size: 1                                                  │
│                                                                    │
│   t=1200  push(600, 300, 1200)                                     │
│           buffer: [(500,300,1000), (600,300,1200)]                  │
│           size: 2                                                  │
│                                                                    │
│   t=1400  push(480, 300, 1400)                                     │
│           buffer: [(500,300,1000), (600,300,1200), (480,300,1400)]  │
│           size: 3                                                  │
│                                                                    │
│   t=1600  push(620, 300, 1600)                                     │
│           cutoff = 1600 - 500 = 1100                               │
│           (500,300,1000) is older than 1100 → REMOVED              │
│           buffer: [(600,300,1200), (480,300,1400), (620,300,1600)]  │
│           size: 3 (one removed, one added)                         │
│                                                                    │
│   t=1800  push(450, 300, 1800)                                     │
│           cutoff = 1800 - 500 = 1300                               │
│           (600,300,1200) is older than 1300 → REMOVED              │
│           buffer: [(480,300,1400), (620,300,1600), (450,300,1800)]  │
│           size: 3                                                  │
│                                                                    │
│   The buffer always contains only events from the last 500ms.      │
│   Memory never grows unbounded.                                    │
│   Perfect for a daemon running 24/7.                               │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Where Ring Buffers Are Used

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Application              What it stores                   │
│   ──────────────────────   ──────────────────────           │
│   Audio processing         Last N samples of sound          │
│   Network logging          Last N packets received          │
│   Game input               Last N frames of input           │
│   Terminal scrollback      Last N lines of output           │
│   CPU monitoring           Last N seconds of CPU usage      │
│   shake-cursor             Last 500ms of mouse positions    │
│                                                             │
│   All share the same need:                                  │
│   "Keep recent data, discard old data, fixed memory"        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Summary

| Property | Ring Buffer |
|----------|------------|
| **Size** | Fixed, never grows |
| **Push** | O(1), overwrites oldest when full |
| **Memory** | Constant, bounded |
| **Data** | Always has the N most recent items |
| **Best for** | Streaming data where only recent values matter |
| **Rust type** | `VecDeque` with `pop_front` + `push_back` |
| **In shake-cursor** | Stores mouse events from the last 500ms |
