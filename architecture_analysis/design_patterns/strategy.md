# Strategy Pattern

## Definition

The **strategy pattern** defines a family of interchangeable algorithms (or behaviors) behind a **common interface**. The code that uses the algorithm doesn't know or care which specific implementation is running — it just calls the interface.

```
┌─────────────────────────────────────────────────────────────┐
│                    Real World Analogy                        │
│                                                             │
│   Getting to work:                                          │
│                                                             │
│   You need to "travel(home → office)".                      │
│   HOW you travel can change without changing your goal:     │
│                                                             │
│   ┌──────────────────────────────────────────┐              │
│   │  You (the caller)                        │              │
│   │                                          │              │
│   │  "I need to get to work"                 │              │
│   │  transport.travel(home, office)          │              │
│   │                                          │              │
│   │  You don't care HOW. You just call       │              │
│   │  travel() and arrive at office.          │              │
│   └──────────────┬───────────────────────────┘              │
│                  │                                          │
│                  │ could be any of:                          │
│                  │                                          │
│   ┌──────────────┼──────────────┬──────────────┐            │
│   ▼              ▼              ▼              ▼            │
│ ┌──────┐    ┌──────┐     ┌──────┐     ┌──────┐            │
│ │ Car  │    │ Bus  │     │ Bike │     │ Walk │            │
│ │      │    │      │     │      │     │      │            │
│ │drive │    │ride  │     │pedal │     │walk  │            │
│ │roads │    │route │     │lanes │     │paths │            │
│ └──────┘    └──────┘     └──────┘     └──────┘            │
│                                                             │
│   Same interface: travel(from, to)                          │
│   Different implementations: car, bus, bike, walk           │
│   Swap one for another without changing YOUR code.          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## The Problem It Solves

Without the strategy pattern, you end up with **if/else chains scattered everywhere**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   WITHOUT strategy pattern:                                         │
│                                                                     │
│   fn get_next_event(protocol: &str) -> Event {                      │
│       if protocol == "x11" {                                        │
│           x11_wait_for_event()                                      │
│       } else if protocol == "wayland" {                             │
│           wayland_dispatch_events()                                 │
│       }                                                             │
│   }                                                                 │
│                                                                     │
│   fn enlarge_cursor(protocol: &str) {                               │
│       if protocol == "x11" {                                        │
│           x11_define_cursor(big)                                    │
│       } else if protocol == "wayland" {                             │
│           wayland_set_cursor(big)                                   │
│       }                                                             │
│   }                                                                 │
│                                                                     │
│   fn restore_cursor(protocol: &str) {                               │
│       if protocol == "x11" {                                        │
│           x11_define_cursor(normal)                                 │
│       } else if protocol == "wayland" {                             │
│           wayland_set_cursor(normal)                                │
│       }                                                             │
│   }                                                                 │
│                                                                     │
│   Problems:                                                         │
│   ❌ Every function has if/else for every protocol                  │
│   ❌ Adding a new protocol means touching EVERY function            │
│   ❌ Easy to forget to handle a protocol in one function            │
│   ❌ Protocol logic is scattered across the entire codebase         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   WITH strategy pattern:                                            │
│                                                                     │
│   fn main_loop(backend: &mut dyn DisplayBackend) {                  │
│       loop {                                                        │
│           let event = backend.next_motion_event();                  │
│           if is_shaking {                                           │
│               backend.enlarge_cursor();                             │
│           }                                                         │
│       }                                                             │
│   }                                                                 │
│                                                                     │
│   Benefits:                                                         │
│   ✅ ZERO if/else for protocols in main logic                       │
│   ✅ Adding new protocol = add one new struct, touch nothing else   │
│   ✅ Impossible to forget a method (compiler enforces trait)        │
│   ✅ All protocol logic in ONE place per protocol                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## The Three Parts

The strategy pattern has three components:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   1. TRAIT (the interface)                                          │
│      Defines WHAT operations exist, not HOW they work.              │
│                                                                     │
│   2. IMPLEMENTATIONS (the strategies)                               │
│      Each one defines HOW a specific variant works.                 │
│                                                                     │
│   3. CALLER (the context)                                           │
│      Uses the trait. Doesn't know which implementation is behind it.│
│                                                                     │
│                                                                     │
│   ┌─────────────────────────────────────────┐                       │
│   │  CALLER (main loop)                     │                       │
│   │                                         │                       │
│   │  backend.next_motion_event()            │                       │
│   │  backend.enlarge_cursor()               │                       │
│   │  backend.restore_cursor()               │                       │
│   │                                         │                       │
│   │  "I don't know if this is X11 or        │                       │
│   │   Wayland. I don't care. I just call    │                       │
│   │   the methods."                         │                       │
│   └──────────────┬──────────────────────────┘                       │
│                  │                                                   │
│                  ▼                                                   │
│   ┌─────────────────────────────────────────┐                       │
│   │  TRAIT (DisplayBackend)                 │                       │
│   │                                         │                       │
│   │  fn next_motion_event() → MotionEvent   │                       │
│   │  fn enlarge_cursor()                    │                       │
│   │  fn restore_cursor()                    │                       │
│   │                                         │                       │
│   │  "These are the operations that MUST    │                       │
│   │   exist. Every backend must have them." │                       │
│   └──────────────┬──────────────────────────┘                       │
│                  │                                                   │
│           ┌──────┴──────┐                                           │
│           ▼             ▼                                           │
│   ┌──────────────┐  ┌──────────────┐                                │
│   │ X11Backend   │  │WaylandBackend│                                │
│   │              │  │              │                                │
│   │ Uses x11rb   │  │ Uses wayland │                                │
│   │ to talk to   │  │ to talk to   │                                │
│   │ Xorg         │  │ compositor   │                                │
│   └──────────────┘  └──────────────┘                                │
│                                                                     │
│   Each implementation has completely different internal code,        │
│   but the caller sees the exact same interface.                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Rust Implementation: Traits

In Rust, the strategy pattern is implemented using **traits**. A trait defines a set of methods that any type can implement:

### Step 1: Define the Trait (Interface)

```rust
/// A motion event from any display server
struct MotionEvent {
    x: i32,
    y: i32,
    timestamp: u64,
}

