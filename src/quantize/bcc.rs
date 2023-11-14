/// Represents a point of the shifted body-centred cubic lattice.
///
/// We use such points to represent quantised wavelet coefficients. Wavelets
/// come in triplets, indexed by [`VHC`]. Therefore, we call the coordinates
/// `v`, `h` and `c`.
///
/// The body-centred cubic lattice is the optimal 3D quantisation lattice, i.e.
/// if you round a uniformly random point in 3D space onto a point of the BCC
/// lattice then the expected norm of the rounding error is smaller than for
/// any other lattice with the same density. We orient and scale the lattice
/// such that the shortest lattice vectors are `(±1, ±1, ±1)` (of norm 3).
///
/// `ShiftedBCC`s are formed from the points of the BCC lattice by adding a
/// constant vector `b`. We choose `b` such that `(±1, 0, ½)` and `(0, ±1, -½)`
/// are `ShiftedBCC`s. These are in fact the nearest four `ShiftedBCC`s to the
/// origin, which is not a `ShiftedBCC`. Interpreted as wavelet coefficients,
/// they are related by 90° rotations.
///
/// Given `ShiftedBCC`s `A` and `B`, write `A → B` if `½A` is closer to `B`
/// than to any other `ShiftedBCC`. Our choice of `b` minimises the expected
/// norm of the [`Residual`] `½A - B` (for uniformly random A).
///
/// Define a "chain" to be a sequence of points `Qₙ → ... → Q₀` related by `→`,
/// such that `Q₀` is one of the four points near the origin, but no other `Qᵢ`
/// is. Thus, there is a unique chain beginning with each quantisation point.
/// The length of the chain is `n`.
///
/// [`VHC`]: crate::VHC
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ShiftedBCC {
    /// The `v` coordinate minus `1.0`.
    v: i16,
    /// The `h` coordinate.
    h: i16,
    /// The `c` coordinate minus `0.5`.
    c: i16,
}

/// Round to the nearest even integer.
fn round2(x: f32) -> f32 { 2.0 * (x * 0.5).round() }

/// L2 norm.
fn norm(v: f32, h: f32, c: f32) -> f32 { v * v + h * h + c * c }

impl ShiftedBCC {
    fn new_inner(v: i16, h: i16, c: i16) -> Self {
        assert_eq!(v & 1, c & 1, "Not a quantisation point");
        assert_eq!(h & 1, c & 1, "Not a quantisation point");
        Self {v, h, c}
    }

    /// Construct a `ShiftedBCC` given its 3D coordinates.
    ///
    /// # Panics
    ///
    /// Panics if `(v, h, c)` is not a quantisation point.
    /// Undefined if it is further from the origin than about `32767`.
    pub fn new(v: f32, h: f32, c: f32) -> Self {
        Self::new_inner((v - 1.0) as i16, (h - 0.0) as i16, (c - 0.5) as i16)
    }

    pub fn v(self) -> f32 { self.v as f32 + 1.0 }
    pub fn h(self) -> f32 { self.h as f32 + 0.0 }
    pub fn c(self) -> f32 { self.c as f32 + 0.5 }

    /// Returns the coordinates of `self`.
    pub fn vhc(self) -> (f32, f32, f32) { (self.v(), self.h(), self.c()) }

    /// Returns the nearest `ShiftedBCC` to `(v, h, c)`, and the L2 norm of the
    /// difference.
    ///
    /// Undefined if it is further from the origin than about `32767`.
    pub fn quantize(v: f32, h: f32, c: f32) -> (Self, f32) {
        let v1 = round2(v - 1.0) + 1.0;
        let h1 = round2(h - 0.0) + 0.0;
        let c1 = round2(c - 0.5) + 0.5;
        let norm1 = norm(v - v1, h - h1, c - c1);
        let v2 = round2(v + 0.0) - 0.0;
        let h2 = round2(h + 1.0) - 1.0;
        let c2 = round2(c + 0.5) - 0.5;
        let norm2 = norm(v - v2, h - h2, c - c2);
        if norm1 < norm2 {
            (Self::new(v1, h1, c1), norm1)
        } else {
            (Self::new(v2, h2, c2), norm2)
        }
    }

