# XInput2 Coordinates — Fp1616 Fixed-Point Format

## What Is a Coordinate in XInput2?

When the X server sends a `MotionNotify` event via **XInput2**, the cursor position is
**not** given as a plain integer like `420`. Instead, it arrives as a special numeric
format called **Fp1616** — a 32-bit fixed-point number that encodes both an integer
pixel position and a fractional sub-pixel offset inside a single `i32` value.

```
┌────────────────────────────────────────────────────────────────────┐
│                       One Fp1616 value                             │
│                                                                    │
│   Bits:   31 ─────────────── 16  15 ──────────────────── 0        │
│           │                   │  │                        │        │
│           └──── upper 16 ─────┘  └──────── lower 16 ─────┘        │
│                  INTEGER part            FRACTIONAL part           │
│                 (whole pixels)          (fractions of a pixel)     │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

The name **Fp1616** is shorthand for **Fixed-Point 16.16** — 16 bits for the integer
part and 16 bits for the fractional part.

---

## Why Fixed-Point Instead of a Plain Integer?

### The Problem: Sub-Pixel Input Devices

When X11 was extended with XInput2, it needed to accommodate devices that produce
motion data with **more precision than one pixel**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Types of Input Devices                           │
│                                                                     │
│   Device               Resolution      Can Report Sub-Pixels?       │
│   ─────────────────    ───────────     ──────────────────────       │
│   Standard mouse       ~400–800 DPI    No (jumps pixel-to-pixel)    │
│   High-DPI mouse       4000–8000 DPI   Yes (fractional positions)   │
│   Drawing tablet       ~5080 LPI       Yes (very fine sub-pixels)   │
│   Touchpad             Software        Yes (smooth tracking)        │
│   Touch screen         Hardware grid   Yes (between pixel centers)  │
│                                                                     │
│   A 4000 DPI mouse can physically detect movement of               │
│   1/4000 of an inch — far smaller than one screen pixel            │
│   (a typical pixel on a 96 DPI screen is 1/96 of an inch).        │
│                                                                     │
│   Sub-pixel data would be LOST if we stored positions as integers. │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

If the protocol discarded sub-pixel data and sent plain integers, a high-precision
stylus hovering at position `(420.75, 310.25)` would be rounded to `(420, 310)` — and
that rounding error would accumulate over time, causing drift in precision drawing
applications.

### The Solution: Fixed-Point Numbers

A **fixed-point number** represents a real number by scaling it and storing the result
as an integer. In Fp1616:

```
Real value  =  raw_bits / 2^16
            =  raw_bits / 65536
```

Or equivalently, the binary point (analogous to the decimal point) is placed between
bit 15 and bit 16:

```
Bit position:  31  30  …  17  16  |  15  14  …   1   0
               ───────────────────┼───────────────────
Meaning:       2^15 2^14… 2^1 2^0 | 2^-1 2^-2… 2^-15 2^-16
               (integer digits)      (fractional digits)
```

---

## Anatomy of an Fp1616 Value

### Example 1: Whole pixel, no fraction

```
┌────────────────────────────────────────────────────────────────────┐
│   Pixel position: exactly 420                                      │
│                                                                    │
│   As Fp1616:                                                       │
│   420 × 65536 = 27,525,120 = 0x01A4_0000                          │
│                                                                    │
│   Binary layout:                                                   │
│   ┌────────────────┬────────────────┐                              │
│   │   0000_0001_1010_0100   (=420)  │   0000_0000_0000_0000 (=.0) │
│   └────────────────┴────────────────┘                              │
│   Upper 16 bits               Lower 16 bits                        │
│                                                                    │
│   Integer part:    0x01A4  = 420  pixels                           │
│   Fractional part: 0x0000  = 0/65536 = 0.0                        │
│   Real value:      420.0                                           │
└────────────────────────────────────────────────────────────────────┘
```

### Example 2: Fractional position

```
┌────────────────────────────────────────────────────────────────────┐
│   Pixel position: 420.5 (exactly halfway between pixels 420–421)  │
│                                                                    │
│   As Fp1616:                                                       │
│   420.5 × 65536 = 27,557,888 = 0x01A4_8000                        │
│                                                                    │
│   Binary layout:                                                   │
│   ┌────────────────┬────────────────┐                              │
│   │ 0000_0001_1010_0100  (=420)     │ 1000_0000_0000_0000 (=.5)   │
│   └────────────────┴────────────────┘                              │
│                                                                    │
│   Integer part:    0x01A4  = 420      pixels                       │
│   Fractional part: 0x8000  = 32768/65536 = 0.5                    │
│   Real value:      420.5                                           │
└────────────────────────────────────────────────────────────────────┘
```

### Example 3: Fine sub-pixel position

```
┌────────────────────────────────────────────────────────────────────┐
│   Pixel position: 420.25                                           │
│                                                                    │
│   As Fp1616:                                                       │
│   420.25 × 65536 = 27,541,504 = 0x01A4_4000                       │
│                                                                    │
│   Integer part:    0x01A4  = 420                                   │
│   Fractional part: 0x4000  = 16384/65536 = 0.25                   │
│   Real value:      420.25                                          │
└────────────────────────────────────────────────────────────────────┘
```

---

## Fixed-Point vs Floating-Point

You might ask: why not use a `float` (f32) instead? Both can represent fractional values.

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Fixed-Point vs Floating-Point                      │
│                                                                     │
│   FIXED-POINT (Fp1616)               FLOATING-POINT (f32)           │
│                                                                     │
│   • Integer arithmetic only          • CPU float unit required      │
│   • Exact representation of          • Rounding errors for          │
│     multiples of 1/65536               most values                 │
│   • Predictable performance          • Variable precision           │
│   • Simple bit operations            • More complex hardware        │
│   • Designed for coordinates         • General purpose              │
│                                                                     │
│   In 1987 when X11 was designed, floating-point was expensive      │
│   on many hardware architectures. Fixed-point was the practical    │
│   choice for a protocol that had to run on everything.             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## How to Decode Fp1616 in Code

### Method A: Bitwise right shift (what shake-cursor uses)

```rust
// Discard the fractional part entirely. Fast, exact for integers.
let pixel_x: i16 = (motion.root_x >> 16) as i16;
```

**Mechanism:** `>> 16` shifts all bits 16 positions to the right, which is identical
to dividing by 65536 and discarding the remainder.

```
Before shift:   0x 01A4 8000   (binary: ...0001 1010 0100  1000 0000 0000 0000)
                   ────  ────
                   ↑    ↑
                integer  fractional

