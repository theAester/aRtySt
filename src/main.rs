extern crate getopts;

use getopts::{Options, Occur, HasArg};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use imageproc::image::io::Reader as ImageReader;
use imageproc::image::imageops::FilterType;
use imageproc::image::DynamicImage;
use imageproc::filter::sharpen_gaussian;

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

#[macro_export]
macro_rules! meprintln {
    ($($x:tt)*) => {
        eprint!("\x1b[31m");
        eprint!($($x)*);
        eprintln!("\x1b[0m");
    }
}

fn print_help(progname: String, parser: Options){
    println!("HDS aRtySt v{}\nGNU GPL 3.0 license.\n\n", env!("CARGO_PKG_VERSION"));
    println!("{}\n\n{}\n  NOTE: character and line formatting are not implemented yet.\n  NOTE: HTML output format is not implemented yet.", parser.short_usage(&progname), parser.usage(PROGDESC));
}

fn parse_args(args: Vec<String>, map_kernel: &KerMap) -> Result<(
                                            ProgType,
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
                                            f32,
                                            f32,
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
    parser.opt("C", "chars", "list of characters to use as output", "STRING|@FILENAME", HasArg::Yes, Occur::Optional);
    parser.opt("I", "inter-points", "interpolation points", "(FLOAT,)*|@FILENAME", HasArg::Yes, Occur::Optional);
    parser.opt("G", "gaussian", "apply a gaussian filter.", "FLOAT", HasArg::Yes, Occur::Optional);
    parser.opt("S", "sharpen", "use sharpen to emphasize on the edges on the image. best used along with -G. for this option to have any effect, -G value must be negative.", "FLOAT>0", HasArg::Yes, Occur::Optional);

    let matches = match parser.parse(args[1..].into_iter()) {
        Ok(s) => s,
        Err(e)=>{
            meprintln!("Error: {}", e);
            return Err(());
        }
    };

    let mut out_type: ProgType = ProgType::TXT;
    let mut seg_type: SegType = SegType::LEGACY;
    let mut dith_type: DithType = DithType::INTER;
    let mut ker_type: String = String::from("NONE"); 
    let mut threshold: ThreshOption = None;
    let /* mut */ fmt_str: String = String::from("{}");
    let /* mut */ fmt_ln_str: String = String::from("{}\n");
    let mut contrast: f32 = 0.0;
    let mut brighten: i32 = 0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut gaussian: f32 = 0.0;
    let mut sharpen: f32 = 0.0;
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
                meprintln!("-t option expects an argument: TXT|BRAILE");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "txt" { out_type = ProgType::TXT; }
        else if temp == "braile" { out_type = ProgType::BRAILE; }
        else {
            meprintln!("-t option expects an argument: TXT|BRAILE");
            return Err(());
        }
    }


    if matches.opt_present("d"){
        let temp: String = match matches.opt_str("d"){
            Some(s) => s,
            None => {
                meprintln!("-d option expects an argument: ONOFF|INTERPOLATING");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "onoff" { dith_type = DithType::ONOFF; }
        else if temp == "inter" || temp == "interpolating" { 
            if out_type == ProgType::BRAILE{
                meprintln!("Illegal Combination of options: cannot use interpolating ditherer for braile output.");
                return Err(());
            }
            dith_type = DithType::INTER; 
        }
        else {
            meprintln!("-d option expects an argument: ONOFF|INTERPOLATING");
            return Err(());
        }
    }

    if matches.opt_present("k"){
        let temp: String = match matches.opt_str("k"){
            Some(s) => s,
            None => {
                meprintln!("-k option expects an argument: {}", help_kernel_types);
                return Err(());
            }
        }.trim().to_lowercase();
        if map_kernel.contains_key(temp.to_uppercase().as_str()) {
            ker_type = temp.to_uppercase();
        }else{
            meprintln!("-k option expects an argument: {}", help_kernel_types);
            return Err(());
        }
    }

    if matches.opt_present("T"){
        let temp = match matches.opt_str("T"){
            Some(s) => s,
            None => {
                meprintln!("-T option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        threshold = match temp {
            Ok(s) => ThreshOption::Some(s),
            Err(_) => {
                meprintln!("the argument given to -T is not a valid FLOAT number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("f"){
        meprintln!("Formatting features are not implemented yet. Ignoring this option: -f");
    }
    if matches.opt_present("F"){
        meprintln!("Formatting features are not implemented yet. Ignoring this option: -F");
    }

    if matches.opt_present("c"){
        let temp = match matches.opt_str("c"){
            Some(s) => s,
            None => {
                meprintln!("-c option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        contrast = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -c is not a valid FLOAT number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("b"){
        let temp = match matches.opt_str("b"){
            Some(s) => s,
            None => {
                meprintln!("-b option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<i32>();

        brighten = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("W"){
        let temp = match matches.opt_str("W"){
            Some(s) => s,
            None => {
                meprintln!("-W option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        width = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -W is not a valid INTEGER.");
                return Err(());
            }
        };
    }

    if matches.opt_present("H"){
        let temp = match matches.opt_str("H"){
            Some(s) => s,
            None => {
                meprintln!("-H option expects an argument: INTEGER");
                return Err(());
            }
        }.parse::<u32>();

        height = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -H is not a valid INTEGER number.");
                return Err(());
            }
        };
    }

    if matches.opt_present("o"){
        let temp: String = match matches.opt_str("o"){
            Some(s) => s,
            None => {
                meprintln!("-o option expects an argument: FILENAME");
                return Err(());
            }
        };
        let temp_path = Path::new(&temp);
        if temp_path.exists() && !temp_path.is_file() {
            meprintln!("cannot open {} for writing. File exists and is not a regular file.", temp);
            return Err(());
        }
        output = Some(File::options().write(true).append(false).create(true).open(temp).expect("unexpected error occured when openning output file"));
    }

    if matches.opt_present("C"){
        let temp: String = match matches.opt_str("C"){
            Some(s) => s,
            None => {
                meprintln!("-C option expects an argument: STRING|@FILENAME");
                return Err(());
            }
        };
        // file specified
        if temp.chars().nth(0).unwrap() == '@' {
            let temp_path = Path::new(&temp);
            if !temp_path.exists() {
                meprintln!("cannot open {} for reading: File does not exist.", temp);
                return Err(());
            }
            if temp_path.exists() && !temp_path.is_file() {
                meprintln!("cannot open {} for reading: File exists and is not a regular file.", temp);
                return Err(());
            }
            let mut file = File::options().read(true).open(temp).expect("unexpected error occured when openning chars file");
            let mut string = String::new();
            file.read_to_string(&mut string).unwrap();
            chars = Some(string);
        }else{ // string specified
            chars = Some(temp);
        }
    }

    if matches.opt_present("I"){
        let mut temp: String = match matches.opt_str("I"){
            Some(s) => s,
            None => {
                meprintln!("-I option expects an argument: (FLOAT,)*|@FILENAME");
                return Err(());
            }
        };
        if dith_type == DithType::ONOFF{
            meprintln!("Illegal Combination of options: cannot specify interpolation points with OnOff ditherer.");
            return Err(());
        }
        if temp.chars().nth(0).unwrap() == '@' {
            let temp_path = Path::new(&temp);
            if !temp_path.exists() {
                meprintln!("cannot open {} for reading: File does not exist.", temp);
                return Err(());
            }
            if temp_path.exists() && !temp_path.is_file() {
                meprintln!("cannot open {} for reading: File exists and is not a regular file.", temp);
                return Err(());
            }
            let mut file = File::options().read(true).open(temp).expect("unexpected error occured when openning chars file");
            let mut string = String::new();
            file.read_to_string(&mut string).unwrap();
            temp = string;
        }
        let parts:Vec<&str> = temp.split(",").collect();
        let mut nums = Vec::<f32>::with_capacity(parts.len());
        let mut previous = -0.00000000000001;
        for part in parts{
            let part = part.trim();
            let num = match part.parse::<f32>(){
                Ok(s) => {
                    if s < previous {
                        meprintln!("Illegal Argument: the list of numbers provided to -I must be in increasing order");
                        return Err(());
                    }
                    if s < 0.0 || s > 1.0 {
                        meprintln!("Illegal argument: the list on numbers provided to -I must contain only numbers between 0 and 1");
                        return Err(());
                    }
                    previous = s;
                    s
                },
                Err(_) => {
                    meprintln!("error while parsin interpolating points argument. {} is not a valid float number.", part);
                    return Err(());
                }
            };
            nums.push(num);
        }
        inter_points = Some(nums);
    }
    if matches.opt_present("G"){
        let temp = match matches.opt_str("G"){
            Some(s) => s,
            None => {
                meprintln!("-G option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        gaussian = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -G is not a valid FLOAT number.");
                return Err(());
            }
        };
    }
    if matches.opt_present("S"){
        let temp = match matches.opt_str("S"){
            Some(s) => s,
            None => {
                meprintln!("-S option expects an argument: FLOAT");
                return Err(());
            }
        }.parse::<f32>();

        sharpen = match temp {
            Ok(s) => s,
            Err(_) => {
                meprintln!("the argument given to -S is not a valid FLOAT number.");
                return Err(());
            }
        };
        if sharpen != 0.0 && gaussian > 0.0 {
            meprintln!("WARNING: in order for --sharpen to have any effect, the argument to --gaussian must be zero or negative. The value for --sharpen is ignored");
            sharpen = 0.0;
        }
    }

    if matches.opt_present("s"){
        let temp: String = match matches.opt_str("s"){
            Some(s) => s,
            None => {
                meprintln!("-s option expects an argument: RESIZE|LEGACY");
                return Err(());
            }
        }.trim().to_lowercase();
        if temp == "resize" { seg_type = SegType::RESIZE; }
        else if temp == "legacy" { 
            if out_type == ProgType::BRAILE{
                meprintln!("Illegal Combination of options: cannot use Legacy segmentation for braile output.");
                return Err(());
            }
            if gaussian != 0.0 {
                meprintln!("WARNING: gaussian filter is not compatible with legacy segmentation. the given value will be ignored.");
                gaussian = 0.0;
            }
            if sharpen != 0.0 {
                meprintln!("WARNING: sharpening is not compatible with legacy segmentation. the given value will be ignored.");
                sharpen = 0.0;
            }
            seg_type = SegType::LEGACY;
        }
        else {
            meprintln!("-s option expects an argument: RESIZE|LEGACY");
            return Err(());
        }
    }

    // check for illegal combinations
    

    if matches.free.len() != 1{
        meprintln!("You need to specify the input image file name");
        return Err(());
    }
    let temp_path = Path::new(&matches.free[0]);
    if ! temp_path.exists() {
        meprintln!("cannot open {} for reading. File does not exist.", matches.free[0]);
        return Err(());
    }
    if temp_path.exists() && !temp_path.is_file() {
        meprintln!("cannot open {} for reading. File exists and is not a regular file.", matches.free[0]);
        return Err(());
    }
    let input = matches.free[0].clone();

    Ok((out_type, seg_type, dith_type, ker_type, threshold, fmt_str, fmt_ln_str, contrast, brighten, width, height, output, chars, inter_points, gaussian, sharpen, input))
}

fn main() {

    let args: Vec<String> = std::env::args().collect();
    let map_kernel = get_kernels();
    let parsed = parse_args(args, &map_kernel);
    if parsed.is_err() {
            println!("An Error occured when parsing input arguments.");
            println!("See --help for more info");
            println!("Aborting...");
            return;
    }
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
        gaussian,
        sharpen,
        input,
        ) = parsed.expect("All error cases have been checked");

    if width == 0 && height == 0 {
        meprintln!("Input error: atleast one of --width or --height options must be specified.");
        eprintln!("see --help for more info.");
        eprintln!("Aborting...");
        return;
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
            produce_txt(width, height, dyn_image, seg_type, contrast, brighten, chars, fmt_str, fmt_ln_str, dith_type, kernel, threshold, inter_points, gaussian, sharpen, output);
        },
        ProgType::BRAILE => {
            produce_braile(width, height, dyn_image, contrast, brighten, chars, fmt_str, fmt_ln_str, kernel, threshold, gaussian, sharpen, output);
        }
    }
}

fn produce_txt(width: u32, height: u32, dyn_image: DynamicImage, seg_type: SegType,
                  contrast:f32, brighten: i32, chars: CharsOption, fmt_str: String,
                  fmt_ln_str: String, dith_type: DithType, kernel: Kernel,
                  threshold: ThreshOption, inter_points: InterPoints, gaussian: f32, 
                  sharpen: f32, output: OutputFile){

    let mut matrix = Matrix::<f32>::new(width, height, 0.0);
    match seg_type {
        SegType::RESIZE => {
            let dyn_image = dyn_image
                .brighten(brighten)
                .adjust_contrast(contrast)
                .grayscale();

            let stt_image = if gaussian == 0.0 && sharpen == 0.0{
                dyn_image .resize_exact(width, height, FilterType::Gaussian)
                    .into_luma8()
            }else if gaussian > 0.0 {
                dyn_image.blur(gaussian)
                    .resize_exact(width, height, FilterType::Gaussian)
                    .into_luma8()
            }else {
                let stt = dyn_image.resize_exact(width, height, FilterType::Gaussian).into_luma8();
                sharpen_gaussian(&stt, -gaussian, sharpen)
            };

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
                  kernel: Kernel, threshold: ThreshOption, gaussian:f32,
                  sharpen:f32, output: OutputFile){
    let width = if width % 2 == 0 {width} else {width + 1};
    let height = match height % 4 {
        0 => height,
        1 => height + 3,
        2 => height - 2,
        3 => height + 1,
        _ => unreachable!()
    };

    let mut matrix = Matrix::<f32>::new(width, height, 0.0);

    let dyn_image = dyn_image
        .brighten(brighten)
        .adjust_contrast(contrast)
        .grayscale();

    let stt_image = if gaussian == 0.0 && sharpen == 0.0{
        dyn_image .resize_exact(width, height, FilterType::Gaussian)
            .into_luma8()
    }else if gaussian > 0.0 {
        dyn_image.blur(gaussian)
            .resize_exact(width, height, FilterType::Gaussian)
            .into_luma8()
    }else {
        let stt = dyn_image.resize_exact(width, height, FilterType::Gaussian).into_luma8();
        sharpen_gaussian(&stt, -gaussian, sharpen)
    };

    generate_matrix(stt_image, &mut matrix);
    apply_transformation(&DithType::ONOFF, kernel, threshold, InterPoints::None, 2 /* not used but must be 2 to avoid unwanted warning */, &mut matrix);
    print_output(matrix, fmt_str, fmt_ln_str, chars, ProgType::BRAILE, DithType::ONOFF, output);
}

