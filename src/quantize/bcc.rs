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

/// A self-inverse symmetry operation. A subset of:
/// - Exchange `v` with `h` and negate `c`.
/// - Exchange `v` with `-h` and negate `c`.
// Private: represented by bits `0` and `1` respectively.
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Symmetry(u8);

/// All possible [`Residual`]s.
pub const ALL_SYMMETRIES: [Symmetry; 4] = [
    Symmetry(0), Symmetry(1), Symmetry(2), Symmetry(3),
];

const RESIDUALS: [(f32, f32, f32); 8] = [
    (-0.5,  0.0,  0.75),
    ( 0.0, -0.5, -0.75),
    ( 0.0,  0.5, -0.75),
    ( 0.5,  0.0,  0.75),
    (-0.5,  0.0, -0.25),
    ( 0.0, -0.5,  0.25),
    ( 0.0,  0.5,  0.25),
    ( 0.5,  0.0, -0.25),
];

/// A simple function of `RESIDUALS` used to implement [`ShiftedBCC::arrow()`].
const DELTAS: [(i16, i16, i16); 8] = [
    ( 0,  0,  2),
    ( 1, -1, -1),
    ( 1,  1, -1),
    ( 2,  0,  2),
    ( 0,  0,  0),
    ( 1, -1,  1),
    ( 1,  1,  1),
    ( 2,  0,  0),
];

/// Represents `½A - B` where `A → B`. See [`ShiftedBCC::arrow()`].
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Residual(u8);

impl Residual {
    /// Returns the components of `self`.
    pub fn vhc(self) -> (f32, f32, f32) { RESIDUALS[self.0 as usize] }

    /// Returns the components of the unique [`ShiftedBCC`] `fp` such that
    /// `fp.arrow()` is `(fp, self)`.
    pub fn fixed_point(self) -> (f32, f32, f32) {
        let (v, h, c) = self.vhc();
        (-2.0 * v, -2.0 * h, -2.0 * c)
    }

    /// Returns the unique [`Symmetry`] that maps `self` onto either
    /// `ALL_RESIDUALS[0]` or `ALL_RESIDUALS[4]`.
    pub fn recommend_symmetry(self) -> Symmetry {
        Symmetry(self.0 & 3)
    }

    /// Applies `symmetry` to `self`. This is its own inverse.
    pub fn apply_symmetry(self, s: Symmetry) -> Self {
        Self(self.0 ^ s.0)
    }
}

/// All possible [`Residual`]s.
pub const ALL_RESIDUALS: [Residual; 8] = [
    Residual(0), Residual(1), Residual(2), Residual(3),
    Residual(4), Residual(5), Residual(6), Residual(7),
];

/// Used to implement [`ShiftedBCC::arrow()`].
const SYNDROMES: [[[Residual; 4]; 2]; 2] = [[
    [Residual(4), Residual(6), Residual(0), Residual(2)],
    [Residual(3), Residual(5), Residual(7), Residual(1)],
], [
    [Residual(7), Residual(1), Residual(3), Residual(5)],
    [Residual(0), Residual(2), Residual(4), Residual(6)],
]];

// ----------------------------------------------------------------------------

/// Represents a `ShiftedBCC` as a chain of `arrow()`s.
///
/// Each `arrow() operation roughly halves the `ShiftedBCC`, and produces a
/// [`Residual`]. Eventually the chain reaches a fixed point of `arrow()`.
/// The fixed point may be deduced from the last `Residual`.
///
/// [`arrow()`]: ShiftedBCC::arrow
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct Chain {
    /// The [`Residual`]s, listed from least to most significant.
    pub residuals: Vec<Residual>,
    /// The last `Residual`, which maps the fixed point to itself.
    /// It is even more significant.
    pub last_residual: Residual,
}

impl Chain {
    /// Convert a `ShiftedBCC` to a `Self`.
    pub fn from_bcc(mut bcc: ShiftedBCC) -> Self {
        let mut residuals = Vec::new();
        loop {
            let (half, last_residual) = bcc.arrow();
            if bcc == half { return Self {residuals, last_residual}; }
            residuals.push(last_residual);
            bcc = half;
        }
    }

    /// Convert wavelet coefficients to a `Self`.
    pub fn quantize(v: f32, h: f32, c: f32) -> Self {
        Self::from_bcc(ShiftedBCC::quantize(v, h, c).0)
    }

