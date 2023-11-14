use std::mem::{transmute};

use multidimension::{NonTuple, Index, StaticIndex, NewView, View, Array};

use super::{Grid};

// ----------------------------------------------------------------------------

/// An `Index` that distinguishes colour channels.
pub trait Channels: StaticIndex {
    /// The number of colour channels.
    const NUM_CHANNELS: usize = Self::ALL.len();

    /// Returns `true` if `self` is the `Alpha` channel.
    ///
    /// The `Alpha` channel is typically not gamma-corrected.
    fn is_alpha(self) -> bool;
}

// ----------------------------------------------------------------------------

/// Indicates the unique channel of a luma-only image.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct L;

impl NonTuple for L {}

impl StaticIndex for L {
    const ALL: &'static [Self] = &[L];
    fn to_usize(self) -> usize { 0 }
    fn from_usize(_: usize) -> Self { L }
}

impl Channels for L {
    fn is_alpha(self) -> bool { false }
}

// ----------------------------------------------------------------------------

/// Indicates a channel of a luma + alpha image.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum LA {Luma=0, Alpha=1}

impl NonTuple for LA {}

impl StaticIndex for LA {
    const ALL: &'static [Self] = &[LA::Luma, LA::Alpha];
    fn to_usize(self) -> usize { self as usize }
    fn from_usize(index: usize) -> Self { unsafe { transmute(index as u8) } }
}

impl Channels for LA {
    fn is_alpha(self) -> bool { self == LA::Alpha }
}

// ----------------------------------------------------------------------------

/// Indicates a channel of a colour image.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum RGB {Red=0, Green=1, Blue=2}

impl NonTuple for RGB {}

impl StaticIndex for RGB {
    const ALL: &'static [Self] = &[RGB::Red, RGB::Green, RGB::Blue];
    fn to_usize(self) -> usize { self as usize }
    fn from_usize(index: usize) -> Self { unsafe { transmute(index as u8) } }
}

impl Channels for RGB {
    fn is_alpha(self) -> bool { false }
}

// ----------------------------------------------------------------------------

/// Indicates a channel of a colour + alpha image.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum RGBA {Red=0, Green=1, Blue=2, Alpha=3}

impl NonTuple for RGBA {}

impl StaticIndex for RGBA {
    const ALL: &'static [Self] = &[RGBA::Red, RGBA::Green, RGBA::Blue, RGBA::Alpha];
    fn to_usize(self) -> usize { self as usize }
    fn from_usize(index: usize) -> Self { unsafe { transmute(index as u8) } }
}

impl Channels for RGBA {
    fn is_alpha(self) -> bool { self == RGBA::Alpha }
}

// ----------------------------------------------------------------------------

/// A rectangular grid of pixels with colour channels indexed by `C`.
pub struct PixelArray<C: Channels>(pub Array<(Grid, C), f32>);

impl<C: Channels> PixelArray<C> {
    /// Removes a border from `self` to make the size a multiple of `quantum`.
    pub fn crop_to_multiple(&self, quantum: usize) -> Self {
        let ((height, width), ()) = self.size();
        let (h_r, w_r) = (height % quantum, width % quantum);
        let new_size = (height - h_r, width - w_r);
        let (top, left) = (h_r / 2, w_r / 2);
        <(Grid, C)>::all((new_size, ())).map(
            |((y, x), c)| ((y + top, x + left), c)
        ).compose(self).collect()
    }
}

impl<C: Channels> View for PixelArray<C> {
    type I = <Array<(Grid, C), f32> as View>::I;
    type T = <Array<(Grid, C), f32> as View>::T;
    fn size(&self) -> <Self::I as Index>::Size { self.0.size() }
    fn at(&self, index: Self::I) -> Self::T { self.0.at(index) }
}

impl<C: Channels> NewView for PixelArray<C> {
    type Buffer = <Array<(Grid, C), f32> as NewView>::Buffer;

    fn new_view(
        size: <Self::I as Index>::Size,
        callback: impl FnOnce(&mut Self::Buffer),
    ) -> Self {
        Self(<Array<(Grid, C), f32> as NewView>::new_view(size, callback))
    }
}

// ----------------------------------------------------------------------------

/// Represents an uncompressed image, at ample precision, in a linear colour
/// space.
pub enum Pixels {
    L(PixelArray<L>),
    LA(PixelArray<LA>),
    RGB(PixelArray<RGB>),
    RGBA(PixelArray<RGBA>),
}
