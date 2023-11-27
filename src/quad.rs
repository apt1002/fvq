use multidimension::{Index, View, ViewRef, ViewMut, impl_ops_for_view, impl_ops_for_memoryview};

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

impl<T: Clone> View for Quad<T> {
    type I = Small;
    type T = T;
    fn size(&self) -> <Self::I as Index>::Size { ((), ()) }
    fn at(&self, index: Self::I) -> Self::T { self[index].clone() }
}

impl<T: Clone> ViewRef for Quad<T> {
    fn at_ref(&self, index: Self::I) -> &Self::T {
        &self.0[index.0 as usize][index.1 as usize]
    }
}

impl<T: Clone> ViewMut for Quad<T> {
    fn at_mut(&mut self, index: Self::I) -> &mut Self::T {
        &mut self.0[index.0 as usize][index.1 as usize]
    }
}

impl_ops_for_view!(Quad<T: Clone>);
impl_ops_for_memoryview!(Quad<T: Clone>);

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
    /// Constructs a non-blank `Tree`.
    pub fn branch(payload: B, children: Quad<Self>) -> Self {
        Tree::Branch(Box::new(Branch {payload, children}))
    }
}

// ----------------------------------------------------------------------------

/// `path.0` must be less than this.
const LIMIT: u32 = 0x55555555;

/// Padding in highest bit position.
const TOP_BIT: u32 = LIMIT ^ (LIMIT >> 2);

/// Represents an empty `Path`.
const EMPTY: u32 = LIMIT - 1;

/// Represents a path down a `Tree` of length up to `15`.
///
/// `Path` behaves like a stack of [`Small`]s. Construct it using `default()`
/// and `push()` and destruct it using `pop()`. The first `pop()`ped item
/// selects a child of the root of the `Tree`; the second selects a child of
/// that; and so on.
///
/// `Path` implements [`Index`]. Its [`Size`] is an exclusive bound on
/// [`Path::len()`]. To illustrate:
/// - Size 0 admits no `Path`s at all.
/// - Size 1 admits only the root of the `Tree`.
/// - Size 2 admits the root and its immediate children.
/// - Size 3 admits the root, its children and its grandchildren.
/// - And so on.
// Internal representation is a string of 2-bit values in little-endian order
// followed by `00` then many copies of `01`.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Path(u32);

impl Path {
    /// Remove and return one [`Small`], if there is one.
    pub fn pop(&mut self) -> Option<Small> {
        if self.0 >= EMPTY { return None; }
        let bit0 = (self.0 & 1) != 0;
        let bit1 = (self.0 & 2) != 0;
        self.0 >>= 2;
        self.0 |= TOP_BIT;
        Some((bit0, bit1))
    }

    /// Append one [`Small`]. Panics if the `Path` is already full.
    pub fn push(&mut self, small: Small) {
        assert!(self.0 >= TOP_BIT, "Overflow");
        self.0 <<= 2;
        self.0 |= (small.0 as u32) << 0;
        self.0 |= (small.1 as u32) << 1;
    }

    /// The number of [`Small`]s that can be [`pop()`]ped.
    pub fn len(self) -> usize {
        (31 - (self.0 ^ LIMIT).leading_zeros() as usize) / 2
    }

    pub fn iter(self) -> PathIterator { PathIterator(self) }
}

impl Default for Path {
    fn default() -> Self { Path(EMPTY) }
}

impl Index for Path {
    type Size = usize;
    fn length(size: Self::Size) -> usize { (LIMIT - (LIMIT << (2 * size))) as usize }
    fn to_usize(self, _: Self::Size) -> usize { (EMPTY - self.0) as usize }

    fn from_usize(size: Self::Size, index: usize) -> (usize, Self) {
        let length = Self::length(size);
        let q = index / length;
        let r = index - q * length;
        (q, Self(EMPTY - r as u32))
    }

    fn each(size: Self::Size, mut f: impl FnMut(Self)) {
        for p in 0..(Self::length(size) as u32) { f(Self(EMPTY - p)); }
    }
}

// ----------------------------------------------------------------------------

/// The return type of `Path::iter()`.
#[derive(Debug, Copy, Clone)]
pub struct PathIterator(Path);

impl Iterator for PathIterator {
    type Item = Small;
    fn next(&mut self) -> Option<Self::Item> { self.0.pop() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.0.len();
        (length, Some(length))
    }
}

// ----------------------------------------------------------------------------

/// Represents the top `N` levels of a [`Tree`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TreeTop<const N: usize, B> {
    /// The unerlying [`Tree`].
    pub tree: Tree<B>,

    /// `true` to reflect the `Tree` left-to-right.
    pub h_flip: bool,

    /// `true` to reflect the `Tree` top-to-bottom.
    pub v_flip: bool,
}

impl<const N: usize, B> TreeTop<N, B> {
    fn flip(&self, small: Small) -> Small {
        (small.0 ^ self.v_flip, small.1 ^ self.h_flip)
    }
}

impl<const N: usize, B: Clone> View for TreeTop<N, B> {
    type I = Path;
    type T = Option<B>;
    fn size(&self) -> <Self::I as Index>::Size { N + 1 }

    fn at(&self, index: Self::I) -> Self::T {
        let mut t = &self.tree;
        let mut index = index;
        while let Tree::Branch(branch) = t {
            if let Some(small) = index.pop() {
                t = &branch.children[self.flip(small)];
            } else {
                return Some(branch.payload.clone())
            }
        }
        None
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        Path::each(2, |p| {
            let l = p.len();
            assert!(l < 2);
            let mut smalls: Vec<Small> = p.iter().collect();
            assert_eq!(smalls.len(), l);
            let mut q = Path::default();
            while let Some(small) = smalls.pop() { q.push(small); }
            assert_eq!(p, q);
        });
    }
}
