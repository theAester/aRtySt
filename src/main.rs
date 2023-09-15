extern crate getopts;

use getopts::{Options, Occur, HasArg};
use std::path::Path;
use std::fs::File;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;

mod segment;
mod image_process;
mod matrix;
mod text;

use crate::segment::*;
use crate::image_process::*;
use crate::matrix::Matrix;
use crate::text::*;

const PROGDESC: &'static str = "A simple program that converts images into ascii art.\n";

pub enum ProgType{
    TXT,
    BRAILE,
}

pub enum AlgType{
    KERNEL,
    LEGACY,
}

fn print_help(progname: String, parser: Options){
    println!("{}\n\n{}\n  NOTE: character and line formatting are not implemented yet.\n  NOTE: HTML output format is not implemented yet.", parser.short_usage(&progname), parser.usage(PROGDESC));
}

fn parse_args(args: Vec<String>) -> Result<(ProgType,
                                            AlgType,
                                            String, 
                                            String,
                                            f32,
                                            i32,
                                            u32,
                                            u32,
                                            Option<File>,
                                            Option<File>,
                                            String), ()>{
    let progname = args[0].clone();
    let mut parser = Options::new();
    parser.optflag("h", "help", "display this help message");
    parser.opt("t", "type", "type of output", "[TXT|BRAILE]", HasArg::Yes, Occur::Optional);
    parser.opt("T", "op-type", "how to process the image", "[KERNEL|LEGACY]", HasArg::Yes, Occur::Optional);
    parser.opt("f", "fmt", "format string for each character", "FORMATSTR", HasArg::Yes, Occur::Optional);
    parser.opt("F", "fmtln", "format string for each line", "FORMATSTR", HasArg::Yes, Occur::Optional);
    parser.opt("c", "contrast", "contrast level", "FLOAT", HasArg::Yes, Occur::Optional);
    parser.opt("b", "brighten", "increase image brightness level", "INTEGER", HasArg::Yes, Occur::Optional);
    parser.opt("W", "width", "width of the output character matrix", "INTEGER", HasArg::Yes, Occur::Optional);
    parser.opt("H", "height", "width of the output character matrix", "INTEGER", HasArg::Yes, Occur::Optional);

    parser.opt("o", "output", "output file default=stdout", "FILENAME", HasArg::Yes, Occur::Optional);
    parser.opt("C", "chars", "list of characters to use as output", "FILENAME", HasArg::Yes, Occur::Optional);

    let matches = parser.parse(args[1..].into_iter()).unwrap();

    let mut out_type: ProgType = ProgType::TXT;
    let mut alg_type: AlgType = AlgType::LEGACY;
    let mut fmt_str: String = String::from("{}");
    let mut fmt_ln_str: String = String::from("{}\n");
    let mut contrast: f32 = 0.0;
    let mut brighten: i32 = 0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut output: Option<File> = None;
    let mut chars: Option<File> = None;


    if matches.opt_present("h"){
        print_help(progname, parser);
        std::process::exit(0);
    }

    if matches.opt_present("t"){
        let temp: String = match matches.opt_str("t"){
            Some(s) => s,
            None => {
                println!("-t option expects an argument: TXT | BRAILE");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "txt" { out_type = ProgType::TXT; }
        else if temp == "braile" { out_type = ProgType::BRAILE; }
        else {
            println!("-t option expects an argument: TXT | BRAILE");
            return Err(());
        }
    }

    if matches.opt_present("T"){
        let temp: String = match matches.opt_str("T"){
            Some(s) => s,
            None => {
                println!("-T option expects an argument: KERNEL | LEGACY");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "kernel" { alg_type = AlgType::KERNEL; }
        else if temp == "legacy" { alg_type = AlgType::LEGACY; }
        else {
            println!("-T option expects an argument: KERNEL | LEGACY");
            return Err(());
        }
    }

    if matches.opt_present("f"){
        println!("the formatting feature is not implemented yet. Ignoring this option");
    }
    if matches.opt_present("F"){
        println!("the formatting feature is not implemented yet. Ignoring this option");
    }

    if matches.opt_present("c"){
        let temp = match matches.opt_str("c"){
            Some(s) => s,
            None => {
                println!("-c option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        contrast = match temp {
            Ok(s) => s,
            Err(_) => {
                println!("the argument given to -c is not a valid FLOAT number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("b"){
        let temp = match matches.opt_str("b"){
            Some(s) => s,
            None => {
                println!("-b option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<i32>();

        brighten = match temp {
            Ok(s) => s,
            Err(_) => {
                println!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("W"){
        let temp = match matches.opt_str("W"){
            Some(s) => s,
            None => {
                println!("-W option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        width = match temp {
            Ok(s) => s,
            Err(_) => {
                println!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("H"){
        let temp = match matches.opt_str("H"){
            Some(s) => s,
            None => {
                println!("-H option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        height = match temp {
            Ok(s) => s,
            Err(_) => {
                println!("the argument given to -H is not a valid INTEGER number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("o"){
        let temp: String = match matches.opt_str("o"){
            Some(s) => s,
            None => {
                println!("-o option expects an argument: FILENAME");
                return Err(());
            }
        };
        let temp_path = Path::new(&temp);
        if temp_path.exists() && !temp_path.is_file() {
            println!("cannot open {} for writing. File exists and is not a regular file.", temp);
            return Err(());
        }
        output = Some(File::options().write(true).append(false).create(true).open(temp).expect("unexpected error occured when openning output file"));
    }

    if matches.opt_present("C"){
        let temp: String = match matches.opt_str("C"){
            Some(s) => s,
            None => {
                println!("-C option expects an argument: FILENAME");
                return Err(());
            }
        };
        let temp_path = Path::new(&temp);
        if !temp_path.exists() {
            println!("cannot open {} for reading: File does not exist.", temp);
            return Err(());
        }
        if temp_path.exists() && !temp_path.is_file() {
            println!("cannot open {} for reading: File exists and is not a regular file.", temp);
            return Err(());
        }
        chars = Some(File::options().read(true).open(temp).expect("unexpected error occured when openning chars file"));
    }

    if matches.free.len() != 1{
        println!("WRONG USAGE! See --help for more info");
        return Err(());
    }
    let temp_path = Path::new(&matches.free[0]);
    if ! temp_path.exists() {
        println!("cannot open {} for reading. File does not exist.", matches.free[0]);
        return Err(());
    }
    if temp_path.exists() && !temp_path.is_file() {
        println!("cannot open {} for reading. File exists and is not a regular file.", matches.free[0]);
        return Err(());
    }
    let input = matches.free[0].clone();

    Ok((out_type, alg_type, fmt_str, fmt_ln_str, contrast, brighten, width, height, output, chars, input))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (
        _out_type,
        alg_type,
        fmt_str,
        fmt_ln_str,
        contrast,
        brighten,
        mut width,
        mut height,
        output,
        chars,
        input,
        ) = parse_args(args).expect("Error while parsing input arguments. Aborting.");

    if width == 0 && height == 0 {
        panic!("Input error: atleast one of --width or --height options must be specified.\nsee --help for more info.");
    }

    let dyn_image = ImageReader::open(&input).expect("Unexpected error while reading input file").decode().unwrap();

    if width == 0 {
        let aspect_ratio = (dyn_image.width() as f32) / (dyn_image.height() as f32);
        width = ((height as f32) * aspect_ratio).floor() as u32;
    }

    if height == 0 {
        let iaspect_ratio = (dyn_image.height() as f32) / (dyn_image.width() as f32);
        height = ((width as f32) * iaspect_ratio).floor() as u32;
    }

    let width = width;
    let height = height;

    match alg_type {
        AlgType::LEGACY => {
            let stt_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale()
                .into_luma8();


            let segment_info = SegmentInfo::generate(stt_image.width(), stt_image.height(), width, height);

            let mut matrix = Matrix::<f32>::new(width, height, 0.0);

            generate_matrix_legacy(stt_image, &mut matrix, segment_info);

            print_output(matrix, fmt_str, fmt_ln_str, chars, output);
        },
        AlgType::KERNEL => {
            let stt_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale()
                .resize_exact(width, height, FilterType::Gaussian)
                .into_luma8();

            let mut matrix = Matrix::<f32>::new(width, height, 0.0);

            generate_matrix(stt_image, &mut matrix); //, segment_info);

            print_output(matrix, fmt_str, fmt_ln_str, chars, output);
        }
    }
}
