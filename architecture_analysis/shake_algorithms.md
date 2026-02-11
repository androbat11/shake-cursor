# Shake Detection Algorithm

## Goal

Detect when the user is **shaking** the cursor (rapid back-and-forth movement) and distinguish it from **normal** cursor movement.

## What Defines a Shake?

A shake has three properties:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   1. DIRECTION REVERSALS                                    │
│      Cursor repeatedly changes direction                    │
│      →→→←←←→→→←←←                                          │
│                                                             │
│   2. HIGH VELOCITY                                          │
│      Cursor is moving fast between reversals                │
│      Not slow, careful back-and-forth                       │
│                                                             │
│   3. SHORT TIME WINDOW                                      │
│      All reversals happen within a brief period             │
│      Not spread over several seconds                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Data We Collect

Each `MotionNotify` event gives us:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   MotionNotify Event                                        │
│   {                                                         │
│       x: i32,          // cursor X position (pixels)        │
│       y: i32,          // cursor Y position (pixels)        │
│       timestamp: u64,  // time in milliseconds              │
│   }                                                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

We store a **rolling buffer** of recent events:

```
┌─────────────────────────────────────────────────────────────┐
│                    Rolling Buffer                           │
│                                                             │
│   Stores the last N motion events (e.g., last 50 events)   │
│   Old events are discarded as new ones arrive               │
│                                                             │
│   Index:  [0]      [1]      [2]      [3]      [4]  ...     │
│   X:      500      520      540      530      510  ...     │
│   Y:      300      300      301      300      300  ...     │
│   Time:   1000     1010     1020     1030     1040 ...     │
│                                                             │
│   Newest events are added at the end.                       │
│   Events older than the time window are removed.            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Step 1: Calculate Direction

For each consecutive pair of events, determine which direction the cursor moved on each axis:

```
Direction Formula:

  direction_x = current.x - previous.x
  direction_y = current.y - previous.y

  If direction_x > 0 → moving RIGHT
  If direction_x < 0 → moving LEFT
  If direction_x = 0 → no horizontal movement

  If direction_y > 0 → moving DOWN
  If direction_y < 0 → moving UP
  If direction_y = 0 → no vertical movement
```

### Example: Horizontal Shake

```
  Events:  (500,300) → (540,300) → (580,300) → (550,300) → (510,300)

  Movement:
  500 → 540 = +40  (RIGHT)
  540 → 580 = +40  (RIGHT)
  580 → 550 = -30  (LEFT)   ← REVERSAL!
  550 → 510 = -40  (LEFT)

  Direction X:  [+, +, -, -]
  Reversals:    1 (at position 3, changed from + to -)
```

### Example: Strong Shake

```
  Events:  (500,300) → (600,300) → (450,300) → (620,300) → (430,300)

  Movement:
  500 → 600 = +100 (RIGHT)
  600 → 450 = -150 (LEFT)   ← REVERSAL!
  450 → 620 = +170 (RIGHT)  ← REVERSAL!
  620 → 430 = -190 (LEFT)   ← REVERSAL!

  Direction X:  [+, -, +, -]
  Reversals:    3
```

## Step 2: Count Direction Reversals

A **reversal** occurs when the sign of the direction changes:

```
Reversal Detection Formula:

  For each consecutive pair of directions (d1, d2):

  reversal = (d1 > 0 AND d2 < 0) OR (d1 < 0 AND d2 > 0)

  In other words: the sign flipped.
```

We check reversals on **both axes independently**:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   X axis reversals:                                         │
│   directions: [+, +, -, -, +, +, -, -]                      │
│   reversals:       ↑  ↑        ↑  ↑     = 4 reversals      │
│                    here here   here here                    │
│                                                             │
│   Y axis reversals:                                         │
│   directions: [0, 0, 0, 0, 0, 0, 0, 0]                     │
│   reversals:                             = 0 reversals      │
│                                                             │
│   Total shake strength = max(x_reversals, y_reversals)      │
│                        = max(4, 0) = 4                      │
│                                                             │
│   We use max() because a shake typically happens on         │
│   ONE axis (left-right OR up-down), not both.               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Step 3: Calculate Velocity

Velocity tells us how fast the cursor is moving. Slow back-and-forth is normal usage, not a shake.

```
Velocity Formula:

  For each consecutive pair of events:

  dx = current.x - previous.x
  dy = current.y - previous.y
  dt = current.timestamp - previous.timestamp    (milliseconds)

  distance = sqrt(dx² + dy²)                     (pixels)
  velocity = distance / dt                        (pixels per millisecond)
  velocity_per_sec = velocity * 1000              (pixels per second)
