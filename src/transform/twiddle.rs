use multidimension::{View, Array};

use super::{Grid, Haar};

//----------------------------------------------------------------------

/// Applies an orthonormal decorrelating transform.
///
/// The transform is equivalent to the following algorithm:
/// - For `b` in `[true, false]`:
///   - Let `low[i]` mean `hs[i][b][false]`
///   - Let `high[i]` mean `hs[i][b][true]`.
///   - For each even `i`, swap `low[i]` with `high[i]`.
///   - Let `ring` be the cyclic concatenation of `low` with the reverse of
///     `high`.
///   - For each even `i`, mix `ring[i]` with `ring[i+1]`.
///   - For each even `i`, mix `ring[i]` with `ring[i-1]`.
///   - Undo the cyclic concatenation.
///   - Undo the swaps.
///
/// In the above, "mix x with y" means rotate the vector `(x, y)` by
/// `atan(1/8)`.
///
/// - IS_INVERSE - `true` for the inverse transform.
pub fn twiddle<const IS_INVERSE: bool>(hs: &mut[Haar]) {
    let n = hs.len();
    // a = 1.0 / 16.0
    let cos = 0.9980475107000991; // cos(a)
    let sin = 0.0624593178423802; // sin(a)
    let sin = if IS_INVERSE { -sin } else { sin };
    let mut rotate = |x: usize, y: usize, is_x_high: bool| {
        for b in [false, true] {
            let old_x = hs[x][(b, is_x_high)];
            let old_y = hs[y][(b, !is_x_high)];
            hs[x][(b, is_x_high)] = cos * old_x + sin * old_y;
            hs[y][(b, !is_x_high)] = cos * old_y - sin * old_x;
        }
    };
    for start in [0, 1, 1, 0] {
        let mut i = start;
        if i == 0 {
            rotate(i, i, false);
            i += 2;
        }
        while i < n {
            rotate(i-1, i, false);
            rotate(i-1, i, true);
            i += 2;
        }
        if i == n {
            rotate(i-1, i-1, true);
        }
    }
}

fn twiddle_columns<const IS_INVERSE: bool>(quads: Array<Grid, Haar>) -> Array<Grid, Haar> {
    let (height, _) = quads.size();
    quads.columns::<usize, usize>().map(|column| {
        let mut column: Array<usize, Haar> = column.map(Haar::transpose).collect();
        twiddle::<IS_INVERSE>(column.as_mut());
        column
    }).nested_collect(height)
}

pub fn twiddle_grid<const IS_INVERSE: bool>(quads: Array<Grid, Haar>) -> Array<Grid, Haar> {
    let quads = twiddle_columns::<IS_INVERSE>(quads);
    let quads = twiddle_columns::<IS_INVERSE>(quads);
    quads
}

//----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut hs: [Haar; 3] = [
            Haar::new(1.25, 1.0, 2.5, 5.75),
            Haar::new(9.25, 3.0, 4.5, 4.75),
            Haar::new(25.25, 5.0, 8.5, 1.75),
        ];
        let old_hs = hs.clone();
        twiddle::<false>(&mut hs);
        println!("{:#?}", hs);
        twiddle::<true>(&mut hs);
        println!("{:#?}", hs);
        for i in 0..3 {
            for bb in [(false, false), (false, true), (true, false), (true, true)] {
                assert!((hs[i][bb] - old_hs[i][bb]).abs() < 0.00001);
            };
        }
    }

    #[test]
    fn ramp() {
        let mut hs: [Haar; 8] = (0..8).map(|x| {
            let x = x as f32 * 2.0;
            Haar::new(x, x + 1.0, x - 15.0, x - 14.0)
        }).map(Haar::transform).collect::<Vec<_>>().try_into().unwrap();
        println!("{:#?}", hs);
        twiddle::<false>(&mut hs);
        println!("{:#?}", hs);
        for x in 3..5 {
            let h = &hs[x];
            let x = x as f32 * 4.0;
            assert!((x - 14.0 - h[(false, false)]).abs() < 0.02);
            assert!(h[(false, true)].abs() < 0.02);
            assert!((15.0 - h[(true, false)]).abs() < 0.02);
            assert!(h[(true, true)].abs() < 0.02);
        }
    }
}
