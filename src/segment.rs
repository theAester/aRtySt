pub struct SegmentInfo{
    // unused
    //in_width: u32,
    //in_height: u32,
    out_width: u32,
    out_height: u32,
    w_quotient: u32, 
    w_remainder: u32,
    h_quotient: u32,
    h_remainder: u32,
}

impl SegmentInfo{
    pub fn generate(in_width: u32, in_height: u32, out_width: u32, out_height: u32) -> SegmentInfo {
        let w_quotient = in_width / out_width;
        let w_remainder = in_width % out_width;

        let h_quotient = in_height / out_height;
        let h_remainder = in_height % out_height;

        SegmentInfo{
            // unused
            //in_width: in_width,
            //in_height: in_height,
            out_width: out_width,
            out_height: out_height,
            w_quotient: w_quotient,
            w_remainder: w_remainder,
            h_quotient: h_quotient,
            h_remainder: h_remainder,
        }
    }
    pub fn get_block_dims(&self, i: u32, j: u32) -> (u32, u32) {
        let mut w = self.w_quotient;
        let mut h = self.h_quotient;
        if i < self.h_remainder {
            h += 1;
        }
        if j < self.w_remainder {
            w += 1;
        }
        (w, h)
    }
    pub fn get_block_start_index(&self, i: u32, j: u32) -> (u32, u32){
        (self.get_j_index(j), self.get_i_index(i))
    }
    fn get_i_index(&self, i: u32) -> u32{
        if i < self.h_remainder {
            return i * (self.h_quotient + 1);
        }else{
            return self.h_remainder * (self.h_quotient + 1) + (i - self.h_remainder) * self.h_quotient;
        }
    }
    fn get_j_index(&self, j: u32) -> u32{
        if j < self.w_remainder {
            return j * (self.w_quotient + 1);
        }else{
            return self.w_remainder * (self.w_quotient + 1) + (j - self.w_remainder) * self.w_quotient;
        }
    }
    pub fn get_width(&self) -> u32 { self.out_width }
    pub fn get_height(&self) -> u32 { self.out_height }
    // unused
    //pub fn get_img_width(&self) -> u32 { self.in_width }
    //pub fn get_img_height(&self) -> u32 { self.in_height }
}
