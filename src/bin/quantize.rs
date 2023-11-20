use clap::{Parser};
use multidimension::{Size, View, Array};
use fvq::{Error, Grid, Position, Pyramid};
use fvq::io::{cli, load_image, save_image, Pixels, PixelArray, L};
use fvq::quantize::{to_digital, from_digital};

fn main() -> fvq::Result {
    let args = cli::InOutOrder::parse();
    let order = args.order(5);
    let in_pixels = load_image(&args.in_path)?;
    let in_pixels: Array<Grid, f32> = match in_pixels {
        Pixels::L(pa) => pa.crop_to_multiple(1 << order).column(L).collect(),
        _ => Err(Error("Image must only have a luma channel"))?,
    };
    let mut pyramid = Pyramid::from_pixels(order, true, in_pixels);
    pyramid.size().each(|yx| {
        let low = pyramid[yx];
        let pos = Position {level: 0, yx};
        let tree = pyramid.get(pos);
        let tree = to_digital(order, low, &tree);
        let tree = from_digital(order, low, &tree);
        pyramid.set(pos, &tree);
    });
    let out_pixels = pyramid.to_pixels(true);
    let out_pixels = Pixels::L(PixelArray(Array::new(((), out_pixels.size()), out_pixels.to_raw())));
    save_image(&out_pixels, &args.out_path("quantize")?)
}
