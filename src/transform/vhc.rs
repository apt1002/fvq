use multidimension::{NonTuple, StaticIndex, Index, View, Array};

use super::{Grid, Small, Haar};

/// Identifies a high-frequency wavelet component.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum VHC {
    /// A wavelet component that is anti-correlated horizontally and correlated
    /// vertically.
    Vertical = 0,

    /// A wavelet component that is correlated horizontally and anti-correlated
    /// vertically.
    Horizontal = 1,

    /// A wavelet component that is anti-correlated horizontally and
    /// vertically.
    Cross = 2,
}

impl NonTuple for VHC {}

impl StaticIndex for VHC {
    const ALL: &'static [Self] = &[
        VHC::Vertical,
        VHC::Horizontal,
        VHC::Cross,
    ];

    #[inline(always)]
    fn to_usize(self) -> usize { self as usize }
}

// ----------------------------------------------------------------------------

/// Extract the low-frequency component from a grid of `Haar`.
pub fn to_low(pixels: impl View<I=Grid, T=Haar>) -> Array<Grid, f32> {
    pixels.map(|haar| haar.at((false, false))).collect()
}

/// Extract the high-frequency components from a grid of `Haar`.
pub fn to_high(pixels: impl View<I=Grid, T=Haar>) -> Array<(Grid, VHC), f32> {
    let index_map: Array<VHC, Small> = Array::new((), [
        (false, true), (true, false), (true, true),
    ]);
    pixels.map(|haar| (&index_map).compose(haar)).nested_collect(())
}

/// Combine the low- and high-frequency parts to form a grid of `Haar`.
pub fn from_low_high(
    low: impl View<I=Grid, T=f32>,
    high: impl View<I=(Grid, VHC), T=f32>,
) -> Array<Grid, Haar> {
    let (size, ()) = high.size();
    assert_eq!(size, low.size());
    Grid::all(size).map(|yx| {
        let a = low.at(yx);
        let b = high.at((yx, VHC::Vertical));
        let c = high.at((yx, VHC::Horizontal));
        let d = high.at((yx, VHC::Cross));
        Haar::new(a, b, c, d)
    }).collect()
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vhc() {
        VHC::each((), |vhc| {
            let index = <VHC as Index>::to_usize(vhc, ());
            assert_eq!(vhc, <VHC as Index>::from_usize((), index).1);
        });
    }
}
