use multidimension::{View, NewView, Array};

use super::{Tile, Tree, VHC};
use super::transform::{Haar};

mod bcc;
pub use bcc::{ShiftedBCC, Symmetry, ALL_SYMMETRIES, Residual, ALL_RESIDUALS, Chain};

// ----------------------------------------------------------------------------

fn tolerance(linear: f32) -> f32 {
    let luma = if linear < 0.001 { 0.001 } else { linear };
//    1.0 / 6.0
//    luma / (4.0 * luma.sqrt())
    luma / (3.0 * luma.cbrt())
//    luma / (2.0 * luma.sqrt().sqrt())
//    luma / 2.0
}

/// The recursive part of `to_digital()`.
///
/// Returns the digital [`Tree`], the L2 norm of the quantisation error (i.e.
/// after dividing by sensitivity), and the L2 norm of `tree` (before dividing
/// by sensitivity).
fn to_digital_inner(low: f32, tree: &Tree<Array<VHC, f32>>, gain: f32) -> (Tree<ShiftedBCC>, f32, f32) {
    match tree {
        Tree::Branch(branch) => {
            let tolerance = tolerance(low * gain);
            let sensitivity = tolerance.recip();
            let v = branch.payload.at(VHC::Vertical);
            let h = branch.payload.at(VHC::Horizontal);
            let c = branch.payload.at(VHC::Cross);
            let mut leaf_norm = v * v + h * h + c * c;
            let (bcc, mut branch_error_norm) = ShiftedBCC::quantize(
                sensitivity * v,
                sensitivity * h,
                sensitivity * c,
            );
            let new_v = tolerance * bcc.v();
            let new_h = tolerance * bcc.h();
            let new_c = tolerance * bcc.c();
            let haar = Haar::new(low, new_v, new_h, new_c).transform();
            let children = Tile::new_view(((), ()), |buffer| {
                haar.zip(branch.children.as_ref()).each(|(child_low, child)| {
                    let (child, child_error_norm, child_leaf_norm) = to_digital_inner(child_low, child, gain * 2.0);
                    branch_error_norm += child_error_norm;
                    leaf_norm += child_leaf_norm;
                    buffer.push(child);
                });
            });
            let leaf_error_norm = leaf_norm * (sensitivity * sensitivity);
            if leaf_error_norm < branch_error_norm {
                // Quantise it to a leaf.
                (Tree::Leaf, leaf_error_norm, leaf_norm)
            } else {
                // Quantise it to a branch.
                (Tree::branch(bcc, children), branch_error_norm, leaf_norm)
            }
        },
        Tree::Leaf => (Tree::Leaf, 0.0, 0.0),
    }
}

/// Convert an image tile from analogue to digitial form, after using a
/// perceptual model to divide every value by the smallest visible difference.
/// Blank subtrees are replaced with leaves.
///
/// - order - the number of generations of wavelets.
/// - low - the low-frequency wavelet component of the tile.
/// - tree - all other wavelet components of the tile.
pub fn to_digital(order: usize, low: f32, tree: &Tree<Array<VHC, f32>>) -> Tree<ShiftedBCC> {
    to_digital_inner(low, tree, 0.5_f32.powi(order as i32)).0
}

/// The recursive part of `from_digital()`.
pub fn from_digital_inner(low: f32, tree: &Tree<ShiftedBCC>, gain: f32) -> Tree<Array<VHC, f32>> {
    match tree {
        Tree::Branch(branch) => {
            let tolerance = tolerance(low * gain);
            let v = tolerance * branch.payload.v();
            let h = tolerance * branch.payload.h();
            let c = tolerance * branch.payload.c();
            let haar = Haar::new(low, v, h, c).transform();
            let children = haar.zip(branch.children.as_ref()).map(
                |(child_low, child)| from_digital_inner(child_low, child, gain * 2.0)
            ).collect();
            Tree::branch(Array::new((), [v, h, c]), children)
        },
        Tree::Leaf => Tree::Leaf,
    }
}

/// Convert an image tile from digital to analogue form, then use a perceptual
/// model to multiply every value by the smallest visible difference.
///
/// - order - the number of generations of wavelets.
/// - low - the low-frequency wavelet component of the tile.
/// - tree - all other wavelet components of the tile.
pub fn from_digital(order: usize, low: f32, tree: &Tree<ShiftedBCC>) -> Tree<Array<VHC, f32>> {
    from_digital_inner(low, tree, 0.5_f32.powi(order as i32))
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let low = 0.5;
        let digital = Tree::branch(
            ShiftedBCC::new(2.0, -1.0, -0.5),
            Tile::new(Tree::Leaf, Tree::Leaf, Tree::Leaf, Tree::branch(
                ShiftedBCC::new(1.0, -2.0, 0.5),
                Tile::new(Tree::Leaf, Tree::Leaf, Tree::Leaf, Tree::Leaf),
            )),
        );
        let analogue = from_digital(2, low, &digital);
        let digital2 = to_digital(2, low, &analogue);
        assert_eq!(digital, digital2);
    }
}
