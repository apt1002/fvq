# A Glued Quantisation Lattice for Images

A [2023 paper] by Agrell, Pook-Kolb and Allen observes that glued lattices make better quantisation lattices than product lattices. Like everybody else, in my image compression algorithms I have until now been quantising images onto a product lattice. This document describes how I might instead use a glued lattice.

[recent paper]: https://arxiv.org/pdf/2312.00481.pdf

## Background

Compressing an image involves three main concepts: exploiting correlations; throwing away imperceptible information; and encoding the rest as compactly as possible.

Nearby pixels of an image are significantly correlated. After converting the image to a linear colour space, the first step is a rough pass to exploit the linear correlations. A 2×2 tile of pixels can be expressed as a linear combination of the following basis vectors:

    +½ +½   +½ -½   +½ +½   +½ -½
    +½ +½   +½ -½   -½ -½   -½ +½

This is the Haar basis. The vectors are orthogonal and of norm 1, so the transform does not change the noise ellipsoid; this is an important property. The first of these vectors is a low pass filter. The other three are wavelets, which I call V (Vertical edge), H (Horizontal edge) and C (Chequer pattern). For colour images, each coefficient has three colour channels, but handling colour is outside the scope of this document.

Collecting the low-frequency component of all the 2×2 tiles yields an image of half the resolution and twice the signal-to-noise ratio. The other three components are typically small and much less correlated with each other than the original pixels were. Correlations can be slightly further reduced by mixing coefficients from neighbouring tiles, but that is outside the scope of this document.

The half-resolution image, being an image, still contains significant correlations. Repeating the transform on this image yields a quarter-resolution image, and so on. Eventually (after about five generations) the image is small enough, and of a high enough signal-to-noise ratio, that it is not worth making any effort to compress it. The task is then to compress the five generations of wavelet coefficients, each of which is a grid of (V, H, C) triplets.

The next step is to scale each wavelet coefficient according to how easy it is to perceive a small change. The perceptual model is outside the scope of this document. Here, let's assume that the coefficients are scaled such that the appropriate measure of the difference between two similar images is the sum of the squares of the differences of the wavelet coefficients.

The next step is to quantise the image, i.e. to round the (effectively analogue) input image to the nearest of a discrete set of "representable images". These are the images that can possibly be the output of the decompression algorithm. Since the compressed file is supposed to be small, we want the set of representable images to be as small as possible, while still containing images that are close to any plausible input image. Choosing the set of representable images is the main subject of this document.

The last step is to encode the quantised image into a stream of bits. This involves making a probability model. It is an opportunity to exploit any remaining correlations in the data, including all the non-linear correlations. The probability model is outside the scope of this document.

To summarise, the focus of this document is to choose the set of images onto which the input image is rounded. A reasonable idea (the alternatives are out of scope) is to define a lattice, i.e. a set which is closed under addition and subtraction. The lattice should have as few points as possible, while still offering a reasonable approximation of any plausible image. The quality of the approximation will be measured by the sum of the squares of the errors in the wavelet coefficients: the L2 norm.
