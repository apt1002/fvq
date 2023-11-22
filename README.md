# Fractal Vector Quantizer: version 34 of my image compression experiment.

My goal is to invent a lossy image compression algorithm for natural images
(i.e. photographs). This is the latest in a series of experiments with which I
am gradually developing the technology. If successful, I will define the FVQ
file format for storing compressed images, and I will deliver a tool for
converting images to and from FVQ format.

## Competitive position

Many good lossy image compression formats already exist. [JPEG] remains the
most popular, though there are now some established formats that compress
better, such as [WebP] and [AVIF]. Mine competes as follows:

 - Smaller compressed files. I'm aiming for 0.3 bits per pixel or better: half
the file size of a JPEG.

 - Designed for noisy images. Since about 2010, the quality of photographs has
been limited by noise, not by resolution. There is a gap in the market for a
format with low image quality and small files, suitable for storing high
resolution images.

 - Computationally cheap, especially for decompression. It is more expensive
than JPEG, but not much.

 - Easy to implement.

## Interesting features

I have developed a good wavelet transform. It has the following properties:

 - Exactly orthonormal - It does not distort the noise ellipsoid.

 - Exactly invertible - No loss of information (besides rounding errors).

 - Smooth - The [mother wavelet] has four zero moments, meaning that an image
that is a bicubic function or simpler has a high-frequency component of zero.

 - Reasonably cheap to compute.

I have developed a perceptual model that replaces [gamma correction]. This is
needed to avoid applying the wavelet transform to gamma-corrected data. JPEG
and some other image compression algorithms *do* apply linear transforms to
gamma-corrected data, which results in unsightly artifacts.

I use [lattice quantization] for the larger wavelet coefficients. The
coefficients naturally come in triplets, which I quantise using the
body-centred cubic lattice (a.k.a. A3* and D3*).

I (will) use [vector quantization] for the smaller wavelet coefficients.

More to come!

## Status

This is a work in progress (#3). Some parts of the algorithm are working, and others
are not yet written. I have not yet reached the point where I can generate a
compressed image file.

Even after that point, the specification of an FVQ file will likely change as I
optimise and simplify the algorithm. This software should therefore not be used
for storing valuable data at the moment.

If the file format proves useful, it will be desirable to optimise the
software. That would be a nice problem to have.

## Usage

The software is written in [Rust] and built using [cargo].

```
$ git clone git@github.com:apt1002/fvq.git
$ cd fvq
$ cargo run --bin [...]
```

[JPEG]: https://en.wikipedia.org/wiki/JPEG
[WebP]: https://en.wikipedia.org/wiki/WebP
[AVIF]: https://en.wikipedia.org/wiki/AVIF
[mother wavelet]: https://en.wikipedia.org/wiki/Wavelet#Mother_wavelet
[gamma correction]: https://en.wikipedia.org/wiki/Gamma_correction
[lattice quantization]: https://publications.lib.chalmers.se/records/fulltext/23008/23008.pdf
[body-centred cubic lattice]: https://www.physics-in-a-nutshell.com/article/12/body-centered-cubic-bcc
[vector quantization]: https://en.wikipedia.org/wiki/Vector_quantization
[Rust]: https://www.rust-lang.org/
[cargo]: https://doc.rust-lang.org/cargo/
