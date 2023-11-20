use clap::{Parser};
use multidimension::{View, Array};
use fvq::io::{cli, load_image, save_image, Pixels, PixelArray, L};
use fvq::{Error, Pyramid, Grid};

fn main() -> fvq::Result {
    let args = cli::InOutOrder::parse();
    let order = args.order(5);
    let in_pixels = load_image(&args.in_path)?;
    let in_pixels: Array<Grid, f32> = match in_pixels {
        Pixels::L(pa) => pa.crop_to_multiple(1 << order).column(L).collect(),
        _ => Err(Error("Image must only have a luma channel"))?,
    };
    let pyramid = Pyramid::from_pixels(order, true, in_pixels);
    let out_pixels = pyramid.montage();
    let out_pixels = Pixels::L(PixelArray(Array::new(((), out_pixels.size()), out_pixels.to_raw())));
    save_image(&out_pixels, &args.out_path("wavelet")?)
}