    /// Convert self to wavelet coefficients.
    pub fn vhc(&self) -> (f32, f32, f32) {
        let (mut v, mut h, mut c) = self.last_residual.fixed_point();
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

    /// Applies `symmetry` to `self`. This is its own inverse.
    pub fn apply_symmetry(&self, s: Symmetry) -> Self {
        Self {
            residuals: self.residuals.iter().map(|&r| r.apply_symmetry(s)).collect(),
            last_residual: self.last_residual.apply_symmetry(s),
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_POINTS: [(f32, f32, f32); 8] = [
        ( 0.0, -1.0,  1.5),
        ( 0.0,  1.0,  1.5),
        (-1.0,  0.0,  0.5),
        ( 1.0,  0.0,  0.5),
        ( 0.0, -1.0, -0.5),
        ( 0.0,  1.0, -0.5),
        (-1.0,  0.0, -1.5),
        ( 1.0,  0.0, -1.5),
    ];

    /// Generate a list of 250 `ShiftedBCC` values.
    fn some_bccs() -> Box<[ShiftedBCC]> {
        const RANGE: [f32; 5] = [-4.0, -2.0, 0.0, 2.0, 4.0];
        let mut ret = Vec::new();
        for &v in &RANGE {
            for &h in &RANGE {
                for &c in &RANGE {
                    ret.push(ShiftedBCC::new(v + 1.0, h, c + 0.5));
                    ret.push(ShiftedBCC::new(v, h - 1.0, c - 0.5));
                }
            }
        }
        ret.into()
    }

    #[test]
    fn round_trip() {
        for &(v, h, c) in &FIXED_POINTS {
            let bcc = ShiftedBCC::new(v, h, c);
            assert_eq!(bcc.vhc(), (v, h, c));
        }
    }

    #[test]
    fn arrow() {
        for a in some_bccs().into_iter() {
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
        }
    }

    #[test]
    fn symmetries() {
        // Test `Symmetry(1)`.
        for &bcc in &ALL_RESIDUALS {
            let (v, h, c) = bcc.vhc();
            let sbcc = bcc.apply_symmetry(Symmetry(1));
            let (sv, sh, sc) = sbcc.vhc();
            assert_eq!(v, sh);
            assert_eq!(h, sv);
            assert_eq!(c, -sc);
        }
        // Test `Symmetry(2)`.
        for &bcc in &ALL_RESIDUALS {
            let (v, h, c) = bcc.vhc();
            let sbcc = bcc.apply_symmetry(Symmetry(2));
            let (sv, sh, sc) = sbcc.vhc();
            assert_eq!(v, -sh);
            assert_eq!(h, -sv);
            assert_eq!(c, -sc);
        }
        // Test recommended_symmetry().
        for &bcc in &ALL_RESIDUALS {
            let s = bcc.recommend_symmetry();
            let sbcc = bcc.apply_symmetry(s);
            assert_eq!(sbcc.0 & 3, 0);
        }
        // Test self-inverse property.
        for &s in &ALL_SYMMETRIES {
            for &bcc in &ALL_RESIDUALS {
                let sbcc = bcc.apply_symmetry(s);
                let ssbcc = sbcc.apply_symmetry(s);
                assert_eq!(bcc, ssbcc);
            }
        }
    }

    #[test]
    fn short_chain() {
        for &(v, h, c) in &FIXED_POINTS {
            let bcc = ShiftedBCC::new(v, h, c);
            let chain = Chain::from_bcc(bcc);
            assert_eq!(chain.residuals, []);
            assert_eq!(chain.vhc(), (v, h, c));
            let (bcc2, residual) = bcc.arrow();
            assert_eq!(bcc, bcc2);
            assert_eq!(residual, chain.last_residual);
        }
    }

    #[test]
    fn long_chain() {
        for &bcc in some_bccs().iter() {
            let chain = Chain::from_bcc(bcc);
            let new_bcc = chain.to_bcc();
            assert_eq!(bcc, new_bcc);
        }
    }

    #[test]
    fn chain_symmetries() {
        for &bcc in some_bccs().iter() {
            let chain = Chain::from_bcc(bcc);
            for &s in &ALL_SYMMETRIES {
                let schain = chain.apply_symmetry(s);
                let sschain = schain.apply_symmetry(s);
                assert_eq!(chain, sschain);
            }
        }
    }
}