/// The common interface for all display server backends.
/// Any struct that implements this trait can be used
/// by the main loop.
trait DisplayBackend {
    /// Wait for the next mouse motion event.
    /// Blocks (sleeps) until the mouse moves.
    fn next_motion_event(&mut self) -> Option<MotionEvent>;

    /// Change the cursor to the enlarged size.
    fn enlarge_cursor(&mut self);

    /// Restore the cursor to its original size.
    fn restore_cursor(&mut self);

    /// Check if the cursor is currently enlarged.
    fn is_enlarged(&self) -> bool;
}
```

### Step 2: Implement for X11

```rust
struct X11Backend {
    // X11-specific connection and state
    connection: x11rb::rust_connection::RustConnection,
    root_window: u32,
    normal_cursor: u32,
    big_cursor: u32,
    enlarged: bool,
}

impl DisplayBackend for X11Backend {
    fn next_motion_event(&mut self) -> Option<MotionEvent> {
        // X11-specific: wait for MotionNotify event
        loop {
            let event = self.connection.wait_for_event().ok()?;
            if let Event::MotionNotify(motion) = event {
                return Some(MotionEvent {
                    x: motion.root_x as i32,
                    y: motion.root_y as i32,
                    timestamp: motion.time as u64,
                });
            }
            // Ignore non-motion events
        }
    }

    fn enlarge_cursor(&mut self) {
        // X11-specific: load big cursor, apply to root window
        self.connection.change_window_attributes(
            self.root_window,
            &ChangeWindowAttributesAux::new().cursor(self.big_cursor),
        ).ok();
        self.connection.flush().ok();
        self.enlarged = true;
    }

    fn restore_cursor(&mut self) {
        // X11-specific: restore normal cursor on root window
        self.connection.change_window_attributes(
            self.root_window,
            &ChangeWindowAttributesAux::new().cursor(self.normal_cursor),
        ).ok();
        self.connection.flush().ok();
        self.enlarged = false;
    }

    fn is_enlarged(&self) -> bool {
        self.enlarged
    }
}
```

### Step 3: Implement for Wayland (Future)

```rust
struct WaylandBackend {
    // Wayland-specific connection and state
    // ... (to be implemented later)
    enlarged: bool,
}

impl DisplayBackend for WaylandBackend {
    fn next_motion_event(&mut self) -> Option<MotionEvent> {
        // Wayland-specific: completely different code
        // Uses wl_pointer events, compositor extensions, etc.
        todo!("Wayland implementation")
    }

    fn enlarge_cursor(&mut self) {
        // Wayland-specific: completely different approach
        todo!("Wayland implementation")
    }

    fn restore_cursor(&mut self) {
        todo!("Wayland implementation")
    }

