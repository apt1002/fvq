use std::ops::{Add, Sub, Neg};
use num_traits::{Zero, ToPrimitive};
use vector_space::{InnerSpace};
use simple_vectors::{Vector};

/// Map `f` over the elements of `v`.
fn map_vector<T, U, const N: usize>(
    v: Vector<T, N>,
    mut f: impl FnMut(T) -> U,
) -> Vector<U, N> {
    let v: [T; N] = v.into();
    let v: Vec<U> = Vec::from_iter(v.into_iter().map(|t| f(t)));
    let v: [U; N] = v.try_into().unwrap_or_else(
        |v: Vec<U>| panic!("Expected {} elements but got {}", N, v.len())
    );
    Vector::new(v)
}

// ----------------------------------------------------------------------------

/// An integral lattice.
///
/// This is an integral lattice; i.e. the dot product of any two lattice
/// vectors must be an integer.
pub trait Lattice: Copy + Zero + PartialEq where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Neg<Output = Self>,
{
    /// The analogue vector space that this `Lattice` approximates.
    type V: InnerSpace<Scalar=f32>;

    /// Rounds `analogue` to the nearest `Self`.
    /// Adds to `error` the [`norm`] of the quantisation error, which is
    /// `analogue - Self::to_digital(analogue).to_analogue()`.
    ///
    /// [`norm`]: InnerSpace::magnitude2()
    fn to_digital(analogue: Self::V, error: &mut f32) -> Self;

    /// Converts `self` to an analogue vector.
    fn to_analogue(self) -> Self::V;

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

    fn to_digital(analogue: Self::V, error: &mut f32) -> Self {
        let ret = analogue.round().to_i32().expect("Overflow");
        *error += (analogue - ret.to_analogue()).magnitude2();
        ret
    }

    fn to_analogue(self) -> Self::V { self.to_f32().unwrap() }
}

impl<D: Lattice<V=f32>, const N: usize> Lattice for Vector<D, N> {
    type V = Vector<D::V, N>;

    fn to_digital(analogue: Self::V, error: &mut f32) -> Self {
        map_vector(analogue, |a| D::to_digital(a, error))
    }

    fn to_analogue(self) -> Self::V {
        map_vector(self, D::to_analogue)
    }
}

// ----------------------------------------------------------------------------

