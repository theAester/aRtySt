
use image::{GrayImage};
use crate::segment::SegmentInfo;
use crate::matrix::Matrix;

/*
pub fn generate_matrix(image: GrayImage, matrix: &mut Matrix<f32>, segment_info: SegmentInfo){
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
*/

pub fn generate_matrix(image: GrayImage, matrix: &mut Matrix<f32>){
    for x in 0..image.width(){
        for y in 0..image.height(){
            matrix.set(y, x, (image.get_pixel(x, y).0[0] as f32)/255.0);
        }
    }
}
