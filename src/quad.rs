use multidimension::{Index, View, impl_ops_for_view};

use super::{Small};

/// A 2x2 grid of `T`s
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Quad<T>(pub [[T; 2]; 2]);

impl<T> Quad<T> {
    pub fn new(a: T, b: T, c: T, d: T) -> Self {
        Quad([[a, b], [c, d]])
    }
    /// Exchanges the indices.
    pub fn transpose(self) -> Self {
        let [[a, b], [c, d]] = self.0;
        Self::new(a, c, b, d)
    }

    /// Borrows the four `T`s, applies `f` to each one, and wraps the results
    /// in a `Quad`.
    pub fn as_ref(&self) -> Quad<&T> {
        let [[ref a, ref b], [ref c, ref d]] = self.0;
        Quad::new(a, b, c, d)
    }
}

impl<T> std::ops::Index<Small> for Quad<T> {
    type Output = T;
    fn index(&self, index: Small) -> &Self::Output {
        &self.0[index.0 as usize][index.1 as usize]
    }
}

impl<T> std::ops::IndexMut<Small> for Quad<T> {
    fn index_mut(&mut self, index: Small) -> &mut Self::Output {
        &mut self.0[index.0 as usize][index.1 as usize]
    }
}

impl<T: Clone> View for Quad<T> {
    type I = Small;
    type T = T;
    fn size(&self) -> <Self::I as Index>::Size { ((), ()) }
    fn at(&self, index: Self::I) -> Self::T { self[index].clone() }
}

impl_ops_for_view!(Quad<T: Clone>);

// ----------------------------------------------------------------------------

impl<T: Clone> multidimension::NewView for Quad<T> {
    type Buffer = Vec<T>;
    fn new_view(_size: ((), ()), callback: impl FnOnce(&mut Self::Buffer)) -> Self {
        let mut buffer = Vec::new();
        callback(&mut buffer);
        let mut buffer = buffer.into_iter();
        let a = buffer.next().expect("Buffer is under full");
        let b = buffer.next().expect("Buffer is under full");
        let c = buffer.next().expect("Buffer is under full");
        let d = buffer.next().expect("Buffer is under full");
        assert!(buffer.next().is_none(), "Buffer is over full");
        Quad::new(a, b, c, d)
    }
}

// ----------------------------------------------------------------------------

/// A `Tree` that is not blank.
///
/// A value of type `B` is attached to every `Branch`.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Branch<B> {
    /// The coefficients of the largest wavelets, indexed by [`VHC`].
    pub payload: B,

    /// A 2×2 grid of half-size tiles.
    pub children: Quad<Tree<B>>,
}

// ----------------------------------------------------------------------------

/// Represents a square tile of an image, minus its mean value. The size  of
/// the tile in pixels is a power of two. `None` represents a completely blank
/// tile, everywhere equal to its mean value.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Tree<B> {
    Branch(Box<Branch<B>>),
    Leaf,
}

impl<B> Default for Tree<B> {
    fn default() -> Self { Self::Leaf }
}

impl<B> Tree<B> {
    pub const EMPTY: Self = Tree::Leaf;

    /// Constructs a non-blank `Tree`.
    pub fn branch(payload: B, children: Quad<Self>) -> Self {
        Tree::Branch(Box::new(Branch {payload, children}))
    }
}
