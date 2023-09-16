
use std::collections::HashMap;

use crate::matrix::Matrix;


pub type KerMap = HashMap<&'static str, Kernel>;

#[derive(Clone)]
pub struct Kernel{
    pub matrix: Matrix<f32>,
    pub origin: (i32, i32),
}

//pub enum KerType{
//    NONE(Kernel),
//    FS(Kernel),
//    STUCKI(Kernel),
//    ATKINSON(Kernel),
//}

pub fn get_kernels() -> KerMap {

    let none_kernel: Kernel = Kernel{
        matrix: Matrix::<f32>::from(vec![], 0, 0),
        origin: (0,0),
    };

    let fs_kernel: Kernel = Kernel {
        matrix: Matrix::<f32>::from(vec![
            0.0,        0.0,        7.0/16.0,
            3.0/16.0,   5.0/16.0,   1.0/16.0
        ], 3, 2),
        origin: (1,0),
    };

    let stucki_kernel: Kernel = Kernel{
        matrix: Matrix::<f32>::from(vec![
            0.0,        0.0,        0.0,        8.0/42.0,   4.0/42.0,
            2.0/42.0,   4.0/42.0,   8.0/42.0,   4.0/42.0,   2.0/42.0,
            1.0/42.0,   2.0/42.0,   4.0/42.0,   2.0/42.0,   1.0/42.0
        ], 5, 3),
        origin: (2,0),
    };

    let atkinson_kernel: Kernel = Kernel{
        matrix: Matrix::<f32>::from(vec![
            0.0,	0.0,    0.125,	0.125,
            0.125,	0.125,	0.125,	0.0,
            0.0,    0.125,	0.0,    0.0
        ], 4, 3),
        origin: (1,0),
    };

    HashMap::from([
                  ("NONE", none_kernel),
                  ("FS", fs_kernel),
                  ("STUCKI", stucki_kernel),
                  ("ATKINSON", atkinson_kernel)
    ])
}