```

### Example Calculations

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Slow movement (normal usage):                             │
│   (500,300) → (510,300) in 100ms                            │
│   distance = sqrt(10² + 0²) = 10 pixels                    │
│   velocity = 10 / 100 = 0.1 px/ms = 100 px/sec             │
│                                                             │
│   Fast movement (likely shake):                             │
│   (500,300) → (600,300) in 50ms                             │
│   distance = sqrt(100² + 0²) = 100 pixels                  │
│   velocity = 100 / 50 = 2.0 px/ms = 2000 px/sec            │
│                                                             │
│   Very fast shake:                                          │
│   (500,300) → (700,300) in 30ms                             │
│   distance = sqrt(200² + 0²) = 200 pixels                  │
│   velocity = 200 / 30 = 6.67 px/ms = 6667 px/sec           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Average Velocity Over Time Window

Instead of checking one pair, we calculate the **average velocity** over the time window:

```
Average Velocity Formula:

  total_distance = sum of all distances between consecutive events
  total_time     = last_timestamp - first_timestamp

  avg_velocity = total_distance / total_time    (pixels per ms)
```

## Step 4: The Complete Detection Formula

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                    SHAKE DETECTION FORMULA                           │
│                                                                     │
│   Given:                                                            │
│     events[]     = rolling buffer of recent MotionNotify events     │
│     TIME_WINDOW  = 500ms (only consider events in last 500ms)       │
│     MIN_REVERSALS = 3 (minimum direction changes to count as shake) │
│     MIN_VELOCITY  = 500 px/sec (minimum speed threshold)            │
│                                                                     │
│   Algorithm:                                                        │
│                                                                     │
│   1. Filter events to only those within TIME_WINDOW                 │
│                                                                     │
│   2. Calculate directions between consecutive events:               │
│      for i in 1..events.len():                                      │
│          dx[i] = events[i].x - events[i-1].x                       │
│          dy[i] = events[i].y - events[i-1].y                       │
│                                                                     │
│   3. Count reversals on each axis:                                  │
│      x_reversals = count sign changes in dx[]                       │
│      y_reversals = count sign changes in dy[]                       │
│      reversals = max(x_reversals, y_reversals)                      │
│                                                                     │
│   4. Calculate average velocity:                                    │
│      total_dist = sum(sqrt(dx[i]² + dy[i]²)) for all i             │
│      total_time = last_event.time - first_event.time                │
│      avg_velocity = (total_dist / total_time) * 1000                │
│                                                                     │
│   5. Determine if shaking:                                          │
│                                                                     │
│      is_shaking = (reversals >= MIN_REVERSALS)                      │
│                   AND                                                │
│                   (avg_velocity >= MIN_VELOCITY)                     │
│                                                                     │
│                                                                     │
│   SHAKE = reversals >= 3  AND  avg_velocity >= 500 px/sec           │
│           within the last 500ms                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Visual Example: Full Detection

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   Time (ms):  0     50    100   150   200   250   300              │
│   X position: 500   600   480   620   460   610   470              │
│                 └─┐   └─┐   └─┐   └─┐   └─┐   └─┐                │
│                   ▼     ▼     ▼     ▼     ▼     ▼                  │
│   dx:           +100  -120  +140  -160  +150  -140                 │
│   direction:      +     -     +     -     +     -                  │
│                      ↑     ↑     ↑     ↑     ↑                     │
│                      R     R     R     R     R                     │
│                                                                    │
│   Reversals: 5                                                     │
│                                                                    │
│   Distances: 100 + 120 + 140 + 160 + 150 + 140 = 810 pixels       │
│   Time span: 300 - 0 = 300ms                                      │
│   Avg velocity: (810 / 300) * 1000 = 2700 px/sec                  │
│                                                                    │
│   Check:                                                           │
│     reversals (5) >= MIN_REVERSALS (3)  ✅                          │
│     velocity (2700) >= MIN_VELOCITY (500) ✅                        │
│                                                                    │
│   RESULT: SHAKE DETECTED!                                          │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Visual Example: Normal Usage (NOT a Shake)

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   User carefully positioning cursor on a button:                   │
│                                                                    │
│   Time (ms):  0     200   400   600   800   1000                   │
│   X position: 500   510   505   512   508   511                    │
│                 └─┐   └─┐   └─┐   └─┐   └─┐                       │
│                   ▼     ▼     ▼     ▼     ▼                        │
│   dx:            +10    -5    +7    -4    +3                       │
│   direction:      +     -     +     -     +                        │
│                      ↑     ↑     ↑     ↑                           │
│                      R     R     R     R                           │
│                                                                    │
│   Reversals: 4                                                     │
│                                                                    │
│   Distances: 10 + 5 + 7 + 4 + 3 = 29 pixels                      │
│   Time span: 1000 - 0 = 1000ms                                    │
│   Avg velocity: (29 / 1000) * 1000 = 29 px/sec                    │
│                                                                    │
│   Check:                                                           │
│     reversals (4) >= MIN_REVERSALS (3)  ✅                          │
│     velocity (29) >= MIN_VELOCITY (500) ❌                          │
│                                                                    │
│   RESULT: NOT A SHAKE (too slow)                                   │
│                                                                    │
│   The velocity threshold filters out small, slow adjustments.      │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Visual Example: Fast Straight Movement (NOT a Shake)

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   User quickly moving cursor to the other side of the screen:      │
│                                                                    │
│   Time (ms):  0     30    60    90    120   150                    │
│   X position: 100   300   500   700   900   1100                   │
│                 └─┐   └─┐   └─┐   └─┐   └─┐                       │
│                   ▼     ▼     ▼     ▼     ▼                        │
│   dx:           +200  +200  +200  +200  +200                       │
│   direction:      +     +     +     +     +                        │
│                                                                    │
│   Reversals: 0                                                     │
│                                                                    │
│   Distances: 200 * 5 = 1000 pixels                                │
│   Time span: 150ms                                                 │
│   Avg velocity: (1000 / 150) * 1000 = 6667 px/sec                 │
│                                                                    │
│   Check:                                                           │
│     reversals (0) >= MIN_REVERSALS (3)  ❌                          │
│     velocity (6667) >= MIN_VELOCITY (500) ✅                        │
│                                                                    │
│   RESULT: NOT A SHAKE (no reversals, just fast movement)           │
│                                                                    │
│   The reversal threshold filters out fast straight-line movement.  │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Why Both Conditions Are Needed

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                        HIGH VELOCITY                                │
│                  ┌──────────┬──────────┐                             │
│                  │  Fast    │  Fast    │                             │
│                  │  straight│  back &  │                             │
│   HIGH           │  line    │  forth   │                             │
│   REVERSALS      │          │          │                             │
│                  │ NOT shake│  SHAKE!  │  ← Only this quadrant      │
│                  ├──────────┼──────────┤                             │
│                  │  Still   │  Slow    │                             │
│   LOW            │  cursor  │  careful │                             │
│   REVERSALS      │          │  adjust  │                             │
│                  │ NOT shake│ NOT shake│                             │
│                  └──────────┴──────────┘                             │
│                  LOW VELOCITY  HIGH VELOCITY                        │
│                                                                     │
│   Both conditions together isolate true shakes from:                │
│   • Fast straight movement (high velocity, no reversals)            │
│   • Careful adjustments (reversals, but low velocity)               │
│   • Idle cursor (neither)                                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Configurable Thresholds

These values are starting points. They should be tunable:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Parameter        Default    Purpose                       │
│   ──────────────   ────────   ──────────────────────        │
│   TIME_WINDOW      500ms      How far back to look          │
│   MIN_REVERSALS    3          Direction changes needed       │
│   MIN_VELOCITY     500 px/s   Speed threshold                │
│   COOLDOWN         2000ms     Time before cursor shrinks     │
│                               back to normal size            │
│                                                             │
│   Lower MIN_REVERSALS → more sensitive (easier to trigger)  │
│   Higher MIN_REVERSALS → less sensitive (harder to trigger)  │
│                                                             │
│   Lower MIN_VELOCITY → triggers on slower shakes            │
│   Higher MIN_VELOCITY → only triggers on fast shakes        │
│                                                             │
│   These will be tuned through testing.                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Pseudocode

```
struct MotionEvent {
    x: i32,
    y: i32,
    timestamp: u64,
}

