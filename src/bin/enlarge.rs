use clap::{Parser};
use multidimension::{View, Array};
use fvq::io::{cli, load_image, save_image, Pixels, PixelArray, L};
use fvq::transform::{Haar, from_haar, twiddle_grid};
use fvq::{Error, Grid};

fn main() -> fvq::Result {
    let args = cli::InOutOrder::parse();
    let in_pixels = load_image(&args.in_path)?;
    let in_pixels: Array<Grid, f32> = match in_pixels {
        Pixels::L(pa) => pa.column(L).collect(),
        _ => Err(Error("Image must only have a luma channel"))?,
    };
    let mut pixels = in_pixels;
    for _ in 0..args.order(1) {
        let haar = pixels.map(|low| Haar::new(low * 2.0, 0.0, 0.0, 0.0)).collect();
        let haar = twiddle_grid::<true>(haar);
        pixels = from_haar(haar).collect::<Array<Grid, f32>>();
    }
    let out_pixels = Pixels::L(PixelArray(Array::new(((), pixels.size()), pixels.to_raw())));
    save_image(&out_pixels, &args.out_path("enlarge")?)
}
