//! A test/template generator for ensuring the image-related chrome works

use {
  clap::{App, Arg, ArgMatches},
  std::io,
  crate::utils::noise::{Checkerboard, Noise2D},
  image::{
    ColorType,
    codecs::png::{PngEncoder, CompressionType, FilterType},
  },
  rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator},
};

const WIDTH: usize = 2048;
const HEIGHT: usize = 2048;
const PIX_WIDTH: f32 = 32.0;
const PIX_HEIGHT: f32 = 32.0;

// the number of octaves, if we're using them
const OCTAVES: usize = 4;
const ZOOM: f32 = 2.0;
const SCALE: f32 = 0.75;

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
    let octaves = if opts.is_present("octave") { OCTAVES } else { 1 };
    let mut subseed = Vec::with_capacity(seed.len() + 1);
    subseed.push(0);
    subseed.extend(seed);
    let red = Checkerboard::new(&subseed).octaves(octaves, ZOOM, SCALE);
    subseed[0] += 1;
    let green = Checkerboard::new(&subseed).octaves(octaves, ZOOM, SCALE);
    subseed[0] += 1;
    let blue = Checkerboard::new(&subseed).octaves(octaves, ZOOM, SCALE);

    let mut rows = Vec::with_capacity(HEIGHT);
    rows.par_extend((0..HEIGHT).into_par_iter().map(|y| {
      let mut data_out = [0; WIDTH * 3];
      for x in 0..WIDTH {
        let r = (red.get(x as f32 / PIX_WIDTH, y as f32 / PIX_HEIGHT) * 255.0) as u8;
        let g = (green.get(x as f32 / PIX_WIDTH, y as f32 / PIX_HEIGHT) * 255.0) as u8;
        let b = (blue.get(x as f32 / PIX_WIDTH, y as f32 / PIX_HEIGHT) * 255.0) as u8;
        data_out[x*3+0] = r;
        data_out[x*3+1] = g;
        data_out[x*3+2] = b;
      }
      data_out
    }));

    let mut pixels = vec![0; WIDTH * HEIGHT * 3];
    for i in 0..HEIGHT {
      let start = i * WIDTH * 3;
      let end = start + WIDTH * 3;
      pixels[start..end].copy_from_slice(&rows[i]);
    }

    println!("Writing");
    let encoder = PngEncoder::new_with_quality(output, CompressionType::Fast, FilterType::Sub);
    encoder.encode(&pixels, WIDTH as u32, HEIGHT as u32, ColorType::Rgb8)?;
    Ok(())
  }
}
