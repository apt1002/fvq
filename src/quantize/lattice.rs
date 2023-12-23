use std::ops::{Add, Sub, Neg};
use num_traits::{Zero, ToPrimitive};
use num_traits::real::{Real};
use vector_space::{VectorSpace, InnerSpace};
use simple_vectors::{Vector};

/// An integral lattice.
///
/// This is an integral lattice; i.e. the dot product of any two lattice
/// vectors must be an integer.
pub trait Lattice<const N: usize>: Copy + Zero + PartialEq where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Neg<Output = Self>,
{
    /// The analogue vector space that this `Lattice` approximates.
    type V: InnerSpace;

    /// Rounds `analogue` to the nearest `Self`.
    /// Also returns the [`norm`] of the quantisation error, which is
    /// `analogue - Self::to_digital(analogue).to_analogue()`.
    ///
    /// [`norm`]: InnerSpace::magnitude2()
    fn to_digital(analogue: Self::V) -> (Self, <Self::V as VectorSpace>::Scalar);

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

impl Lattice<1> for i32 {
    type V = f32;

    fn to_digital(analogue: Self::V) -> (Self, f32) {
        let ret = analogue.round().to_i32().expect("Overflow");
        let error = analogue - ret.to_analogue();
        (ret, error.magnitude2())
    }

    fn to_analogue(self) -> Self::V { self.to_f32().unwrap() }
}

impl<const N: usize> Lattice<N> for Vector<i32, N> {
    type V = Vector<f32, N>;

    fn to_digital(_analogue: Self::V) -> (Self, f32) {
        unimplemented!();
    }

    fn to_analogue(self) -> Self::V {
        unimplemented!();
    }
}
