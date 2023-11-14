use multidimension::{Index, View, impl_ops_for_view};

// TODO: Use `View::to_usize()` and `View::from_usize()` instead.

/// A `View` type that maps `usize` to `(usize, I)`.
#[derive(Debug, Copy, Clone)]
pub struct FromUsize<I: Index>(pub usize, pub <I as Index>::Size);

impl<I: Index> View for FromUsize<I> {
    type I = usize;
    type T = (usize, I);
    fn size(&self) -> <Self::I as Index>::Size { self.0 * I::length(self.1) }
    fn at(&self, index: Self::I) -> Self::T { I::from_usize(self.1, index) }
}

impl_ops_for_view!(FromUsize<I: Index>);

// ----------------------------------------------------------------------------

/// A `View` type that maps `(usize, I)` to `usize`.
#[derive(Debug, Copy, Clone)]
pub struct ToUsize<I: Index>(pub usize, pub <I as Index>::Size);

impl<I: Index> View for ToUsize<I> {
    type I = (usize, I);
    type T = usize;
    fn size(&self) -> <Self::I as Index>::Size { (self.0, self.1) }

    fn at(&self, index: Self::I) -> Self::T {
        index.0 * I::length(self.1) + index.1.to_usize(self.1)
    }
}

impl_ops_for_view!(ToUsize<I: Index>);