const TIME_WINDOW: u64 = 500;       // ms
const MIN_REVERSALS: u32 = 3;
const MIN_VELOCITY: f64 = 500.0;    // px/sec

fn detect_shake(events: &[MotionEvent]) -> bool {
    // 1. Filter to events within time window
    let now = events.last().timestamp;
    let recent: Vec<&MotionEvent> = events
        .iter()
        .filter(|e| now - e.timestamp <= TIME_WINDOW)
        .collect();

    if recent.len() < 3 {
        return false;  // need at least 3 events
    }

    // 2. Calculate directions and distances
    let mut x_reversals = 0;
    let mut y_reversals = 0;
    let mut total_distance = 0.0;
    let mut prev_dx: i32 = 0;
    let mut prev_dy: i32 = 0;

    for i in 1..recent.len() {
        let dx = recent[i].x - recent[i-1].x;
        let dy = recent[i].y - recent[i-1].y;
        let distance = ((dx*dx + dy*dy) as f64).sqrt();
        total_distance += distance;

        // 3. Count reversals (sign change)
        if i > 1 {
            if (prev_dx > 0 && dx < 0) || (prev_dx < 0 && dx > 0) {
                x_reversals += 1;
            }
            if (prev_dy > 0 && dy < 0) || (prev_dy < 0 && dy > 0) {
                y_reversals += 1;
            }
        }

        prev_dx = dx;
        prev_dy = dy;
    }

    // 4. Calculate average velocity
    let time_span = recent.last().timestamp - recent.first().timestamp;
    if time_span == 0 {
        return false;
    }
    let avg_velocity = (total_distance / time_span as f64) * 1000.0;

    // 5. Check both conditions
    let reversals = max(x_reversals, y_reversals);

    reversals >= MIN_REVERSALS && avg_velocity >= MIN_VELOCITY
}
```

## Summary

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   SHAKE = many direction reversals + high velocity          │
│           within a short time window                        │
│                                                             │
│   Formula:                                                  │
│     1. Collect mouse positions with timestamps              │
│     2. Keep only events within last 500ms                   │
│     3. Count direction reversals on X and Y axes            │
│     4. Calculate average velocity (pixels/second)           │
│     5. If reversals >= 3 AND velocity >= 500 px/s → SHAKE  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```