    /// Finds the nearest `ShiftedBCC` to `½ self`, and returns it and the
    /// [`Residual`].
    pub fn arrow(self) -> (Self, Residual) {
        let v_bit = (self.v as usize >> 1) & 1;
        let h_bit = (self.h as usize >> 1) & 1;
        let c_bits = self.c as usize & 3;
        let residual = SYNDROMES[v_bit][h_bit][c_bits];
        let (dv, dh, dc) = DELTAS[residual.0 as usize];
        let destination = Self::new_inner(
            (self.v - dv) >> 1,
            (self.h - dh) >> 1,
            (self.c - dc) >> 1,
        );
        (destination, residual)
    }
}

// ----------------------------------------------------------------------------

const RESIDUALS: [(f32, f32, f32); 8] = [
    ( 0.0, -0.5, -0.75),
    ( 0.0,  0.5, -0.75),
    (-0.5,  0.0, -0.25),
    ( 0.5,  0.0, -0.25),
    ( 0.0, -0.5,  0.25),
    ( 0.0,  0.5,  0.25),
    (-0.5,  0.0,  0.75),
    ( 0.5,  0.0,  0.75),
];

/// A simple function of `RESIDUALS` used to implement [`ShiftedBCC::arrow()`].
const DELTAS: [(i16, i16, i16); 8] = [
    ( 1, -1, -1),
    ( 1,  1, -1),
    ( 0,  0,  0),
    ( 2,  0,  0),
    ( 1, -1,  1),
    ( 1,  1,  1),
    ( 0,  0,  2),
    ( 2,  0,  2),
];

/// Represents `½A - B` where `A → B`. See [`ShiftedBCC::arrow()`].
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Residual(u8);

impl Residual {
    /// Returns the components of `self`.
    pub fn vhc(self) -> (f32, f32, f32) { RESIDUALS[self.0 as usize] }
}

/// All possible [`Residual`]s.
pub const ALL_RESIDUALS: [Residual; 8] = [
    Residual(0),
    Residual(1),
    Residual(2),
    Residual(3),
    Residual(4),
    Residual(5),
    Residual(6),
    Residual(7),
];

/// Used to implement [`ShiftedBCC::arrow()`].
const SYNDROMES: [[[Residual; 4]; 2]; 2] = [[
    [Residual(2), Residual(5), Residual(6), Residual(1)],
    [Residual(7), Residual(4), Residual(3), Residual(0)],
], [
    [Residual(3), Residual(0), Residual(7), Residual(4)],
    [Residual(6), Residual(1), Residual(2), Residual(5)],
]];

// ----------------------------------------------------------------------------

const ROTATIONS: [(f32, f32, f32); 4] = [
    ( 1.0,  0.0,  0.5),
    ( 0.0,  1.0, -0.5),
    (-1.0,  0.0,  0.5),
    ( 0.0, -1.0, -0.5),
];

const ROTATION_RESIDUALS: [Residual; 4] = [
    Residual(2),
    Residual(4),
    Residual(3),
    Residual(5),
];

/// Represents one of the four shortest `ShiftedBCC`s: the fixed points of
/// [`ShiftedBCC::arrow()`].
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Rotation(u8);

impl Rotation {
    /// Returns the components of `self`.
    pub fn vhc(self) -> (f32, f32, f32) { ROTATIONS[self.0 as usize] }

    /// Returns `½self - self`.
    pub fn residual(self) -> Residual { ROTATION_RESIDUALS[self.0 as usize] }
}

/// All possible [`Rotation`]s.
pub const ALL_ROTATIONS: [Rotation; 4] = [
    Rotation(0),
    Rotation(1),
    Rotation(2),
    Rotation(3),
];

// ----------------------------------------------------------------------------

/// Represents a `ShiftedBCC` as a chain of `arrow()`s.
///
/// Each `arrow() operation roughly halves the `ShiftedBCC`, and produces a
/// [`Residual`]. Eventually the chain reaches a fixed point of `arrow()`,
/// which is a [`Rotation`] of the bias vector `b`.
///
/// [`arrow()`]: ShiftedBCC::arrow
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct Chain {
    /// The [`Residual`]s, listed from least to most significant.
    pub residuals: Vec<Residual>,
    /// The [`Rotation`], which is even more significant.
    pub rotation: Rotation,
}