After >> 16:    0x 0000 01A4   (binary: ...0000 0000 0000  0001 1010 0100)
                                                           ↑
                                                     integer part only = 420
```

This is the **cheapest possible operation** — one CPU instruction, zero branches.

### Method B: Integer division

```rust
// Equivalent to >> 16, but reads more literally.
let pixel_x: i32 = motion.root_x / 65536;
```

### Method C: Floating-point conversion (when you need the fraction)

```rust
// Preserve sub-pixel precision for drawing applications.
let pixel_x: f64 = motion.root_x as f64 / 65536.0;
// e.g.  27_557_888 / 65536.0 = 420.5
```

---

## Why shake-cursor Only Needs `>> 16`

The shake detection algorithm works at the granularity of **whole pixels**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Shake detection needs:                                            │
│   • Did the cursor change direction?                                │
│     → Compare sign of (current_x - previous_x)                    │
│     → Sub-pixel differences are irrelevant                          │
│                                                                     │
│   • How fast is the cursor moving?                                  │
│     → Velocity in pixels-per-second                                 │
│     → A 0.5 px difference at this timescale is noise               │
│                                                                     │
│   • Is the user shaking?                                            │
│     → Requires movements of 50–200 pixels between reversals        │
│     → Sub-pixel precision adds no value                             │
│                                                                     │
│   Using >> 16 to truncate to integer pixels:                        │
│   ✅ Correct for shake detection purposes                           │
│   ✅ Fastest possible operation (single shift instruction)          │
│   ✅ Avoids float arithmetic entirely                               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Where Fp1616 Appears in XInput2

The `XInuptMotion` event (called `DeviceMotionNotify` in the C API) carries several
Fp1616 fields:

```
┌─────────────────────────────────────────────────────────────────────┐
│   XInput2 DeviceMotionNotify event fields                           │
│                                                                     │
│   Field          Type      Meaning                                  │
│   ────────────   ───────   ──────────────────────────────────────   │
│   root_x         Fp1616    Cursor X relative to the root window     │
│   root_y         Fp1616    Cursor Y relative to the root window     │
│   event_x        Fp1616    Cursor X relative to the event window    │
│   event_y        Fp1616    Cursor Y relative to the event window    │
│   valuators[]    Fp3232    Raw axis values (absolute devices)       │
│                                                                     │
│   root_x / root_y are what shake-cursor reads — the global         │
│   cursor position on screen, regardless of which window is focused. │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   Fp1616 (Fixed-Point 16.16)                                        │
│                                                                     │
│   • A 32-bit integer that encodes a real number                     │
│   • Upper 16 bits = integer (pixel) part                            │
│   • Lower 16 bits = fractional (sub-pixel) part                    │
│   • Range: roughly –32768.0 to +32767.99998                        │
│   • Precision: 1/65536 of a pixel ≈ 0.0000153                      │
│                                                                     │
│   To get whole-pixel coordinate:                                    │
│       pixel = fp1616_value >> 16                                    │
│                                                                     │
│   To get real-valued coordinate:                                    │
│       real  = fp1616_value as f64 / 65536.0                        │
│                                                                     │
│   Used in XInput2 because high-precision devices (tablets,         │
│   high-DPI mice) can sense movement smaller than one pixel.        │
│                                                                     │
│   shake-cursor uses >> 16 because whole-pixel resolution is        │
│   sufficient for shake detection.                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```
