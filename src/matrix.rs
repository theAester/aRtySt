#[derive(Clone)]
pub struct Matrix<T>
    where T: Copy{
    storage: Vec<T>,
    width: u32,
    height: u32,
}

impl<T: std::marker::Copy> Matrix<T>{
    pub fn new(width: u32, height: u32, filler: T) -> Matrix<T> {
        Matrix{
            storage: vec![filler; (width * height).try_into().unwrap()],
            width:width,
            height:height,
        }
    }

    pub fn from(input: Vec<T>, width:u32, height:u32) -> Matrix<T>{
        if input.len() != (width * height).try_into().unwrap() {
            panic!("BAD INPUT. sizes dont match.");
        }
        Matrix{
            storage: input,
            width: width,
            height: height,
        }
    }

    pub fn get(&self, i: u32, j: u32) -> Result<T, String>{
        if i >= self.height ||
            j >= self.width {
                return Err(format!("({},{}) is out of range for matrix({},{})", i, j, self.height, self.width));
            }
        let index: usize = (i*self.width + j).try_into().unwrap();
        Ok(self.storage[index])
    }

    pub fn set(&mut self, i: u32, j: u32, val: T) -> Result<(), String>{
        if i >= self.height ||
            j >= self.width  {
                return Err(format!("({},{}) is out of range for matrix({},{})", i, j, self.height, self.width));
            }
        let index: usize = (i*self.width + j).try_into().unwrap();
        self.storage[index] = val;
        Ok(())
    }

    pub fn get_width(&self) -> u32 { self.width }
    pub fn get_height(&self) -> u32 { self.height }
}

