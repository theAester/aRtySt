extern crate getopts;

use getopts::{Options, Occur, HasArg};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;
use image::DynamicImage;

mod segment;
mod image_process;
mod matrix;
mod text;
mod ditherer;
mod kernel;

use crate::segment::*;
use crate::image_process::*;
use crate::matrix::Matrix;
use crate::text::*;
use crate::kernel::*;

const PROGDESC: &'static str = "A simple program that converts images into ascii art.\n";

#[derive(PartialEq)]
pub enum ProgType{
    TXT,
    BRAILE,
}

#[derive(PartialEq)]
pub enum SegType{
    RESIZE,
    LEGACY,
}

#[derive(PartialEq)]
pub enum DithType{
    ONOFF,
    INTER,
}

pub type ThreshOption = Option<f32>;
pub type CharsOption = Option<String>;
pub type OutputFile = Option<File>;
pub type InterPoints = Option<Vec<f32>>;

fn print_help(progname: String, parser: Options){
    println!("{}\n\n{}\n  NOTE: character and line formatting are not implemented yet.\n  NOTE: HTML output format is not implemented yet.", parser.short_usage(&progname), parser.usage(PROGDESC));
}

fn parse_args(args: Vec<String>, map_kernel: &KerMap) -> Result<(ProgType,
                                            SegType,
                                            DithType,
                                            String,
                                            ThreshOption,
                                            String, 
                                            String,
                                            f32,
                                            i32,
                                            u32,
                                            u32,
                                            OutputFile,
                                            CharsOption,
                                            InterPoints,
                                            String), ()>{

    let ker_types: Vec<&str> = map_kernel.clone().into_keys().collect();
    let help_kernel_types = ker_types.join("|");

    let progname = args[0].clone();
    let mut parser = Options::new();
    parser.optflag("h", "help", "display this help message");
    parser.opt("t", "type", "type of output", "TXT|BRAILE", HasArg::Yes, Occur::Optional);
    parser.opt("s", "seg-type", "how to segmentate the image", "RESIZE|LEGACY", HasArg::Yes, Occur::Optional);
    parser.opt("d", "dith-type", "type of the ditherer used", "ONOFF|INTERPOLATING", HasArg::Yes, Occur::Optional);
    parser.opt("k", "kernel", "type of kernel to use in ditherer", help_kernel_types.as_str(), HasArg::Yes, Occur::Optional);
    parser.opt("T", "threshold", "cut-off threshold", "FLOAT", HasArg::Yes, Occur::Optional);
    parser.opt("f", "fmt", "format string for each character", "FORMATSTR", HasArg::Yes, Occur::Optional);
    parser.opt("F", "fmtln", "format string for each line", "FORMATSTR", HasArg::Yes, Occur::Optional);
    parser.opt("c", "contrast", "contrast level", "FLOAT", HasArg::Yes, Occur::Optional);
    parser.opt("b", "brighten", "increase image brightness level", "INTEGER", HasArg::Yes, Occur::Optional);
    parser.opt("W", "width", "width of the output character matrix", "INTEGER", HasArg::Yes, Occur::Optional);
    parser.opt("H", "height", "width of the output character matrix", "INTEGER", HasArg::Yes, Occur::Optional);
    parser.opt("o", "output", "output file default=stdout", "FILENAME", HasArg::Yes, Occur::Optional);
    parser.opt("C", "chars", "list of characters to use as output", "FILENAME", HasArg::Yes, Occur::Optional);
    parser.opt("I", "inter-points", "interpolation points", "(FLOAT,)*", HasArg::Yes, Occur::Optional);

    let matches = parser.parse(args[1..].into_iter()).unwrap();

    let mut out_type: ProgType = ProgType::TXT;
    let mut seg_type: SegType = SegType::LEGACY;
    let mut dith_type: DithType = DithType::INTER;
    let mut ker_type: String = String::from("NONE"); 
    let mut threshold: ThreshOption = None;
    let mut fmt_str: String = String::from("{}");
    let mut fmt_ln_str: String = String::from("{}\n");
    let mut contrast: f32 = 0.0;
    let mut brighten: i32 = 0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut output: OutputFile = None;
    let mut chars: CharsOption = None;
    let mut inter_points: InterPoints = None;


    if matches.opt_present("h"){
        print_help(progname, parser);
        std::process::exit(0);
    }

    if matches.opt_present("t"){
        let temp: String = match matches.opt_str("t"){
            Some(s) => s,
            None => {
                eprintln!("-t option expects an argument: TXT|BRAILE");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "txt" { out_type = ProgType::TXT; }
        else if temp == "braile" { out_type = ProgType::BRAILE; }
        else {
            eprintln!("-t option expects an argument: TXT|BRAILE");
            return Err(());
        }
    }

    if matches.opt_present("s"){
        let temp: String = match matches.opt_str("s"){
            Some(s) => s,
            None => {
                eprintln!("-s option expects an argument: RESIZE|LEGACY");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "resize" { seg_type = SegType::RESIZE; }
        else if temp == "legacy" { 
            if out_type == ProgType::BRAILE{
                eprintln!("Illegal Combination of options: cannot use Legacy segmentation for braile output.");
                return Err(());
            }
            seg_type = SegType::LEGACY;
        }
        else {
            eprintln!("-s option expects an argument: RESIZE|LEGACY");
            return Err(());
        }
    }

    if matches.opt_present("d"){
        let temp: String = match matches.opt_str("d"){
            Some(s) => s,
            None => {
                eprintln!("-d option expects an argument: ONOFF|INTERPOLATING");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "onoff" { dith_type = DithType::ONOFF; }
        else if temp == "inter" || temp == "interpolating" { 
            if out_type == ProgType::BRAILE{
                eprintln!("Illegal Combination of options: cannot use interpolating ditherer for braile output.");
                return Err(());
            }
            dith_type = DithType::INTER; 
        }
        else {
            eprintln!("-d option expects an argument: ONOFF|INTERPOLATING");
            return Err(());
        }
    }

    if matches.opt_present("k"){
        let temp: String = match matches.opt_str("k"){
            Some(s) => s,
            None => {
                eprintln!("-k option expects an argument: {}", help_kernel_types);
                return Err(());
            }
        }.trim().to_lowercase();
        if map_kernel.contains_key(temp.to_uppercase().as_str()) {
            ker_type = temp.to_uppercase();
        }else{
            eprintln!("-k option expects an argument: {}", help_kernel_types);
            return Err(());
        }
    }

    if matches.opt_present("T"){
        let temp = match matches.opt_str("T"){
            Some(s) => s,
            None => {
                eprintln!("-T option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        threshold = match temp {
            Ok(s) => ThreshOption::Some(s),
            Err(_) => {
                eprintln!("the argument given to -T is not a valid FLOAT number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("f"){
        eprintln!("Formatting features are not implemented yet. Ignoring this option: -f");
    }
    if matches.opt_present("F"){
        eprintln!("Formatting features are not implemented yet. Ignoring this option: -F");
    }

    if matches.opt_present("c"){
        let temp = match matches.opt_str("c"){
            Some(s) => s,
            None => {
                eprintln!("-c option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        contrast = match temp {
            Ok(s) => s,
            Err(_) => {
                eprintln!("the argument given to -c is not a valid FLOAT number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("b"){
        let temp = match matches.opt_str("b"){
            Some(s) => s,
            None => {
                eprintln!("-b option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<i32>();

        brighten = match temp {
            Ok(s) => s,
            Err(_) => {
                eprintln!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("W"){
        let temp = match matches.opt_str("W"){
            Some(s) => s,
            None => {
                eprintln!("-W option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        width = match temp {
            Ok(s) => s,
            Err(_) => {
                eprintln!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("H"){
        let temp = match matches.opt_str("H"){
            Some(s) => s,
            None => {
                eprintln!("-H option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        height = match temp {
            Ok(s) => s,
            Err(_) => {
                eprintln!("the argument given to -H is not a valid INTEGER number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("o"){
        let temp: String = match matches.opt_str("o"){
            Some(s) => s,
            None => {
                eprintln!("-o option expects an argument: FILENAME");
                return Err(());
            }
        };
        let temp_path = Path::new(&temp);
        if temp_path.exists() && !temp_path.is_file() {
            eprintln!("cannot open {} for writing. File exists and is not a regular file.", temp);
            return Err(());
        }
        output = Some(File::options().write(true).append(false).create(true).open(temp).expect("unexpected error occured when openning output file"));
    }

    if matches.opt_present("C"){
        let temp: String = match matches.opt_str("C"){
            Some(s) => s,
            None => {
                eprintln!("-C option expects an argument: FILENAME");
                return Err(());
            }
        };
        // file specified
        if temp.chars().nth(0).unwrap() == '@' {
            let temp_path = Path::new(&temp);
            if !temp_path.exists() {
                eprintln!("cannot open {} for reading: File does not exist.", temp);
                return Err(());
            }
            if temp_path.exists() && !temp_path.is_file() {
                eprintln!("cannot open {} for reading: File exists and is not a regular file.", temp);
                return Err(());
            }
            let mut file = File::options().read(true).open(temp).expect("unexpected error occured when openning chars file");
            let mut string = String::new();
            file.read_to_string(&mut string);
            chars = Some(string);
        }else{ // string specified
            chars = Some(temp);
        }
    }

    if matches.opt_present("I"){
        let temp: String = match matches.opt_str("I"){
            Some(s) => s,
            None => {
                eprintln!("-I option expects an argument: (FLOAT,)*");
                return Err(());
            }
        };
        if dith_type == DithType::ONOFF{
            eprintln!("Illegal Combination of options: cannot specify interpolation points with OnOff ditherer.");
            return Err(());
        }
        let parts:Vec<&str> = temp.split(",").collect();
        let mut nums = Vec::<f32>::with_capacity(parts.len());
        for part in parts{
            let part = part.trim();
            let num = match part.parse::<f32>(){
                Ok(s) => s,
                Err(_) => {
                    eprintln!("error while parsin interpolating points argument. {} is not a valid float number.", part);
                    return Err(());
                }
            };
            nums.push(num);
        }
        inter_points = Some(nums);
    }

    // check for illegal combinations
    

    if matches.free.len() != 1{
        eprintln!("WRONG USAGE! See --help for more info");
        return Err(());
    }
    let temp_path = Path::new(&matches.free[0]);
    if ! temp_path.exists() {
        eprintln!("cannot open {} for reading. File does not exist.", matches.free[0]);
        return Err(());
    }
    if temp_path.exists() && !temp_path.is_file() {
        eprintln!("cannot open {} for reading. File exists and is not a regular file.", matches.free[0]);
        return Err(());
    }
    let input = matches.free[0].clone();

    Ok((out_type, seg_type, dith_type, ker_type, threshold, fmt_str, fmt_ln_str, contrast, brighten, width, height, output, chars, inter_points, input))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let map_kernel = get_kernels();
    let (
        out_type,
        seg_type,
        dith_type,
        ker_type,
        threshold,
        fmt_str,
        fmt_ln_str,
        contrast,
        brighten,
        mut width,
        mut height,
        output,
        chars,
        inter_points,
        input,
        ) = parse_args(args, &map_kernel).expect("Error while parsing input arguments. Aborting.");

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

    let kernel = map_kernel.get(ker_type.as_str()).unwrap().to_owned();

    match out_type{
        ProgType::TXT => {
            produce_txt(width, height, dyn_image, seg_type, contrast, brighten, chars, fmt_str, fmt_ln_str, dith_type, kernel, threshold, inter_points, output);
        },
        ProgType::BRAILE => {
            produce_braile(width, height, dyn_image, contrast, brighten, chars, fmt_str, fmt_ln_str, kernel, threshold, output);
        }
    }
/*
    match seg_type {
        SegType::LEGACY => {
            let stt_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale()
                .into_luma8();

            let segment_info = SegmentInfo::generate(stt_image.width(), stt_image.height(), width, height);

            generate_matrix_legacy(stt_image, &mut matrix, segment_info);
        },
        SegType::RESIZE => {

            match out_type {
                ProgType::TXT => {
                    let mut stt_image = dyn_image
                        .brighten(brighten)
                        .adjust_contrast(contrast)
                        .grayscale()
                        .resize_exact(width, height, FilterType::Gaussian)
                        .into_luma8();

                    generate_matrix(stt_image, &mut matrix);
                },
                ProgType::BRAILE => {
                    let width = if width % 2 == 0 {width} else {width + 1};
                    let height = match height % 4 {
                        0 => height,
                        1 => height + 3,
                        2 => height - 2,
                        3 => height + 1,
                        _ => unreachable!()
                    };

                    let mut matrix = Matrix::<f32>::new(width, height, 0.0);

                    let mut stt_image = dyn_image
                        .brighten(brighten)
                        .adjust_contrast(contrast)
                        .grayscale()
                        .resize_exact(width, height, FilterType::Gaussian)
                        .into_luma8();

                    generate_matrix_braile(stt_image, &mut matrix);
                    print_output(matrix, fmt_str, fmt_ln_str, chars, out_type, output);
                    return;
                }
            }
        }
    }
    print_output(matrix, fmt_str, fmt_ln_str, chars, out_type, output);
    */
}

fn produce_txt(width: u32, height: u32, dyn_image: DynamicImage, seg_type: SegType,
                  contrast:f32, brighten: i32, chars: CharsOption, fmt_str: String,
                  fmt_ln_str: String, dith_type: DithType, kernel: Kernel,
                  threshold: ThreshOption, inter_points: InterPoints, output: OutputFile){

    let mut matrix = Matrix::<f32>::new(width, height, 0.0);
    match seg_type {
        SegType::RESIZE => {
            let mut stt_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale()
                .resize_exact(width, height, FilterType::Gaussian)
                .into_luma8();

            generate_matrix(stt_image, &mut matrix);
        },
        SegType::LEGACY => {
            let stt_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale()
                .into_luma8();

            let segment_info = SegmentInfo::generate(stt_image.width(), stt_image.height(), width, height);

            generate_matrix_legacy(stt_image, &mut matrix, segment_info);
        }
    }
    let len = match chars {
        Some(ref s) => s.len(),
        None => DEFAULT_CHARS_LEN,
    };
    apply_transformation(&dith_type, kernel, threshold, inter_points, len, &mut matrix);
    print_output(matrix, fmt_str, fmt_ln_str, chars, ProgType::TXT, dith_type, output);
}

fn produce_braile(width: u32, height: u32, dyn_image: DynamicImage, contrast: f32,
                  brighten: i32, chars: CharsOption, fmt_str: String, fmt_ln_str: String,
                  kernel: Kernel, threshold: ThreshOption, output: OutputFile){
    let width = if width % 2 == 0 {width} else {width + 1};
    let height = match height % 4 {
        0 => height,
        1 => height + 3,
        2 => height - 2,
        3 => height + 1,
        _ => unreachable!()
    };

    let mut matrix = Matrix::<f32>::new(width, height, 0.0);

    let mut stt_image = dyn_image
        .brighten(brighten)
        .adjust_contrast(contrast)
        .grayscale()
        .resize_exact(width, height, FilterType::Gaussian)
        .into_luma8();

    generate_matrix(stt_image, &mut matrix);
    apply_transformation(&DithType::ONOFF, kernel, threshold, InterPoints::None, 2 /* not used but must be 2 to avoid unwanted warning */, &mut matrix);
    print_output(matrix, fmt_str, fmt_ln_str, chars, ProgType::BRAILE, DithType::ONOFF, output);
}