    fn is_enlarged(&self) -> bool {
        self.enlarged
    }
}
```

### Step 4: The Caller Doesn't Care

```rust
/// The main loop works with ANY backend.
/// It doesn't know or care if it's X11 or Wayland.
fn run(backend: &mut dyn DisplayBackend, detector: &mut ShakeDetector) {
    loop {
        // Get next event — could be from X11, Wayland, or anything
        let event = match backend.next_motion_event() {
            Some(e) => e,
            None => break,  // connection lost
        };

        // Shake detection — completely protocol-independent
        detector.record_motion(event.x, event.y, event.timestamp);

        if detector.is_shaking() && !backend.is_enlarged() {
            backend.enlarge_cursor();
        } else if !detector.is_shaking() && backend.is_enlarged() {
            backend.restore_cursor();
        }
    }
}

fn main() {
    // Choose backend at startup based on environment
    let mut backend: Box<dyn DisplayBackend> = if is_wayland() {
        Box::new(WaylandBackend::new())
    } else {
        Box::new(X11Backend::new())
    };

    let mut detector = ShakeDetector::new(Config::default());

    // Run the main loop — same code regardless of backend
    run(&mut *backend, &mut detector);
}
```

## How Adding a New Backend Works

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Today: X11 only                                                   │
│                                                                     │
│   ┌──────────────────────────┐                                      │
│   │  trait DisplayBackend    │                                      │
│   └────────────┬─────────────┘                                      │
│                │                                                    │
│                ▼                                                    │
│   ┌──────────────────────────┐                                      │
│   │  X11Backend              │                                      │
│   └──────────────────────────┘                                      │
│                                                                     │
│                                                                     │
│   Later: Add Wayland support                                        │
│                                                                     │
│   What changes?                                                     │
│                                                                     │
│   ┌──────────────────────────┐                                      │
│   │  trait DisplayBackend    │  ← NO CHANGES                        │
│   └────────────┬─────────────┘                                      │
│                │                                                    │
│         ┌──────┴──────┐                                             │
│         ▼             ▼                                             │
│   ┌────────────┐  ┌────────────────┐                                │
│   │X11Backend  │  │WaylandBackend  │  ← NEW FILE, new code         │
│   │            │  │                │                                │
│   │ NO CHANGES │  │ implements     │                                │
│   │            │  │ DisplayBackend │                                │
│   └────────────┘  └────────────────┘                                │
│                                                                     │
│   main.rs       → ONE new line to choose backend  (minimal change)  │
│   detector.rs   → NO CHANGES (protocol-independent)                 │
│   cursor.rs     → NO CHANGES to trait                               │
│   x11.rs        → NO CHANGES                                       │
│   wayland.rs    → NEW FILE                                          │
│                                                                     │
│   Adding Wayland doesn't break or modify any existing X11 code.     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Static vs Dynamic Dispatch

Rust offers two ways to use traits:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   STATIC DISPATCH (generics):                                       │
│                                                                     │
│   fn run<B: DisplayBackend>(backend: &mut B) { ... }                │
│                                                                     │
│   • Compiler generates separate code for each type                  │
│   • Slightly faster (no indirection)                                │
│   • Backend type must be known at compile time                      │
│   • Binary may be larger (duplicate code)                           │
│                                                                     │
│                                                                     │
│   DYNAMIC DISPATCH (trait objects):                                  │
│                                                                     │
│   fn run(backend: &mut dyn DisplayBackend) { ... }                  │
│                                                                     │
│   • One version of the code, dispatches at runtime                  │
│   • Tiny overhead (one pointer lookup per call)                     │
│   • Backend type chosen at runtime                                  │
│   • Smaller binary                                                  │
│                                                                     │
│                                                                     │
│   For shake-cursor:                                                 │
│                                                                     │
│   Dynamic dispatch (dyn) is the right choice because:               │
│   • We choose the backend at runtime (check DISPLAY env var)        │
│   • The overhead is negligible (one lookup per mouse event)         │
│   • The code is simpler to read                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Concrete Example: Payment Processing

To make the pattern even clearer, here's a non-shake-cursor example:

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   An online store that accepts multiple payment methods:           │
│                                                                    │
│   trait PaymentStrategy {                                          │
│       fn pay(&self, amount: f64) -> Result<(), String>;            │
│       fn name(&self) -> &str;                                      │
│   }                                                                │
│                                                                    │
│   struct CreditCard { number: String }                             │
│   struct PayPal { email: String }                                  │
│   struct Bitcoin { wallet: String }                                │
│                                                                    │
│   impl PaymentStrategy for CreditCard {                            │
│       fn pay(&self, amount: f64) -> Result<(), String> {           │
│           // Charge credit card via Stripe API                     │
│           Ok(())                                                   │
│       }                                                            │
│       fn name(&self) -> &str { "Credit Card" }                     │
│   }                                                                │
│                                                                    │
│   impl PaymentStrategy for PayPal {                                │
│       fn pay(&self, amount: f64) -> Result<(), String> {           │
│           // Redirect to PayPal, process payment                   │
│           Ok(())                                                   │
│       }                                                            │
│       fn name(&self) -> &str { "PayPal" }                          │
│   }                                                                │
│                                                                    │
│   impl PaymentStrategy for Bitcoin {                               │
│       fn pay(&self, amount: f64) -> Result<(), String> {           │
│           // Generate Bitcoin invoice, wait for confirmation       │
│           Ok(())                                                   │
│       }                                                            │
│       fn name(&self) -> &str { "Bitcoin" }                         │
│   }                                                                │
│                                                                    │
│   // The checkout function doesn't care which payment method:      │
│   fn checkout(cart: &Cart, payment: &dyn PaymentStrategy) {        │
│       let total = cart.total();                                    │
│       println!("Paying {} via {}", total, payment.name());         │
│       payment.pay(total).unwrap();                                 │
│   }                                                                │
│                                                                    │
│   // Usage:                                                        │
│   let card = CreditCard { number: "4242..." };                     │
│   checkout(&cart, &card);                                          │
│                                                                    │
│   let paypal = PayPal { email: "me@mail.com" };                    │
│   checkout(&cart, &paypal);  // same function, different strategy  │
│                                                                    │
│   Adding Apple Pay later? Just implement PaymentStrategy.          │
│   checkout() doesn't change. Nothing else changes.                 │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Strategy Pattern in shake-cursor: File Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   src/                                                              │
│   ├── main.rs                                                       │
│   │   • Detects which display server is active                      │
│   │   • Creates the right backend (X11 or Wayland)                  │
│   │   • Passes backend to main loop                                 │
│   │                                                                 │
│   ├── backend.rs                                                    │
│   │   • Defines the DisplayBackend trait                             │
│   │   • Defines MotionEvent struct                                  │
│   │   • This is the CONTRACT that all backends must follow          │
│   │                                                                 │
│   ├── x11_backend.rs                                                │
│   │   • X11Backend struct                                           │
│   │   • impl DisplayBackend for X11Backend                          │
│   │   • All X11-specific code lives HERE and ONLY here              │
│   │                                                                 │
│   ├── wayland_backend.rs  (future)                                  │
│   │   • WaylandBackend struct                                       │
│   │   • impl DisplayBackend for WaylandBackend                      │
│   │   • All Wayland-specific code lives HERE and ONLY here          │
│   │                                                                 │
│   └── detector.rs                                                   │
│       • Shake detection algorithm                                   │
│       • Uses MotionEvent — doesn't know about X11 or Wayland        │
│       • Completely protocol-independent                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## When to Use the Strategy Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   USE when:                                                 │
│   ✅ You have multiple ways to do the same thing            │
│   ✅ You want to swap implementations without changing      │
│      calling code                                           │
│   ✅ You might add more implementations later               │
│   ✅ You want to isolate implementation details             │
│                                                             │
│   DON'T USE when:                                           │
│   ❌ There's only one way to do something                   │
│   ❌ The implementations will never change                  │
│   ❌ A simple if/else is clearer (1-2 cases)               │
│                                                             │
│   shake-cursor: ✅ Two protocols (X11, Wayland)             │
│   with very different implementations but the same goal.    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Summary

| Concept | Definition |
|---------|------------|
| **Strategy Pattern** | Define interchangeable behaviors behind a common interface |
| **Trait** | The Rust interface that defines what methods exist |
| **Implementation** | A struct that provides the actual code for those methods |
| **Caller** | Code that uses the trait without knowing which implementation is behind it |
| **Dynamic Dispatch** | Choosing which implementation to use at runtime (`dyn Trait`) |

For shake-cursor:
- **Trait:** `DisplayBackend` (next_motion_event, enlarge_cursor, restore_cursor)
- **X11Backend:** Implements the trait using x11rb and Xcursor
- **WaylandBackend:** Will implement the same trait using Wayland protocols
- **Main loop:** Calls `backend.next_motion_event()` without caring which protocol
