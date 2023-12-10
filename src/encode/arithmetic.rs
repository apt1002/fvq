use std::cmp::{min, max};
use std::io::{Read, Write, Result};

const SCALE: u64 = 1 << 32;

/// Divide by `SCALE` rounding to even.
fn divide_by_scale(x: u64) -> u32 {
    let nudge = (x / SCALE) & 1;
    ((x + (SCALE / 2 - 1) + nudge) / SCALE) as u32
}

fn eof() -> std::io::Error { std::io::ErrorKind::UnexpectedEof.into() }

// ----------------------------------------------------------------------------

/// Represents a model of the relative probability of `false` and `true`.
#[derive(Debug, Copy, Clone)]
pub struct Split {
    /// `SCALE` times the probability of `true`.
    p1: u32,
}

impl Split {
    fn new_inner(p1: u32) -> Self {
        let p1 = min(p1, !3); // Small enough that `State::below` changes.
        let p1 = max(p1, 4); // Large enough that `State::above` changes.
        Self {p1: p1}
    }

    /// Constructs a `Split` given the probability of `true`.
    pub fn new(p1: f64) -> Self {
        Self::new_inner((SCALE as f64 * p1.clamp(0.0, 1.0)).round() as u32)
    }

    /// Constructs a `Split` given the ratio of the frequency of `false` to
    // the frequency of `true`.
    pub fn new_ratio(f0: u64, f1: u64) -> Self {
        let total = f0.checked_add(f1).expect("Total must be less than 1<<64");
        Self::new(f1 as f64 / total as f64)
    }
}

// ----------------------------------------------------------------------------

/// Represents an interval inside [0, 1].
#[derive(Default, Debug, Copy, Clone)]
struct Interval {
    /// The lower bound minus `0`, times `SCALE`.
    below: u32,

    /// `1` minus the upper bound, times `SCALE`.
    above: u32,
}

impl Interval {
    pub fn new(below: u32, above: u32) -> Self {
        assert!(below.checked_add(above).is_some()); // Non-empty.
        Self {below, above}
    }

    /// Split this `Interval` into two: one for `false` and one for `true`.
    #[must_use]
    pub fn split(self, model: Split) -> (Self, Self) {
        let p1 = model.p1 as u64;
        let p0 = SCALE - p1;
        let below = divide_by_scale(self.below as u64 * p1 + (SCALE * p0) - self.above as u64 * p0);
        let above = divide_by_scale(self.above as u64 * p0 + (SCALE * p1) - self.below as u64 * p1);
        assert_eq!(below.wrapping_add(above), 0);
        (Self::new(self.below, above), Self::new(below, self.above))
    }

    /// Equivalent to, but more efficient than, `self.split(Split::new(0.5))`.
    #[must_use]
    pub fn half(self) -> (Self, Self) {
        const HALF: u64 = SCALE / 2;
        let below = divide_by_scale(self.below as u64 * HALF + HALF - self.above as u64 * HALF);
        let above = divide_by_scale(self.above as u64 * HALF + HALF - self.below as u64 * HALF);
        assert_eq!(below.wrapping_add(above), 0);
        (Self::new(self.below, above), Self::new(below, self.above))
    }

    /// Returns `true` if `self` contains (inclusive) `other`.
    pub fn contains(self, other: Self) -> bool {
        self.below < other.below && self.above < other.above
    }

    /// Applies a twofold enlargement that maps `half` to `WHOLE`.
    /// `half` must contain `self`.
    /// `half` must be exactly half the size of `WHOLE`.
    /// Candidates for `half` include `LOWER`, `MIDDLE` and `UPPER`.
    pub fn grow(&mut self, half: Interval) {
        assert!(half.contains(*self));
        assert_eq!(half.below + half.above, (SCALE / 2) as u32);
        self.below = 2 * (self.below - half.below);
        self.above = 2 * (self.above - half.above);
    }
}

/// The whole Interval [0, 1].
const WHOLE: Interval = Interval {below: 0, above: 0};

/// The lower Interval [0, 0.5].
const LOWER: Interval = Interval {below: 0, above: (SCALE / 2) as u32};

/// The middle Interval [0.25, 0.25].
const MIDDLE: Interval = Interval {below: (SCALE / 4) as u32, above: (SCALE / 4) as u32};

/// The upper Interval [0.5, 1].
const UPPER: Interval = Interval {below: (SCALE / 2) as u32, above: 0};

// ----------------------------------------------------------------------------

/// Reads a stream of bits.
#[derive(Debug)]
pub struct BitReader<T: Read> {
    inner: T,
    buffer: [u8; 1],
    bit: u8,
}

impl<T: Read> BitReader<T> {
    pub fn new(inner: T) -> Self { Self {inner, buffer: [0], bit: 0} }

    /// Read one bit.
    pub fn read(&mut self) -> Result<bool> {
        if self.bit == 0 {
            let n = self.inner.read(&mut self.buffer)?;
            if n < 1 { Err(eof())? }
            self.bit = 1;
        }
        let ret = (self.buffer[0] & self.bit) != 0;
        self.bit <<= 1;
        Ok(ret)
    }

