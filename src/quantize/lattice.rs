use std::fmt::{Debug};
use std::ops::{Add, Sub, Mul, Neg};

pub trait Vector:
    Debug + Clone + PartialEq +
    Add<Self, Output=Self> +
    Sub<Self, Output=Self> +
    Neg<Output=Self>
{
    /// The additive identity.
    const ZERO: Self;

    /// Multiply by an integer.
    fn scale(self, other: isize) -> Self;

    /// Dot product.
    fn dot(self, other: Self) -> f32;

    /// Dot product with `self`.
    fn norm(self) -> f32 { self.clone().dot(self) }
}

impl Vector for f32 {
    const ZERO: Self = 0.0;
    fn scale(self, other: isize) -> Self { self * (other as f32)}
    fn dot(self, other: Self) -> f32 { self * other }
    fn norm(self) -> f32 { self * self }
}

impl Vector for i16 {
    const ZERO: Self = 0;
    fn scale(self, other: isize) -> Self { self * (other as i16)}
    fn dot(self, other: Self) -> f32 { self as f32 * other as f32}
    fn norm(self) -> f32 { self as f32 * self as f32 }
}

// ----------------------------------------------------------------------------

/// Represents a [`Vector`] with `N` elements of type `T`.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Point<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> IntoIterator for Point<T, N> {
    type Item = T;
    type IntoIter = <[T; N] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<T, const N: usize> FromIterator<T> for Point<T, N> {
    fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item=T> {
        Point(Vec::from_iter(iter).try_into().unwrap_or_else(
            |v: Vec<T>| panic!("Expected {} elements but got {}", N, v.len())
        ))
    }
}

impl<T, U, const N: usize> Add<Point<U, N>> for Point<T, N> where T: Add<U> {
    type Output = Point<<T as Add<U>>::Output, N>;

    fn add(self, other: Point<U, N>) -> Self::Output {
        self.into_iter().zip(other.into_iter()).map(|(x, y)| x + y).collect()
    }
}

impl<T, U, const N: usize> Sub<Point<U, N>> for Point<T, N> where T: Sub<U> {
    type Output = Point<<T as Sub<U>>::Output, N>;

    fn sub(self, other: Point<U, N>) -> Self::Output {
        self.into_iter().zip(other.into_iter()).map(|(x, y)| x - y).collect()
    }
}

impl<T, U, const N: usize> Mul<Point<U, N>> for Point<T, N> where T: Mul<U> {
    type Output = Point<<T as Mul<U>>::Output, N>;

    fn mul(self, other: Point<U, N>) -> Self::Output {
        self.into_iter().zip(other.into_iter()).map(|(x, y)| x * y).collect()
    }
}

impl<T, const N: usize> Neg for Point<T, N> where T: Neg {
    type Output = Point<<T as Neg>::Output, N>;

    fn neg(self) -> Self::Output {
        self.into_iter().map(|x| -x).collect()
    }
}

impl<T, const N: usize> Vector for Point<T, N> where T: Vector {
    const ZERO: Self = Point([T::ZERO; N]);

    fn scale(self, other: isize) -> Self {
        self.into_iter().map(|x| x.scale(other)).collect()
    }

    fn dot(self, other: Self) -> f32 {
        self.into_iter().zip(other.into_iter()).map(|(x, y)| x.dot(y)).sum()
    }

    fn norm(self) -> f32 {
        self.into_iter().map(|x| x.norm()).sum()
    }
}

// ----------------------------------------------------------------------------

/// An `N`-dimensional quantisation lattice.
pub trait Lattice<const N: usize>: Vector {
    /// Round `data` to the nearest `Self`.
    /// Also return the [`norm()`] of the quantisation error.
    ///
    /// [`norm()`]: Vector::norm()
    fn to_digital(data: [f32; N]) -> Self;

    /// Compute the coordinates of `Self`.
    fn to_analogue(data: Self) -> [f32; N];
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_f32() {
        let a = Point([3.0, 4.0]);
        let b = Point([-1.0, 1.0]);
        assert_eq!(a.scale(2), Point([6.0, 8.0]));
        assert_eq!(a.dot(b), 1.0);
        assert_eq!(a.norm(), 25.0);
    }
}
