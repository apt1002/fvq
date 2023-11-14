use multidimension::{Index, View, Scalar, Array};
use super::{Grid, Small, Tile, Tree};

mod haar;
pub use haar::{Haar, to_haar, from_haar};

mod twiddle;
pub use twiddle::{twiddle, twiddle_grid};

mod vhc;
pub use vhc::{VHC, to_low, to_high, from_low_high};

// ----------------------------------------------------------------------------

/// Groups items into [`Small`] tiles.
pub fn group<'a, T: Clone>(
    v: impl 'a + View<I=Grid, T=T>,
) -> impl 'a + View<I=(Grid, Small), T=T> {
    let v = v.from_usize::<(), (usize, bool), usize>(|height| (height / 2, ()));
    let v = v.from_usize::<(usize, bool), (usize, bool), ()>(|width| (width / 2, ()));
    let v = v.transpose::<usize, usize, bool, bool>();
    v.iso()
}

/// Ungroups [`Small`] tiles of items.
pub fn ungroup<'a, T: Clone>(
    v: impl 'a + View<I=(Grid, Small), T=T>
) -> impl 'a + View<I=Grid, T=T> {
    let v = v.transpose::<usize, bool, usize, bool>();
    let v = v.to_usize::<(usize, bool), (usize, bool), ()>();
    let v = v.to_usize::<(), (usize, bool), usize>();
    v.iso()
}

// ----------------------------------------------------------------------------

/// Selects a tile of a `Pyramid`.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    /// The level within the `Pyramid`.
    pub level: usize,

    /// The position within the `Pyramid`, measured in units of
    /// `1 << (order - level)` pixels.
    pub yx: Grid,
}

impl Position {
    fn children(self) -> impl View<I=Small, T=Self> {
        <(bool, bool)>::all(((), ())).map(move |bb| Position {
            level: self.level + 1,
            yx: (2 * self.yx.0 + bb.0 as usize, 2 * self.yx.1 + bb.1 as usize),
        })
    }
}

// ----------------------------------------------------------------------------

/// Represents a pyramid of wavelet coefficients.
pub struct Pyramid {
    pub low: Array<Grid, f32>,
    pub highs: Box<[Array<(Grid, VHC), f32>]>,
}

impl Pyramid {
    /// Transform `pixels` into a `Pyramid`.
    ///
    /// `pixels.size()` must be a multiple of `1 << order` in each dimension.
    pub fn from_pixels(order: usize, is_smooth: bool, pixels: Array<Grid, f32>) -> Self {
        let mut low = pixels;
        let mut highs = Vec::new();
        for _ in 0..order {
            let mut haar = to_haar(low);
            if is_smooth { haar = twiddle_grid::<false>(haar); }
            highs.push(to_high(&haar));
            low = to_low(&haar);
        }
        Self {low, highs: highs.into_iter().rev().collect()}
    }

    pub fn to_pixels(self, is_smooth: bool) -> Array<Grid, f32> {
        let mut low = self.low;
        let mut highs = self.highs.into_vec().into_iter().rev().collect::<Vec<Array<_, _>>>();
        while let Some(high) = highs.pop() {
            let mut haar = from_low_high(low, high);
            if is_smooth { haar = twiddle_grid::<true>(haar); }
            low = from_haar(haar).collect();
        }
        low
    }

    pub fn montage(self) -> Array<Grid, f32> {
        let mut low = self.low;
        let mut highs = self.highs.into_vec().into_iter().rev().collect::<Vec<Array<_, _>>>();
        while let Some(high) = highs.pop() {
            low = from_low_high(low, high + Scalar(0.5)).nested()
                .transpose::<(), Small, Grid, ()>()
                .transpose::<bool, usize, bool, usize>()
                .to_usize::<(bool, usize), (bool, usize), ()>()
                .to_usize::<(), (bool, usize), usize>()
                .iso().collect();
        }
        low
    }

    /// Returns the order of this `Pyramid`.
    pub fn order(&self) -> usize { self.highs.len() }

    /// Returns the size of this `Pyramid` in units of `1 << order()`.
    pub fn size(&self) -> <Grid as Index>::Size { self.low.size() }

    /// Reads the tile at `pos` into a `Tree`.
    pub fn get(&self, pos: Position) -> Tree<Array<VHC, f32>> {
        if pos.level < self.order() {
            Tree::branch(
                (&self.highs[pos.level]).row(pos.yx).collect(),
                pos.children().map(|child_pos| self.get(child_pos)).collect(),
            )
        } else {
            Tree::Leaf
        }
    }

    /// Write the tile at position `yx` from a `Tree`.
    ///
    /// `yx` is measured in units of `1 << order()` pixels.
    pub fn set(&mut self, pos: Position, tree: &Tree<Array<VHC, f32>>) {
        if pos.level < self.order() {
            match tree {
                Tree::Branch(branch) => {
                    VHC::each((), |w| {
                        self[(pos, w)] = branch.payload.at(w);
                    });
                    pos.children().zip(branch.children.as_ref()).each(|(child_pos, child)| {
                        self.set(child_pos, child);
                    });
                },
                Tree::Leaf => {
                    VHC::each((), |w| {
                        self[(pos, w)] = 0.0;
                    });
                    pos.children().each(|child_pos| {
                        self.set(child_pos, &Tree::Leaf);
                    });
                },
            }
        }
    }
}

impl std::ops::Index<Grid> for Pyramid {
    type Output = f32;
    fn index(&self, index: Grid) -> &Self::Output { &self.low[index] }
}

impl std::ops::IndexMut<Grid> for Pyramid {
    fn index_mut(&mut self, index: Grid) -> &mut Self::Output { &mut self.low[index] }
}

impl std::ops::Index<(Position, VHC)> for Pyramid {
    type Output = f32;
    fn index(&self, index: (Position, VHC)) -> &Self::Output {
        let (pos, vhc) = index;
        &self.highs[pos.level][(pos.yx, vhc)]
    }
}

impl std::ops::IndexMut<(Position, VHC)> for Pyramid {
    fn index_mut(&mut self, index: (Position, VHC)) -> &mut Self::Output {
        let (pos, vhc) = index;
        &mut self.highs[pos.level][(pos.yx, vhc)]
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use multidimension::{Index, Array};

    use super::*;

    #[test]
    #[should_panic]
    fn odd_height() {
        group(Array::new((1, 2), ["a", "b"]));
    }

    #[test]
    #[should_panic]
    fn odd_width() {
        group(Array::new((2, 1), ["a", "b"]));
    }

    #[test]
    fn group_ungroup() {
        let a: Array<_, _> = <(usize, usize)>::all((4, 6)).collect();
        let g = group(&a);
        let u = ungroup(&g);
        (&a).zip(u).each(|(x, y)| { assert_eq!(x, y); });
    }

    #[test]
    fn round_trip() {
        let a: Array<_, _> = <(usize, usize)>::all((8, 16)).map(
            |(y, x)| 0.125 * (x * (15-x)) as f32 - 0.25 * (y * (7-y)) as f32
        ).collect();
        let p = Pyramid::from_pixels(2, true, a.clone());
        let b = p.to_pixels(true);
        a.zip(b).each(|(x, y)| { assert!((x - y).abs() < 1e-5); });
    }
}
