//! A test/template generator for ensuring the image-related chrome works

use {
  clap::{App, Arg, ArgMatches},
  std::{
    f32::consts,
    io,
  },
  crate::utils::noise::{Checkerboard, Noise2D},
  image::{
    ColorType,
    codecs::png::{PngEncoder, CompressionType, FilterType},
  },
};

const WIDTH: usize = 2000;
const HEIGHT: usize = 2000;
const PIX_WIDTH: f32 = 50.0;
const PIX_HEIGHT: f32 = 50.0;

pub struct Test2D;

impl super::Gen for Test2D {
  fn command(&self) -> &'static str { "test-2d" }
  fn about(&self) -> &'static str { "A test generator which outputs some ASCII" }
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(Arg::with_name("octave")
      .short("8")
      .long("octave")
      .takes_value(false)
      .help("Demonstrate octave functionality (with 3 layers, scale=1/sqrt(2), zoom=sqrt(2))"))
  }
  fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn io::Write) -> super::Result<()> {
    let checker = Checkerboard::new(seed);
    let octaves = if opts.is_present("octave") { 7 } else { 1 };
    let noise = checker.octaves(octaves, consts::SQRT_2, consts::FRAC_1_SQRT_2);
    let mut data_out = [0; WIDTH * HEIGHT];
    println!("Generating...");
    for x in 0..WIDTH {
      for y in 0..HEIGHT {
        let val = noise.get(x as f32 / PIX_WIDTH, y as f32 / PIX_HEIGHT);
        let g = (val * 255.0) as u8;
        let i = y * WIDTH + x;
        data_out[i+0] = g;
      }
    }
    println!("Writing...");
    let encoder = PngEncoder::new_with_quality(output, CompressionType::Fast, FilterType::Sub);
    encoder.encode(&data_out, WIDTH as u32, HEIGHT as u32, ColorType::L8)?;
    Ok(())
  }
}