impl Chain {
    /// Convert a `ShiftedBCC` to a `Self`.
    pub fn from_bcc(mut bcc: ShiftedBCC) -> Self {
        let mut residuals = Vec::new();
        loop {
            let (half, residual) = bcc.arrow();
            if bcc == half { break; }
            residuals.push(residual);
            bcc = half;
        }
        let v_bit = (bcc.v + 1) & 2;
        let h_bit = bcc.h & 2;
        let c_bit = bcc.c & 1;
        let rotation = Rotation((v_bit + h_bit + c_bit) as u8);
        Chain {residuals, rotation}
    }

    /// Convert wavelet coefficients to a `Self`.
    pub fn quantize(v: f32, h: f32, c: f32) -> Self {
        Self::from_bcc(ShiftedBCC::quantize(v, h, c).0)
    }

    /// Convert self to wavelet coefficients.
    pub fn vhc(&self) -> (f32, f32, f32) {
        let (mut v, mut h, mut c) = self.rotation.vhc();
        for r in self.residuals.iter().rev() {
            let (dv, dh, dc) = r.vhc();
            v = (v + dv) * 2.0;
            h = (h + dh) * 2.0;
            c = (c + dc) * 2.0;
        }
        (v, h, c)
    }

    /// Convert `self` to a `ShiftedBCC`.
    pub fn to_bcc(&self) -> ShiftedBCC {
        let (v, h, c) = self.vhc();
        ShiftedBCC::new(v, h, c)
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for (v, h, c) in [
            ( 0.0, -1.0, -0.5),
            ( 0.0,  1.0, -0.5),
            (-1.0,  0.0,  0.5),
            ( 1.0,  0.0,  0.5),
        ] {
            let bcc = ShiftedBCC::new(v, h, c);
            assert_eq!(bcc.vhc(), (v, h, c));
        }
    }

    #[test]
    fn arrow() {
        let check = |a: ShiftedBCC| {
            let (observed_b, observed_r) = a.arrow();
            // Check the destination.
            let (expected_b, error_norm) = ShiftedBCC::quantize(0.5 * a.v(), 0.5 * a.h(), 0.5 * a.c());
            assert!(error_norm <= 1.25);
            assert_eq!(observed_b, expected_b,
                "{:?}.arrow() gives destination {:?} (should be {:?})", a, observed_b, expected_b,
            );
            // Check the residual.
            let expected_r = (
                0.5 * a.v() - expected_b.v(),
                0.5 * a.h() - expected_b.h(),
                0.5 * a.c() - expected_b.c(),
            );
            assert_eq!(observed_r.vhc(), expected_r,
                "{:?}.arrow() gives residual {:?} (should be {:?})", a, observed_r.vhc(), expected_r,
            );
        };
        for v in [-2.0, 0.0, 2.0] {
            for h in [-2.0, 0.0, 2.0] {
                for c in [-2.0, 0.0, 2.0] {
                    check(ShiftedBCC::new(v + 1.0, h, c + 0.5));
                    check(ShiftedBCC::new(v, h - 1.0, c - 0.5));
                }
            }
        }
    }

    #[test]
    fn short_chain() {
        for (v, h, c) in [
            ( 0.0, -1.0, -0.5),
            ( 0.0,  1.0, -0.5),
            (-1.0,  0.0,  0.5),
            ( 1.0,  0.0,  0.5),
        ] {
            let bcc = ShiftedBCC::new(v, h, c);
            let chain = Chain::from_bcc(bcc);
            assert_eq!(chain.residuals, []);
            assert_eq!(chain.vhc(), (v, h, c));
            let (bcc2, residual) = bcc.arrow();
            assert_eq!(bcc, bcc2);
            assert_eq!(residual, chain.rotation.residual());
        }
    }

    #[test]
    fn long_chain() {
        let bcc = ShiftedBCC::new(8.0, -13.0, -4.5);
        let chain = Chain::from_bcc(bcc);
        assert_eq!(chain.residuals.len(), 4);
        let new_bcc = chain.to_bcc();
        assert_eq!(bcc, new_bcc);
    }
}
