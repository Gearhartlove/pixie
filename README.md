# What is Pixie? 
Pixie is a magically simple image pixelation cli! 

# Example
```bash
$ pixie --help
Usage: pixie [OPTIONS]

Options:
  -p, --palette <PALETTE>      The palatte which will be used to pixelate the image
  -s, --size <SIZE>            The number of pixels making up each pixel in the output image
  -n, --name <NAME>            The name of the output file
  -l, --large                  Exports the image at its input size
  -i, --image <IMAGE>          Image to pixelate
  -d, --directory <DIRECTORY>  File of images to pixelate
  -h, --help                   Print help
  -V, --version                Print version

$ pixie --image example.png
```
<p align="center">
    <img src="readme/showcase.png" alt="drawing" width="600"/>
</p>

# Details
Given a palette and at least one picture, pixie matches the closest color
in the palette to a block of pixels in the base image. It collects these
approximations in an image buffer and then saves to a file. 

# Common Questions
- Can I pixelate a whole directory of photos? 
  - Yes
- What are the supported file formats? 
  - .png & .jpeg
- What format do my palettes need to be in? 
  - .hex
- Do I need to have the palette installed?
  - No, pixie will fetch from lospec.com if pixie can't find it locally
- Can I change the resolution of the base image? 
  - yes with the `--size` flag
- What is the resulting size of my image? 
  - It's scaled down to your pixel size by default. The `--large` flag will persist the original image size.

# Contributers
@MrVintage710
@Gearhartlove