    /// Skip padding.
    pub fn close(self) -> T { self.inner }
}

// ----------------------------------------------------------------------------

/// Read arithmetic-encoded data.
#[derive(Debug)]
pub struct Reader<T: Read> {
    inner: BitReader<T>,
    unfair: Interval,
    fair: Interval,
}

impl<T: Read> Reader<T> {
    pub fn new(inner: BitReader<T>) -> Self {
        Self {inner, unfair: WHOLE, fair: WHOLE}
    }

    /// If `unfair` contains `half`, map `half` to `WHOLE` and return `true`.
    fn grow(&mut self, half: Interval) -> bool {
        if !half.contains(self.unfair) { return false; }
        self.unfair.grow(half);
        self.fair.grow(half);
        true
    }

    /// Read one biased bit.
    pub fn read(&mut self, model: Split) -> Result<bool> {
        assert!(self.unfair.contains(self.fair));
        // Subdivide.
        let data: bool;
        let (i0, i1) = self.unfair.split(model);
        loop {
            if i0.contains(self.fair) { data = false; self.unfair = i0; break; }
            if i1.contains(self.fair) { data = true; self.unfair = i1; break; }
            let (h0, h1) = self.fair.half();
            self.fair = if self.inner.read()? { h1 } else { h0 };
        }
        // Grow to the working range.
        loop {
            if self.grow(LOWER) { continue; }
            if self.grow(UPPER) { continue; }
            break;
        }
        while self.grow(MIDDLE) {}
        Ok(data)
    }

    /// Skip padding.
    pub fn close(self) -> BitReader<T> {
        assert!(self.unfair.contains(self.fair));
        self.inner
    }
}

// ----------------------------------------------------------------------------

/// Writes a stream of bits.
#[derive(Debug)]
pub struct BitWriter<T: Write> {
    inner: T,
    buffer: [u8; 1],
    bit: u8,
}

impl<T: Write> BitWriter<T> {
    pub fn new(inner: T) -> Self {
        Self {inner, buffer: [0], bit: 1}
    }

    /// Write one bit.
    pub fn write(&mut self, data: bool) -> Result<()> {
        if data { self.buffer[0] |= self.bit };
        self.bit <<= 1;
        if self.bit == 0 {
            let n = self.inner.write(&mut self.buffer)?;
            if n < 1 { Err(eof())? }
            self.buffer = [0];
            self.bit = 1;
        }
        Ok(())
    }

    /// Pad as necessary to write all data.
    pub fn close(mut self) -> Result<T> {
        while self.bit != 1 { self.write(false)?; }
        Ok(self.inner)
    }
}

// ----------------------------------------------------------------------------

/// Write arithmetic-encoded data.
#[derive(Debug)]
pub struct Writer<T: Write> {
    inner: BitWriter<T>,
    unfair: Interval,
    middle_count: usize,
}

impl<T: Write> Writer<T> {
    pub fn new(inner: BitWriter<T>) -> Self {
        Self {inner, unfair: WHOLE, middle_count: 0}
    }

    /// If `unfair` contains `half`, map `half` to `WHOLE` and return `true`.
    fn grow(&mut self, half: Interval) -> bool {
        if !half.contains(self.unfair) { return false; }
        self.unfair.grow(half);
        true
    }

    /// Write `data` then `middle_count` copies of `!data`.
    /// Reset `middle_count`.
    fn inner_write(&mut self, data: bool) -> Result<()> {
        self.inner.write(data)?;
        for _ in 0..self.middle_count { self.inner.write(!data)?; }
        self.middle_count = 0;
        Ok(())
    }

    pub fn write(&mut self, model: Split, data: bool) -> Result<()> {
        // Subdivide.
        let (i0, i1) = self.unfair.split(model);
        self.unfair = if data { i1 } else { i0 };
        // Grow to the working range.
        loop {
            if self.grow(LOWER) { self.inner_write(false)?; continue; }
            if self.grow(UPPER) { self.inner_write(true)?; continue; }
            break;
        }
        while self.grow(MIDDLE) { self.middle_count += 1; }
        Ok(())
    }

    /// Pad as necessary to write all data.
    pub fn close(mut self) -> Result<BitWriter<T>> {
        if self.unfair.above > self.unfair.below {
            self.inner_write(false)?;
            if self.unfair.below > 0 {
                self.inner_write(true)?;
            }
        } else if self.unfair.below > self.unfair.above {
            self.inner_write(true)?;
            if self.unfair.above > 0 {
                self.inner_write(false)?;
            }
        }
        Ok(self.inner)
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split() {
        let model = Split::new_inner((SCALE / 8) as u32);
        let (i0, i1) = MIDDLE.split(model);
        println!("i0 = {:x?}", i0);
        println!("i1 = {:x?}", i1);
        assert_eq!(i0.below, MIDDLE.below);
        assert_eq!(i0.above, (SCALE * 5 / 16) as u32);
        assert_eq!(i1.below, (SCALE * 11 / 16) as u32);
        assert_eq!(i1.above, MIDDLE.above);
    }
}
