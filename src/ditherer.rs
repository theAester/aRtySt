
use crate::matrix::Matrix;

pub trait Ditherer{
    fn dither(&self, output: &mut Matrix<f32>);
}

pub struct OnOffKernelDitherer{
    threshold: f32,
    weights: Vec<(i32, i32, f32)>,
}

pub struct InterpolatingKernelDitherer{
    inter_points: Vec<f32>,
    mid_points: Vec<f32>,
    weights: Vec<(i32, i32, f32)>,
}

impl OnOffKernelDitherer{
    pub fn from(threshold: f32, origin: (i32, i32), factors: Matrix<f32>) -> OnOffKernelDitherer {
        let mut weights = Vec::<(i32, i32, f32)>::with_capacity((factors.get_width() * factors.get_height()).try_into().unwrap());
        for x in 0..factors.get_height(){
            for y in 0..factors.get_width(){
                let val = factors.get(x,  y).unwrap();
                weights.push( ( (x as i32) - origin.0, (y as i32) - origin.1, val ) );
            }
        }
        OnOffKernelDitherer{ threshold, weights }
    }
    pub fn new(threshold: f32, weights: Vec<(i32, i32, f32)>) -> OnOffKernelDitherer {
        OnOffKernelDitherer{ threshold, weights }
    }
}

impl InterpolatingKernelDitherer {
    pub fn from(inter_points: Vec<f32>, origin: (i32, i32), factors: Matrix<f32>) -> InterpolatingKernelDitherer {
        let mut weights = Vec::<(i32, i32, f32)>::with_capacity((factors.get_width() * factors.get_height()).try_into().unwrap());
        for x in 0..factors.get_height(){
            for y in 0..factors.get_width(){
                let val = factors.get(x,  y).unwrap();
                weights.push( ( (x as i32) - origin.0, (y as i32) - origin.1, val ) );
            }
        }
        let mut start = 0.0;
        let mut mid_points: Vec<f32> = Vec::with_capacity(inter_points.len());
        for i in 1..inter_points.len(){
            let mid = (inter_points[i] + start) / 2.0;
            mid_points.push(mid); 
            start = inter_points[i];
        }
        mid_points.push((1.0 - start) / 2.0);
        InterpolatingKernelDitherer{ inter_points, mid_points, weights }
    }
    pub fn new(inter_points: Vec<f32>, weights: Vec<(i32, i32, f32)>) -> InterpolatingKernelDitherer {
        let mut start = 0.0;
        let mut mid_points: Vec<f32> = Vec::with_capacity(inter_points.len());
        for i in 1..inter_points.len(){
            let mid = (inter_points[i] + start) / 2.0;
            mid_points.push(mid); 
            start = inter_points[i];
        }
        mid_points.push((1.0 - start) / 2.0);
        InterpolatingKernelDitherer{ inter_points, mid_points, weights }
    }
}

impl Ditherer for OnOffKernelDitherer {
    fn dither(&self, output: &mut Matrix<f32>){
        let mut input = output.clone();
        for y in 0..output.get_height(){
            for x in 0..output.get_width(){
                let val_origi = input.get(y, x).unwrap();
                let val_trans = if val_origi > self.threshold { 1.0 }else{ 0.0 };
                output.set(y, x, val_trans);
                let error = val_origi - val_trans;
                for (off_x, off_y, factor) in &self.weights{
                    if *factor == 0.0 {continue;}
                    let nx = (x as i32) + off_x;
                    let ny = (y as i32) + off_y;
                    if nx < 0 || ny < 0 {continue;}
                    let val = input.get(
                        ny.try_into().unwrap(),
                        nx.try_into().unwrap())
                        .unwrap_or(-1.0);
                    if val == -1.0 {continue;}
                    input.set(ny.try_into().unwrap(), nx.try_into().unwrap(), val + error * factor);
                }
            }
        }
    }
}

impl Ditherer for InterpolatingKernelDitherer {
    fn dither(&self, output: &mut Matrix<f32>){
        let mut input = output.clone();
        for y in 0..output.get_height(){
            for x in 0..output.get_width(){
                let val_origi = input.get(y, x).unwrap();

                let mut index = -1;
                let mut transform = 0.0;
                for i in 0..self.inter_points.len(){
                    if val_origi >= self.inter_points[i]{
                        index += 1;
                        transform = self.mid_points[i];
                        continue;
                    }
                    break;
                }

                let val_trans = transform;

                output.set(y, x, index as f32);
                let error = val_origi - val_trans;
                for (off_x, off_y, factor) in &self.weights{
                    if *factor == 0.0 {continue;}
                    let nx = (x as i32) + off_x;
                    let ny = (y as i32) + off_y;
                    if nx < 0 || ny < 0 {continue;}
                    let val = input.get(
                        ny.try_into().unwrap(),
                        nx.try_into().unwrap())
                        .unwrap_or(-1.0);
                    if val == -1.0 {continue;}
                    input.set(ny.try_into().unwrap(), nx.try_into().unwrap(), val + error * factor);
                }
            }
        }
    }
}
