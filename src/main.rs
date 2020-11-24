// #[macro_use]
extern crate ndarray;

use image::ImageBuffer;
// use image::GrayImage;
use ndarray::prelude::*;
use ndarray_stats::QuantileExt;
use image::{DynamicImage, Luma, imageops};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
struct Particle {
    x: f64,
    y: f64,
    heading: f64,
    sensor_angle: f64,
    sensor_width: i32,
    sensor_offset: f64
}

impl Particle {
    fn step<R: Rng>(&mut self, arr: & Array2<f64>, rng: &mut R) -> (usize, usize) {

    let mut xc = self.x + self.sensor_offset * self.heading.cos();
    let mut yc = self.y + self.sensor_offset * self.heading.sin();

    xc = xc.round() % 1024_f64;
    yc = yc.round() % 1024_f64;


    let mut xl = self.x + self.sensor_offset * (self.heading - self.sensor_angle).cos();
    let mut yl = self.y + self.sensor_offset * (self.heading - self.sensor_angle).sin();
    
    xl = xl.round() % 1024_f64;
    yl = yl.round() % 1024_f64;

    let mut xr = self.x + self.sensor_offset * (self.heading + self.sensor_angle).cos();
    let mut yr = self.y + self.sensor_offset * (self.heading + self.sensor_angle).sin();

    xr = xr.round() % 1024_f64;
    yr = yr.round() % 1024_f64;

    // let pixel = arr.get_pixel(xc as u32, yc as u32);
    let c = arr[[xc as usize, yc as usize]];//pixel[0];

    // let pixel = arr.get_pixel(xl as u32, yl.round() as u32);
    let l = arr[[xl as usize, yl as usize]];//pixel[0];

    // let pixel = arr.get_pixel(xr as u32, yr as u32);
    let r = arr[[xr as usize, yr as usize]];//pixel[0];

    if c < l && c < r {
        //random
        if rng.gen::<f64>() > 0.5 {
            self.heading -= self.sensor_angle;
        } else {
            self.heading += self.sensor_angle;
        }
    } else if l < r {
        // move right
        self.heading += self.sensor_angle;
    } else if r < l {
        //move left
        self.heading -= self.sensor_angle;
    }

    let (x, y) = move_particle(&self);

    self.x = x;
    self.y = y;

    ((self.x.round() % 1024_f64) as usize, (self.y.round() % 1024_f64) as usize)
    }
}

pub type GrayImage = ImageBuffer<Luma<u16>, Vec<u16>>;


fn main() {
    let mut particles: Vec<Particle> = Vec::new();
    let mut rng = rand::thread_rng();

    let xsize = 1024;
    let ysize = 1024;

    let mut trail = Array::<f64, _>::from_elem((xsize, ysize), 0.0_f64);

    for _p in 1..=1_000_00 {
        let some = Particle {x: 512.0, y: 512.0, heading: 1.5 * std::f64::consts::PI * rng.gen::<f64>(), sensor_angle: 45.0_f64.to_radians(), sensor_width: 1, sensor_offset: 9.0_f64};
        particles.push(some);
        let x = *&some.x as usize;
        let y = *&some.y as usize;
        trail[[x, y]] += 5.
    }

    for _i in 1..=5000 {
        println!("{:?}", _i);
        let mut list: Vec<(usize, usize)> = Vec::new();
        for p in particles.iter_mut() {
            list.push(p.step(&trail, &mut rng));
        }
        for item in list.iter() {
            let (x, y) = item;
            trail[[*x, *y]] += 5.;
        }
        trail *= 0.8;
        let imgbuf = array_to_image(&trail);
        // imgbuf = imageops::filter3x3(&imgbuf, &[3.0]);
        // imgbuf = decay(&imgbuf, 0.8);
        // image_to_array(&imgbuf);
// 
        let s = format!("imgs/{:03}.png", _i);
        imgbuf.save(s).unwrap();
    }

}


fn image_to_array(arr: &GrayImage) -> Array2<u16> {

    // let mut out = Array::<f64, _>::from_elem((arr.width() as usize, arr.height() as usize), 0.0_f64);

    // let tmp = arr.into_raw();
    // println!("{:?}", tmp);
    let out = Array::from_shape_vec((arr.width() as usize, arr.height() as usize), arr.as_raw().to_vec()).unwrap();

    // for y in 0..arr.height() {
    //     for x in 0..arr.width() {
    //         let pixel = arr.get_pixel(x, y);
    //         out[[x as usize, y as usize]] = pixel[0];
    //     }
    // }
    // out.swap_axes(0, 1);
    out.as_standard_layout().to_owned()

}


fn array_to_image(arr: & Array2<f64>) -> GrayImage{
    assert!(arr.is_standard_layout());

    let (height, width) = arr.dim();
    let maxval = *arr.max().unwrap() as f64;
    let tmp = (65535_f64*arr / maxval).mapv(|elem| elem as u16);
    let raw = tmp.into_raw_vec();

    GrayImage::from_raw(width as u32, height as u32, raw)
        .expect("container should have the right size for the image dimensions")
}

fn decay(image: &GrayImage, fact: f64) -> GrayImage {
    
    let mut out = DynamicImage::new_luma16(image.width(), image.height()).as_luma16().unwrap().to_owned();

    for y in 0..image.height() {
        for x in 0..image.width(){
            let current_pixel = image.get_pixel(x, y);
            let tmp = current_pixel[0] as f64 * fact;
            out.put_pixel(x, y, Luma([tmp as u16]));
        }
    }

    out
}

// fn scale(image: &GrayImage) -> GrayImage {
    
//     let mut out = DynamicImage::from_pixel(image.width(), image.height(), Luma([0_u16]));

//     let mut max_pixel = 0_u16;
//     for y in 0..image.height() {
//         for x in 0..image.width(){
//             let current_pixel = image.get_pixel(x, y);
//             if max_pixel < current_pixel[0] {
//                 max_pixel = current_pixel[0];
//             }
//         }
//     }

//     for y in 0..image.height() {
//         for x in 0..image.width(){
//             let current_pixel = image.get_pixel(x, y);
//             let fact = (65535_f64 * current_pixel[0] as f64) / max_pixel as f64;
//             out.put_pixel(x, y, Luma([fact as u16]));
//         }
//     }

//     out
// }

fn move_particle(p: &Particle) -> (f64, f64) {

    let xd = p.heading.cos();
    let yd = p.heading.sin();

    let mut new_x = p.x + xd;
    let mut new_y = p.y + yd;

    if new_x > 1024_f64 {
        new_x -= 1024_f64;
    } else if new_x < 0_f64 {
        new_x += 1024_f64;
    }

    if new_y > 1024_f64 {
        new_y -= 1024_f64;
    } else if new_y < 0_f64 {
        new_y += 1024_f64;
    }

    return (new_x, new_y);
}
