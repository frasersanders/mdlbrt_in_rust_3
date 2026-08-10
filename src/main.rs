use std::{ops, println};
use code_timing_macros::{time_function};
use image::{self, Rgb};
use datetime;

const PI: f64 = std::f64::consts::PI;

#[derive(Clone, Copy)]
struct Parameters {
    x_res: u32,
    y_res: u32,
    translation: Complex,
    scale_factor: f64,
    escape_limit: u16,
    colour_mode: u8
}

trait Default {
    fn default() -> Self;
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters{
            x_res: 100,
            y_res: 100,
            translation: Complex{re: 0.0, im: 0.0},
            scale_factor: 1.0,
            escape_limit: 2_000,
            colour_mode: 1,
        }
    
    }
}

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

    fn escape_time_mdlbrt(self, escape_limit: u16) -> Option<u16> {
        let mut i: u16 = 0;
        let mut z = Complex{re: 0.0, im: 0.0};
        while z.squared_modulus() < 4.0 && i <= escape_limit {
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
    fn rgb(&self, colour_mode: u8) -> [u8;3];
}

impl RgbFromInt for Option<u16> {
    fn rgb(&self, colour_mode: u8) -> [u8;3] {
        match &self {
            None => [0,0,0],
            Some(i) => match colour_mode {
                0 => i.greyscale(),
                1 => i.rainbow(),
                _ => [0,0,0]
            }
        }
    }
}

trait ToComplex{
    fn normalise_in_x(self, x_res:u32, y_res: u32) -> Complex;
    fn get_rgb_value(self, parameters: Parameters) -> [u8;3];
}
impl ToComplex for (u32, u32){
    fn normalise_in_x (self, x_res: u32, y_res: u32) -> Complex {
        let (x, y) = self;
        Complex{ 
            re: (2.0 * (x as f64) - (x_res as f64)) / (x_res as f64) , 
            im: ((y_res as f64) - 2.0 * (y as f64)) / (x_res as f64) 
        }
    }

    fn get_rgb_value(self, parameters: Parameters) -> [u8;3] {
        let mut z = self.normalise_in_x(parameters.x_res, parameters.y_res);
        z = z.affine_transform(parameters.scale_factor, parameters.translation);
        z.escape_time_mdlbrt(parameters.escape_limit).rgb(parameters.colour_mode)
    }
    
}
//#[time_function]
fn filename(extension: &str) -> String {
    let now: datetime::Instant = datetime::Instant::now();
    println!("{}_{}", now.seconds().to_string(), now.milliseconds().to_string());
    "images/mdlbrt_".to_string() + &now.seconds().to_string() + "_" + &now.milliseconds().to_string() + extension
}

fn get_data_channel(parameters: Parameters) -> Vec<u8> {
    let mut vector = Vec::new();
    for y in 0..parameters.y_res{
        for x in 0..parameters.x_res{
            let x: [u8;3] = (x,y).get_rgb_value(parameters);
            vector.push(x[0]);
            vector.push(x[1]);
            vector.push(x[2]);
        }
    }
    vector
}

fn get_image_buffer(parameters: Parameters) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let image_data: Vec<u8> = get_data_channel(parameters);

    let image_buffer: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_raw(parameters.x_res, parameters.y_res, image_data).expect("Resolution didn't match data stream");
    image_buffer
}

#[time_function]
fn main() {
    //let a: f64 = -0.03942862882707475;
    //let b: f64 = -0.9880027977017277;
    let mut parameters: Parameters = Parameters::default();
    let file_extension: &str = ".png";
    for _i in 0..1{ //340 before you get floating point weirdness
        let _ = get_image_buffer(parameters).save(filename(file_extension));
        parameters.scale_factor *= 0.9;
    }
}