# aRtySt
Convert any image to beautiful ASCII art with multiple image editing/processing and output formats.

# Basic usage
To convert your picture to a simple ascii art use the following command:
``` bash
artyst -W <width of the output in characters> <input image name>
```
This is just about as easy as it can get.

There are more options available for example you can specify the height instead of the width:
``` bash
artyst -H <height of the output in characters> <input image name>
```
Both of there options try their best to preserve the image aspect ratio. However if you specify both the width and the height of the
output the aspect ratio is discarded.

you can add a cut-off threshold to snap all the pixels with a brightness less than a certain amount to zero. using the `--threshold` option:
``` bash
artyst -W <width of the output in characters> -T 0.1 <input image name>
```
the threshold must be a number between 0 and 1 naturally.

---

There is an option to get the output in unicode braile characters, this method can preserve more image details in a much smaller size.

To do this use the `-t BRAILE` option:
``` bash
artyst -t BRAILE -W <width of the output in dots> -H <height of the output in dots> -T <threshold> <input image name>
```
Notice that in this use case, the width and height are given in dots rather than characters. Basically, each braile character is just
8 dots arranged in a 2x4 matrix ( ⣿ ) take this into account when providing the output dimensions in braile use case.

# Use like a pro :: how it works
The help dialog reads:
```
Usage: ar [-h] [-t TXT|BRAILE] [-s RESIZE|LEGACY] [-d ONOFF|INTERPOLATING] [-k ATKINSON|NONE|FS|STUCKI] [-T FLOAT] [-f FORMATSTR] [-F FORMATSTR] [-c FLOAT
] [-b INTEGER] [-W INTEGER] [-H INTEGER] [-o FILENAME] [-C FILENAME] [-I (FLOAT,)*]

A simple program that converts images into ascii art.


Options:
    -h, --help          display this help message
    -t, --type TXT|BRAILE
                        type of output
    -s, --seg-type RESIZE|LEGACY
                        how to segmentate the image
    -d, --dith-type ONOFF|INTERPOLATING
                        type of the ditherer used
    -k, --kernel ATKINSON|NONE|FS|STUCKI
                        type of kernel to use in ditherer
    -T, --threshold FLOAT
                        cut-off threshold
    -f, --fmt FORMATSTR format string for each character
    -F, --fmtln FORMATSTR
                        format string for each line
    -c, --contrast FLOAT
                        contrast level
    -b, --brighten INTEGER
                        increase image brightness level
    -W, --width INTEGER width of the output character matrix
    -H, --height INTEGER
                        width of the output character matrix
    -o, --output FILENAME
                        output file default=stdout
    -C, --chars STRING|@FILENAME
                        list of characters to use as output
    -I, --inter-points (FLOAT,)*
                        interpolation points

  NOTE: character and line formatting are not implemented yet
  NOTE: HTML output format is not implemented yet.
```
## pre-process phase
Before the program begins its conversion from image to text, it first some initai editing. First it adjusts the brighness of
the image, then the conrast. Pay attention to the order of these operation. The adjustment levels can be manually specified
using the `-b / --brighten` and `-c / --contrast` options for brightness and contrast respetively. Brightness adjustment is 
an integer between -127 and 127 and contrast adjustment can be any floating-point number.

## use cases
There are two output types: `TXT` and `BRAILE`. You can select one of these using the `-t / --type` option. The `TXT` output 
type uses a list of characters, specified by `-C / --chars` to render the ascii art, the list must be ordered from the darkest 
character (eg: ' ') to the brightest character (eg: '#') the program concerts the picture into a matrix on floating numbers, 
each between 0 and 1, 0 being the darkest and 1 being the brightest. this range is then quantized to select one of the characters
from the list of characters. The default list of characters is as shown below:
```
[' ','.','`','\'','-','~','+','^',':',';','>','<','?',')','(','|',']','[','}','{','\\','/',
'i','1','l','L','0','O','m','q','d','k','#','W','%','&','B','@','$']
```
## processing
The quantization takes place in a Kernel Ditherer. There are two types of Ditherers, `Interpolating` and `On-Off`. you can specify this 
behavior using the `-d / --dith-type` option. The `On-Off` ditherer, as the name suggests, snaps each value to either 0 or 1, thus 
obviously, this should only be used when the character sequence only consists of two characters. This ditherer was originally developed
to be used in the `BRAILE` use case, however, you can still use it in `TXT` mode. the `threshold` specified using the `-T / --threshold`
option, specifies the breaking point in this ditherer, anything below this is mapped to 0 and otherwise to 1. According to the previous 
explanation it becomes apparent that the `threshold` option must be specified when in `BRAILE` mode; not specifying it results in a 
warning and the value is assumed to be 0.5 in this case. The `Interpolating` ditherer, uses a sequence on increasing floating point numbers 
between 0 and 1 to quantize the value. This sequece can be manually provided using the `-I / --inter-points` option; again, it is important
that the number of interpolating points are equal to that of the character sequence, if that is not the case, a warning is produced. If this 
option is absent, the sequence will be generated automatically depending on whether the `-T / --threshold` option is specified. If it is not,
the range will be divided equally. If it is provied, then the range [0, `threshold`) is mapped to zero and the range \[`Threshold` , 1\] is
again equally divided to account for the rest of the characters.

The ditherer, will attempt to use an "error distribution" technique, depending on the type of `Kernel` specified using the `-k / --kernel`
option. The default is `NONE` however the you can choose between the other provided kernels in hopes of getting a smoother output. the provided
kernels are:
. [Floyd-Steinberg](https://en.wikipedia.org/wiki/Floyd%E2%80%93Steinberg_dithering)
. [Stucki](https://forum.lightburnsoftware.com/t/stucki-dither-vs-jarvis-dither/14528)
. [Atkinson](https://en.wikipedia.org/wiki/Atkinson_dithering)

## post processing
This part is still in developments.

# Installation
You can download and install pre-compiled releases from this github repo's releases page.

Since this is a cargo project, it is extremely easy to compile it from the source, just download this repository and use
``` bash
cargo build --release
```
to build the latest release version.

The entire project is written in rust and so you need to have the basic pre-requisites to compile this application.

# Special Thanks
I'd like to shout out @LachlanArthur. His amazing work on the project [Braille-ASCII-Art](https://lachlanarthur.github.io/Braille-ASCII-Art/) inspired 
and guided me to implement a lot of the features in this project. Their project being licesed as MIT is what allowed this project to be developed very quicky.

The repo in question: [Braille-ASCII-Art.git](https://github.com/LachlanArthur/Braille-ASCII-Art)
