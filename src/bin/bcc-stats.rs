use std::collections::{HashMap};
use clap::{Parser};
use multidimension::{Size, View, Array};
use fvq::{Error, Grid, Tree, Position, Pyramid};
use fvq::io::{load_image, Pixels, L};
use fvq::quantize::{to_digital, ShiftedBCC, Residual, ALL_RESIDUALS, Chain};

#[derive(Debug, Parser)]
#[command(about = "Collect statistics about a corpus of images.")]
#[command(author, version, long_about = None)]
struct Args {
    /// Filename of a list of image filenames.
    pub list_path: String,

    /// The order of the wavelet pyramid.
    #[arg(short = 'n', long)]
    pub order: Option<usize>,
}

impl Args {
    /// Returns the `order` or the specified default value.
    pub fn order(&self, default_order: usize) -> usize {
        self.order.unwrap_or(default_order)
    }
}

// ----------------------------------------------------------------------------

/// An abbreviation of a `ShiftedBCC` that is not a fixed point of `arrow()`.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BCCSummary {
    /// The number of steps in `chain`.
    pub length: u8,

    /// The fixed-point at which the [`Chain`] ends: [`Chain::last_residual`].
    pub fixed_point: Residual,

    /// The most significant `Residual` in [`Chain::residuals`].
    pub last: Residual,

    /// The least significant `Residual` in [`Chain::residuals`]
    pub first: Residual,
}

impl From<Chain> for BCCSummary {
    fn from(chain: Chain) -> Self {
        let length = u8::try_from(chain.residuals.len()).unwrap();
        let fixed_point = chain.last_residual;
        let last = *chain.residuals.last().expect("Too short");
        let first = *chain.residuals.first().expect("Too short");
        Self {length, fixed_point, last, first}
    }
}

// ----------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
pub struct BCCStatistics {
    /// The number of [`Tree::Leaf`]s.
    pub leaf_count: usize,

    /// For each [`ShiftedBCC`] that is a fixed point of `arrow()`, the number
    /// of [`Tree::Branch`]es whose `payload` is that `ShiftedBCC`.
    pub short_counts: HashMap<Residual, usize>,

    /// For each [`BCCSummary`], the number of [`Tree::Branch`]es whose
    /// `payload` matches that summary.
    pub long_counts: HashMap<BCCSummary, usize>,
}

impl BCCStatistics {
    /// Increment [`leaf_count`].
    ///
    /// [`leaf_count`]: Self::leaf_count
    pub fn count_leaf(&mut self) { self.leaf_count += 1; }

    /// Increment the [`bcc_counts[bcc]`].
    ///
    /// [`bcc_counts[bcc]`]: Self::bcc_counts
    pub fn count_bcc(&mut self, bcc: ShiftedBCC) {
        let chain = Chain::from_bcc(bcc);
        let chain = chain.apply_symmetry(chain.last_residual.recommend_symmetry());
        if chain.residuals.len() == 0 {
            *self.short_counts.entry(chain.last_residual).or_insert(0) += 1;
        } else {
            *self.long_counts.entry(BCCSummary::from(chain)).or_insert(0) += 1;
        }
    }

    /// Recursively count every node of `tree`.
    pub fn count_tree(&mut self, tree: &Tree<ShiftedBCC>) {
        match tree {
            Tree::Branch(branch) => {
                self.count_bcc(branch.payload);
                branch.children.as_ref().each(|child| self.count_tree(child));
            },
            Tree::Leaf => self.count_leaf(),
        }
    }

    /// Count every `Tree` of `pyramid`.
    pub fn count_pyramid(&mut self, pyramid: &Pyramid) {
        pyramid.size().each(|yx| {
            let low = pyramid.low[yx];
            let pos = Position {level: 0, yx};
            let tree = pyramid.get(pos);
            let tree = to_digital(pyramid.order(), low, &tree);
            self.count_tree(&tree);
        });
    }
}

// ----------------------------------------------------------------------------

fn main() -> fvq::Result {
    let args = Args::parse();
    let image_paths: Vec<String> = std::fs::read_to_string(&args.list_path)?.lines().map(String::from).collect();
    eprintln!("Collecting statistics from {} images", image_paths.len());
    let order = args.order(5);
    let mut pixel_count = 0;
    let mut statistics = BCCStatistics::default();
    for image_path in &image_paths {
        let in_pixels = load_image(image_path)?;
        let in_pixels: Array<Grid, f32> = match in_pixels {
            Pixels::L(pa) => pa.crop_to_multiple(1 << order).column(L).collect(),
            _ => Err(Error("Image must only have a luma channel"))?,
        };
        pixel_count += in_pixels.len();
        let pyramid = Pyramid::from_pixels(order, true, in_pixels);
        statistics.count_pyramid(&pyramid);
        eprint!("."); std::io::Write::flush(&mut std::io::stderr())?;
    }
    eprintln!();
    println!("pixel_count = {:?}", pixel_count);
    println!("leaf_count = {:?}", statistics.leaf_count);
    for fixed_point in [ALL_RESIDUALS[0], ALL_RESIDUALS[4]] {
        println!();
        println!("Fixed point {:?}", fixed_point);
        println!("short_count = {:?}", statistics.short_counts.get(&fixed_point).unwrap_or(&0));
        for &last in &ALL_RESIDUALS {
            if last != fixed_point {
                println!();
                println!("Last {:?}", last);
                for &first in &ALL_RESIDUALS {
                    print!("First {:?}:", first);
                    for length in 1..15 {
                        let bs = BCCSummary {length, fixed_point, last, first};
                        print!(" {:8?}", statistics.long_counts.get(&bs).unwrap_or(&0));
                    }
                    println!();
                }
            }
        }
    }
    Ok(())
}
