use multidimension::{View, NewView, Array};

use super::{Grid, Small, Tile};

/// A 2x2 grid of `f32`s
#[derive(Debug, Copy, Clone)]
pub struct Haar(pub Tile<f32>);

impl Haar {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Haar(Tile::new(a, b, c, d))
    }

    /// Transforms `v`. The transformation is its own inverse.
    pub fn transform(self) -> Self {
        let a = 0.5 * self[(false, false)];
        let b = 0.5 * self[(false, true)];
        let c = 0.5 * self[(true, false)];
        let d = 0.5 * self[(true, true)];
        Self::new(
            (a + b) + (c + d), (a - b) + (c - d),
            (a + b) - (c + d), (a - b) - (c - d),
        )
    }

    /// Exchanges the indices.
    pub fn transpose(self) -> Self { Haar(self.0.transpose()) }
}

impl std::ops::Deref for Haar {
    type Target = Tile<f32>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for Haar {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl NewView for Haar {
    type Buffer = <Tile<f32> as NewView>::Buffer;
    fn new_view(size: ((), ()), callback: impl FnOnce(&mut Self::Buffer)) -> Self {
        Haar(Tile::<f32>::new_view(size, callback))
    }
}

// ----------------------------------------------------------------------------

pub fn to_haar(pixels: impl View<I=Grid, T=f32>) -> Array<Grid, Haar> {
    let pixels = super::group(pixels);
    let tiles: Array<Grid, Haar> = pixels.rows::<Grid, Small>().map(
        |tile| tile.collect::<Haar>().transform()
    ).collect();
    tiles
}

pub fn from_haar(tiles: Array<Grid, Haar>) -> impl View<I=Grid, T=f32> {
    let tiles = tiles.map(Haar::transform);
    let pixels: Array<(Grid, Small), f32> = tiles.nested_collect(((), ()));
    let pixels = super::ungroup(pixels);
    pixels
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use multidimension::{Array};
    use super::*;

    #[test]
    fn haar() {
        let a: Array<Small, f32> = Array::new((), [1.0, 4.0, 2.0, 3.0]);
        let h: Haar = (&a).collect();
        let htt = h.transform().transform();
        a.zip(htt).each(|(x, y)| { assert_eq!(x, y) });
    }
}
