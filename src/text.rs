use std::fs::File;
use std::io::{Read, Write};
use crate::matrix::Matrix;

pub fn print_output(matrix: Matrix<f32>, fmt_str: String, fmt_ln_str: String, chars: Option<File>, output: Option<File>){
    // array of characters, arranged in increasing brightness
    let char_array: Vec<char> = match chars{
        Some(mut f)=>{
            let mut temp = String::new();
            f.read_to_string(&mut temp).unwrap();
            let temp = temp.trim().replace("\n", "");
            temp.chars().collect()
        },
        None => {
            Vec::<char>::from(DEFAULT_CHARS)
        }
    };
    println!("{:?}", char_array);
    // output buffer
    let mut output_buff = String::new();
    // cast all matrix entries to characters, then format them and add to buffer
    for i in 0..matrix.get_height(){
        for j in 0..matrix.get_width(){
            let val = matrix.get(i, j).unwrap();
            let index = (val * (char_array.len() as f32)).floor() as usize;
            let index = if index == char_array.len() {char_array.len() - 1} else {index};
            let out_char = char_array[index];
            output_buff.push(out_char);
            output_buff.push(out_char);
        }
        output_buff.push('\n');
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
