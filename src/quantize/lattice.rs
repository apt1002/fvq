use std::ops::{Add, Sub, Neg};
use num_traits::{Zero, ToPrimitive};
use vector_space::{InnerSpace};
use simple_vectors::{Vector};

/// Destruct a [`Vector`].
fn vector_to_iter<T, const N: usize>(v: Vector<T, N>) -> impl Iterator<Item=T> {
    let v: [T; N] = v.into();
    v.into_iter()
}

/// Construct a [`Vector`].
fn vector_from_iter<T, const N: usize>(i: impl Iterator<Item = T>) -> Vector<T, N> {
    Vector::new(i.collect::<Vec<_>>().try_into().unwrap_or_else(
        |v: Vec<T>| panic!("Expected {} elements but got {}", N, v.len())
    ))
}

/// Map `f` over the elements of `v`.
fn map_vector<T, U, const N: usize>(
    v: Vector<T, N>,
    mut f: impl FnMut(T) -> U,
) -> Vector<U, N> {
    vector_from_iter(vector_to_iter(v).map(|t| f(t)))
}

// ----------------------------------------------------------------------------

/// An integral lattice. The dot product of any two lattice vectors must be an
/// integer.
///
/// Implementations must override at least one of [`to_digital()`] and
/// [`quantize()`].
pub trait Lattice: Copy + Zero + PartialEq where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Neg<Output = Self>,
{
    /// The analogue vector space that this `Lattice` approximates.
    type V: InnerSpace<Scalar=f32>;

    /// Converts `self` to an analogue vector.
    fn to_analogue(self) -> Self::V;

    /// Rounds `analogue` to the nearest `Self` and adds the square of the
    /// quantization error to `error`.
    fn to_digital(analogue: Self::V, error2: &mut f32) -> Self {
        let (digital, additional_error2) = Self::quantize(analogue);
        *error2 += additional_error2;
        digital
    }

    /// Rounds `analogue` to the nearest `Self` and returns the square of the
    /// quantization error.
    fn quantize(analogue: Self::V) -> (Self, f32) {
        let mut error2 = 0.0;
        let digital = Self::to_digital(analogue, &mut error2);
        (digital, error2)
    }

    /// Returns the scalar product of `self` and `other`, which must be an
    /// integer.
    ///
    /// The default implementation converts `self` and `other` to analogue
    /// vectors and then takes their scalar product.
    fn scalar(self, other: Self) -> u64 {
        self.to_analogue().scalar(other.to_analogue()).round().to_u64().expect("Overflow")
    }

    /// Returns the L2 norm of `self`, which must be an integer.
    fn magnitude2(self) -> u64 { self.clone().scalar(self) }
}

impl Lattice for i32 {
    type V = f32;

    fn to_analogue(self) -> Self::V { self.to_f32().unwrap() }

    fn quantize(analogue: Self::V) -> (Self, f32) {
        let ret = analogue.round().to_i32().expect("Overflow");
        (ret, (analogue - ret.to_analogue()).magnitude2())
    }
}

impl<D: Lattice<V=f32>, const N: usize> Lattice for Vector<D, N> {
    type V = Vector<D::V, N>;

    fn to_analogue(self) -> Self::V { map_vector(self, D::to_analogue) }

    fn to_digital(analogue: Self::V, error2: &mut f32) -> Self {
        map_vector(analogue, |a| D::to_digital(a, error2))
    }
}

// ----------------------------------------------------------------------------

/// A point of the D_N lattice.
///
/// A point belongs to the lattice iff its coordinates are integers with an
/// even sum. This lattice is interesting because [`to_digital()`] is cheap.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct D<const N: usize>(Vector<i32, N>);

impl<const N: usize> D<N> {
    pub fn new(data: [i32; N]) -> Self {
        assert_eq!(data.iter().sum::<i32>() & 1, 0);
        D(Vector::new(data))
    }
}

impl<const N: usize> Add for D<N> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output { D(self.0 + other.0) }
}

impl<const N: usize> Sub for D<N> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output { D(self.0 - other.0) }
}

impl<const N: usize> Neg for D<N> {
    type Output = Self;
    fn neg(self) -> Self::Output { D(-self.0) }
}

impl<const N: usize> Zero for D<N> {
    fn zero() -> Self { D(Zero::zero()) }
    fn is_zero(&self) -> bool { self.0.is_zero() }
}

impl<const N: usize> Lattice for D<N> {
    type V = Vector<f32, N>;

    fn to_analogue(self) -> Self::V { map_vector(self.0, i32::to_analogue) }

    fn to_digital(analogue: Self::V, error2: &mut f32) -> Self {
        let mut best_i = N;
        let mut best_error = f32::INFINITY;
        let mut best_a = f32::NAN;
        let mut is_odd = false;
        let mut digital = vector_from_iter(
            vector_to_iter(analogue).enumerate().map(|(i, a)| {
                let d = i32::to_digital(a, error2);
                is_odd ^= (d & 1) != 0;
                let error = ((a + 0.5).round() - (a + 0.5)).abs();
                if error < best_error {
                    best_i = i;
                    best_error = error;
                    best_a = a;
                }
                d
            })
        );
        if is_odd {
            let d = &mut digital[best_i];
            *d += if best_a > d.to_analogue() { 1 } else { -1 };
            *error2 += 2.0 * best_error;
        }
        D(digital)
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::{Debug};

    fn check<L: Debug + Lattice>(analogue: L::V, digital: L, error2: f32) {
        let (d, e) = L::quantize(analogue);
        assert_eq!(d, digital);
        assert_eq!(e, error2);
    }

    #[test]
    fn d3() {
        check(Vector::new([-2.0, 0.0, 0.0]), D::new([-2, 0, 0]), 0.0);
        check(Vector::new([0.0, 0.75, 0.0]), D::new([0, 0, 0]), 0.5625);
        check(Vector::new([0.0, 1.25, 0.0]), D::new([0, 2, 0]), 0.5625);
        check(Vector::new([0.0, 1.0, 0.25]), D::new([0, 1, 1]), 0.5625);
        check(Vector::new([2.0, 1.0, 1.75]), D::new([2, 1, 1]), 0.5625);
    }
}
