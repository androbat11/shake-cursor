# Vibecoding Learning Strategy

## What Is This File?

This project (shake-cursor) is being built as a learning exercise. The goal is not just to produce working code, but to deeply understand every concept involved. This file defines the strategy for turning vibecoding into retained knowledge.

## Learning Principles

1. **Understand before implementing** — Read the architecture docs, then implement without looking at them
2. **Test your understanding** — After each module, answer the self-check questions from memory
3. **Break things on purpose** — Remove a design decision and observe what goes wrong
4. **Teach it back** — If you can't explain a concept to a beginner without notes, you don't know it yet
5. **Journal your surprises** — Record what was harder/easier than expected in `learnings.md`

## Implementation Order

Build modules bottom-up, from simplest to most complex. Each step is testable in isolation.

| Order | Module           | Concepts Practiced                              | Depends On   |
|-------|------------------|-------------------------------------------------|--------------|
| 1     | `config.rs`      | Structs, Default trait, Builder pattern          | Nothing      |
| 2     | `detector.rs`    | VecDeque, algorithm design, unit testing         | config       |
| 3     | `backend.rs`     | Traits, Strategy pattern, struct definitions     | Nothing      |
| 4     | `x11_backend.rs` | x11rb API, system programming, error handling    | backend      |
| 5     | `main.rs`        | Event loop, State Machine, signals, wiring       | Everything   |

## Per-Module Routine

For each module, follow this cycle:

### Phase 1: Comprehend

- Read the relevant architecture doc(s)
- Close the docs
- Write down in your own words what the module does and why

### Phase 2: Implement

- Write the code without looking at the architecture docs
- If stuck, re-read only the specific section you need
- Write at least 2 unit tests before moving on

### Phase 3: Verify Understanding

- Answer the self-check questions (below) without looking at code or docs
- If any answer feels fuzzy, go back to Phase 1 for that concept

### Phase 4: Break It

- Remove or change a key design decision
- Observe and document the consequence
- Restore the original code

### Phase 5: Journal

- Write what surprised you in `learnings.md`
- Note any Rust-specific lessons (ownership, borrowing, lifetimes)

## Self-Check Questions

### config.rs

- Why use a Builder instead of `Config::new(500, 3, 500.0, 2000, 96)`?
- What does `self` in `fn time_window_ms(mut self, ms: u64) -> Self` enable?
- What happens if someone calls `.build()` without setting any values?

### detector.rs

- Why use VecDeque instead of Vec?
- Why does the algorithm need BOTH reversals AND velocity?
- What happens with only velocity (no reversal check)?
- What happens with only reversals (no velocity check)?
- Why `max(x_reversals, y_reversals)` instead of summing them?
- What would happen without the 500ms time window?
- Why do we need at least 3 events to detect a shake?

### backend.rs

- What is a trait in Rust and how does it relate to the Strategy pattern?
- If you add Wayland support, which files need to change?
- What is the difference between `dyn DisplayBackend` and `impl DisplayBackend`?
- Why does the main loop not need to know which protocol is running?

### x11_backend.rs

- What does `wait_for_event()` do at the OS level when no events exist?
- Why does this use 0% CPU when idle?
- What is a MotionNotify event and who generates it?
- Why can any X11 client query global cursor position?

### main.rs

- Draw the state machine from memory (IDLE → ENLARGED → IDLE)
- What happens if we don't handle SIGTERM?
- Why is there a 2-second cooldown before restoring the cursor?
- What happens if the daemon crashes without restoring the cursor?
- How does systemd recover from a daemon crash?

## Break-It Experiments

After each module works, try these intentional breakages and document what happens:

| Experiment                                  | Expected Consequence                                      |
|---------------------------------------------|-----------------------------------------------------------|
| Remove time-window pruning from detector    | Memory grows without bound (leak in a 24/7 daemon)        |
| Change `wait_for_event()` to a polling loop | CPU usage jumps to ~1-5% constantly                       |
| Remove SIGTERM handler                      | Cursor stays enlarged after daemon is killed               |
| Set `MIN_REVERSALS = 1`                     | Normal fast cursor corrections trigger false positives     |
| Set `MIN_VELOCITY = 0`                      | Slow careful mouse adjustments trigger false positives     |
| Remove the cooldown timer                   | Cursor flickers between big/small during brief pause       |
| Use `Vec` instead of `VecDeque`             | `remove(0)` shifts all elements — O(n) instead of O(1)    |

## Concept-to-Code Mapping

Fill this in as you implement. It links architecture concepts to actual Rust code:

| Concept            | Architecture Doc             | Rust Code                           |
|--------------------|------------------------------|-------------------------------------|
| Builder pattern    | `design_patterns.md`         | `Config::builder().build()`         |
| Ring Buffer        | `design_patterns/ring_buffer.md` | `VecDeque::push_back / pop_front`   |
| Strategy pattern   | `design_patterns/strategy.md`    | `trait DisplayBackend`              |
| State Machine      | `design_patterns.md`         | `enum CursorState` + `match`        |
| Observer pattern   | `design_patterns.md`         | `conn.wait_for_event()`             |
| Event-driven I/O   | `cpu_on_event.md`            | Blocking `read()` on X11 socket     |
| Daemon lifecycle   | `daemon.md`                  | systemd service file + signal hook  |

## Milestones

Track your progress here. Mark each with a date when completed.

- [ ] `config.rs` implemented and tested
- [ ] `detector.rs` implemented and tested
- [ ] `backend.rs` trait defined
- [ ] `x11_backend.rs` connects to X server and receives events
- [ ] `main.rs` event loop detects shakes and enlarges cursor
- [ ] Signal handling restores cursor on SIGTERM
- [ ] Daemon runs via systemd and auto-restarts on crash
- [ ] All break-it experiments completed and documented
- [ ] Self-check questions answered from memory
- [ ] `learnings.md` has at least 10 entries
