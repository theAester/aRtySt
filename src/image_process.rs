
use image::{GrayImage};
use crate::segment::SegmentInfo;
use crate::matrix::Matrix;
use crate::kernel::*;
use crate::ditherer::*;
use crate::{DithType, InterPoints, ThreshOption};

////////// LEGACY ///////////

pub fn generate_matrix_legacy(image: GrayImage, matrix: &mut Matrix<f32>, segment_info: SegmentInfo){
    for i in 0..segment_info.get_height(){
        for j in 0..segment_info.get_width(){
            let (block_width, block_height) = segment_info.get_block_dims(i, j);
            let (x_index, y_index) = segment_info.get_block_start_index(i, j);
            let value = take_average(&image, x_index, x_index + block_width, y_index, y_index + block_height);
            matrix.set(i, j, value).unwrap();
        }
    }
}

pub fn take_average(image: &GrayImage, x1: u32, x2: u32, y1: u32, y2: u32) -> f32{
    let mut val: f32 = 0.0;
    let mut cnt: u32 = 0;
    for i in x1..x2{
        for j in y1..y2{
            val += image.get_pixel(i, j).0[0] as f32;
            cnt += 1;
        }
    }
    let temp = val / (cnt as f32);
    return temp / 255.0; // a number between 0 and 1
}

////////// KERNEL ///////////


pub fn generate_matrix(image: GrayImage, matrix: &mut Matrix<f32>){
    for x in 0..image.width(){
        for y in 0..image.height(){
            let _ = matrix.set(y, x, (image.get_pixel(x, y).0[0] as f32)/255.0);
        }
    }
}

pub fn apply_transformation(dith_type: &DithType, kernel: Kernel, threshold: ThreshOption,
                            inter_points: InterPoints, chars_cnt: usize, matrix: &mut Matrix<f32>){

    match dith_type {
        DithType::INTER => {
            match inter_points{
                Some(s) => {
                    if chars_cnt > s.len(){
                        eprintln!("WARNING: There are more characters in the char sequence than there are interpolation points specified. This can result in unexpectedly low output quality.");
                    }
                    let ditherer = InterpolatingKernelDitherer::from(s, kernel.origin, kernel.matrix);
                    ditherer.dither(matrix);
                },
                None => {
                    let threshold = match threshold {
                        Some(s) => s,
                        None => {
                            1.0 / (chars_cnt as f32)
                        }
                    };

                    let space = 1.0 - threshold;
                    let parts = space / ((chars_cnt - 1) as f32);
                    let mut inters: Vec<f32> = Vec::with_capacity(chars_cnt);
                    inters.push(0.0);
                    inters.push(threshold);
                    for i in 1..(chars_cnt-1){
                        inters.push(threshold + (i as f32) * parts); 
                    }
                    let ditherer = InterpolatingKernelDitherer::from(inters, kernel.origin, kernel.matrix);
                    ditherer.dither(matrix);
                }
            }
        },
        DithType::ONOFF => {
            if chars_cnt != 2 {
                eprintln!("WARNING: ONOFF ditherer specified but more than 2 characters have been specified. This means that only the first and last characters in the character sequence will be used.");
            }
            let threshold = match threshold {
                Some(s) => s,
                None => {
                    eprintln!("WARNING: You should specify a threshold when using an ONOFF ditherer. Threshold=0.5 is assumed.");
                    0.5
                }
            };

            let ditherer = OnOffKernelDitherer::from(threshold, kernel.origin, kernel.matrix);
            ditherer.dither(matrix);
        }
    }
    
}


// unused
//pub fn generate_matrix_braile(mut image: GrayImage, matrix: &mut Matrix<f32>){
//    for x in 0..image.width(){
//        for y in 0..image.height(){
//            matrix.set(y, x, (image.get_pixel(x, y).0[0] as f32)/255.0);
//        }
//    }
//    let kernel = Matrix::<f32>::from(vec![
//		0.0,	0.0,    0.125,	0.125,
//		0.125,	0.125,	0.125,	0.0,
//		0.0,    0.125,	0.0,    0.0
//    ], 4, 3);
//    let atkinson_ditherer = OnOffKernelDitherer::from(0.4, (1,0), kernel);
//    atkinson_ditherer.dither(matrix);
//    //for i in 0..matrix.get_height(){
//    //    for j in 0..matrix.get_width(){
//    //        print!("{}", matrix.get(i, j).unwrap());
//    //    }
//    //    println!();
//    //}
//}
