use std::fs::File;
use std::io::{Write};
use crate::matrix::Matrix;
use crate::{ProgType, CharsOption, DithType};

pub const DEFAULT_CHARS_LEN: usize = 39;
const DEFAULT_CHARS: [char; DEFAULT_CHARS_LEN] = [' ','.','`','\'','-','~','+','^',':',';','>','<','?',')','(','|',']','[','}','{','\\','/',
                                                  'i','1','l','L','0','O','m','q','d','k','#','W','%','&','B','@','$'];

pub fn print_output(matrix: Matrix<f32>, _fmt_str: String, _fmt_ln_str: String, chars: CharsOption, out_type: ProgType, dith_type: DithType, output: Option<File>){
    // array of characters, arranged in increasing brightness
    let char_array: Vec<char> = match chars{
        Some(s) => {
            let mut starts_with_space = false;
            if s.chars().nth(0).unwrap() == ' ' {starts_with_space = true;}
            let temp = s.trim().replace("\n", "");
            let mut temp: Vec<char> = temp.chars().collect();
            if starts_with_space {
                temp.insert(0, ' ');
            }
            temp
        }
        None => {
            Vec::<char>::from(DEFAULT_CHARS)
        }
    };
    // output buffer
    let mut output_buff = String::new();

    match out_type{
        ProgType::TXT => {
            produce_buffer_txt(matrix, char_array, dith_type, &mut output_buff);
        },
        ProgType::BRAILE => {
            produce_buffer_braile(matrix, &mut output_buff);
        }
    }

    // select output and write
    match output {
        Some(mut f) => {
            write!(f, "{}", output_buff).unwrap();
        },
        None => {
            println!("{}", output_buff);
        }
    };
}

fn produce_buffer_txt(matrix: Matrix<f32>, char_array: Vec<char>, dith_type: DithType, output_buff: &mut String) {
    // cast all matrix entries to characters, then format them and add to buffer
    for i in 0..matrix.get_height(){
        for j in 0..matrix.get_width(){
            let val = matrix.get(i, j).unwrap();
            let index: usize = match dith_type {
                DithType::ONOFF => {
                    (val * (char_array.len() as f32)).floor() as usize
                },
                DithType::INTER => {
                    val as usize 
                }
            };
            let index = if index == char_array.len() {char_array.len() - 1} else {index};
            let out_char = char_array[index];
            output_buff.push(out_char);
            output_buff.push(out_char);
        }
        output_buff.push('\n');
    }
}

fn produce_buffer_braile(matrix: Matrix<f32>, output_buff: &mut String) {
    let lx = matrix.get_width() / 2;
    let ly = matrix.get_height() / 4;
    for i in 0..ly{
        for j in 0..lx{
            let mut charnum: u32= 10240;
            let passes = [(0,0,0), (0,1,1), (0,2,2), (1,0,3), (1,1,4), (1,2,5), (0,3,6), (1,3,7)];
            for (dx, dy, shift) in passes{
                charnum += (matrix.get(4*i + dy , 2*j + dx).unwrap() as u32) << shift;
            }
            output_buff.push(char::from_u32(charnum).unwrap());
        }
        output_buff.push('\n');
    }
}
