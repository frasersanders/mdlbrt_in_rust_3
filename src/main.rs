use std::ops;
use code_timing_macros::{time_function};
use image::{self, Rgb};

const X_RESOLUTION: u32 = 2_000;
const Y_RESOLUTION: u32 = 1_001;

const X_RESOLUTION_AS_FLOAT: f64 = X_RESOLUTION as f64;
const Y_RESOLUTION_AS_FLOAT: f64 = Y_RESOLUTION as f64;

const MODE: u8 = 1;

const ESCAPE_LIMIT: u16 = 1_000;

const PI: f64 = std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
struct Complex{ re: f64, im: f64 }

impl Complex{
    fn conj (&self) -> Complex { Complex{re: self.re, im: -self.im} }
    fn add (&self, z: Complex) -> Complex { Complex{re: self.re + z.re, im: self.im + z.im} }
    fn mult (&self, z: Complex) -> Complex { 
        Complex{re: self.re * z.re - self.im * z.im , 
                im: self.re * z.im + self.im * z.re } 
    }
    fn scalar_mult (&self, a: f64) -> Complex { Complex{re: self.re * a, im: self.im * a} }
    fn squared_modulus (self) -> f64 { self.conj().mult(self).re }

    fn affine_transform(self, scale_factor: f64, translation: Complex) -> Complex {
        (self*scale_factor).add(translation)
    }

    fn escape_time_mdlbrt(self) -> Option<u16> {
        let mut i: u16 = 0;
        let mut z = Complex{re: 0.0, im: 0.0};
        while z.squared_modulus() < 4.0 && i <= ESCAPE_LIMIT {
            i += 1;
            z = z.mult(z).add(self);
        }
        match z.squared_modulus() < 4.0 {
            true => None,
            false => Some(i)
        }
    }
}

impl ops::Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mult(rhs)
    }
}

impl ops::Mul<f64> for Complex {
    type Output = Self;
    
    fn mul(self, rhs: f64) -> Self::Output {
        self.scalar_mult(rhs)
    }
}

trait Rainbow{
    fn rainbow(self) -> [u8;3];
    fn greyscale(self) -> [u8;3];
}

impl Rainbow for u16 {
    fn rainbow(self) -> [u8;3] {
        let (sin_1, sin_2, sin_3) = 
        ( ((self as f64)*0.1).sin(), 
        ((self as f64)*0.1 + (1.0/3.0 * PI)).sin(), 
        ((self as f64)*0.1 + (2.0/3.0 * PI)).sin() );
        let (sin_sq_1, sin_sq_2, sin_sq_3) = (sin_1*sin_1, sin_2*sin_2, sin_3*sin_3);
        [(255.0*sin_sq_1) as u8, (255.0*sin_sq_2) as u8, (255.0*sin_sq_3) as u8]
    }
    fn greyscale(self) -> [u8;3] {
            [((self*10) % 255) as u8, 
            ((self*10) % 255) as u8, 
            ((self*10) % 255) as u8]
    }
}

trait RgbFromInt{
    fn rgb(&self) -> [u8;3];
}

impl RgbFromInt for Option<u16> {
    fn rgb(&self) -> [u8;3] {
        match &self {
            None => [0,0,0],
            Some(i) => match MODE {
                0 => i.greyscale(),
                1 => i.rainbow(),
                _ => [0,0,0]
            }
        }
    }
}

trait ToComplex{
    fn normalise_in_x(self) -> Complex;
    fn get_rgb_value(self, scale_factor: f64, translation: Complex) -> [u8;3];
}
impl ToComplex for (u32, u32){
    fn normalise_in_x (self) -> Complex {
        let (x, y) = self;
        Complex{ 
            re: (2.0 * (x as f64) - X_RESOLUTION_AS_FLOAT) /X_RESOLUTION_AS_FLOAT , 
            im: (Y_RESOLUTION_AS_FLOAT - 2.0 * (y as f64)) /X_RESOLUTION_AS_FLOAT 
        }
    }

    fn get_rgb_value(self, scale_factor: f64, translation: Complex) -> [u8;3] {
        let mut z = self.normalise_in_x();
        z = z.affine_transform(scale_factor, translation);
        z.escape_time_mdlbrt().rgb()
    }
    
}

#[time_function]
fn main() {
    let a: f64 = -0.03942862882707475;
    let b: f64 = -0.9880027977017277;
    let translation = Complex{re: a, im: b};

    let scale_factor: f64 = 1.0/100.0 ;

    //let image_buffer: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(X_RESOLUTION, Y_RESOLUTION, |x, y | (x,y).get_rgb_value(scale_factor, translation));

    let mut pixels_vec: Vec<u8> = Vec::new();
    for y in 0..Y_RESOLUTION{
        for x in 0..X_RESOLUTION{
            let x: [u8;3] = (x,y).get_rgb_value(scale_factor, translation);
            pixels_vec.push(x[0]);
            pixels_vec.push(x[1]);
            pixels_vec.push(x[2]);
        }
    }
    let image_buffer:image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_raw(Y_RESOLUTION, X_RESOLUTION, pixels_vec).expect("oops");
    let _ = image_buffer.save("images/mdlbrt.png");
}