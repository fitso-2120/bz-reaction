use image::{ImageBuffer, Rgb};

pub struct Graphics {
    imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl Graphics {
    pub fn new(width: usize, height: usize) -> Self {
        return Graphics {
            imgbuf: ImageBuffer::new(width as u32, height as u32),
        };
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        let pixel = self
            .imgbuf
            .get_pixel_mut(x.try_into().unwrap(), y.try_into().unwrap());
        *pixel = Rgb(rgb);
    }

    pub fn write(&self, fname: String) {
        self.imgbuf.save(fname).unwrap();
    }
}
