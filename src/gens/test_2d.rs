//! A test/template generator for ensuring the image-related chrome works

use {
  clap::{App, Arg, ArgMatches},
  std::{
    io,
    time::Instant,
  },
  crate::utils::noise::{Checkerboard, Noise2D, Pos},
  image::{
    ColorType,
    codecs::png::{PngEncoder, CompressionType, FilterType},
  },
  rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator},
};

const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;
const HEIGHT_PER_WORKER: usize = 8;
const PIX_SZ: f32 = 32.0;

const SATURATION: f32 = 0.75;

// the number of octaves, if we're using them
const OCTAVES: usize = 3;
const ZOOM: f32 = 3.0;
const SCALE: f32 = 0.8;

pub struct Test2D;

impl super::Gen for Test2D {
  fn category(&self) -> super::Category { super::Category::Test }
  fn command(&self) -> &'static str { "2d" }
  fn about(&self) -> &'static str { "A test generator which outputs a PNG" }
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(Arg::with_name("octave")
      .short("8")
      .long("octave")
      .takes_value(false)
      .help("Demonstrate octaves"))
  }
  fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn io::Write) -> super::Result<()> {
    let octaves = if opts.is_present("octave") { OCTAVES } else { 1 };
    let mut subseed = Vec::with_capacity(seed.len() + 1);
    subseed.push(0);
    subseed.extend(seed);
    let center = Pos::of(WIDTH as f32 / PIX_SZ / 2.0, HEIGHT as f32 / PIX_SZ / 2.0);
    let red = Checkerboard::new(&subseed).octaves().count(octaves).zoom(ZOOM).scale(SCALE).offset(Pos::zero());
    subseed[0] += 1;
    let green = Checkerboard::new(&subseed).octaves().count(octaves).zoom(ZOOM).scale(SCALE).offset(Pos::zero());
    subseed[0] += 1;
    let blue = Checkerboard::new(&subseed).octaves().count(octaves).zoom(ZOOM).scale(SCALE).offset(Pos::zero());

    let num_workers = if HEIGHT % HEIGHT_PER_WORKER == 0 {
      HEIGHT / HEIGHT_PER_WORKER
    } else {
      HEIGHT / HEIGHT_PER_WORKER + 1
    };
    let mut rows = Vec::with_capacity(num_workers);

    let sat_mul = SATURATION * 255.0;
    // bias dark:
    // let sat_add = 0;
    // bias grey:
    let sat_add = (255 - sat_mul as u8) / 2;
    // bias light:
    // let sat_add = (255 - sat_mul as u8);

    let start = Instant::now();
    rows.par_extend((0..num_workers).into_par_iter().map(|row| {
      let start_y = row * HEIGHT_PER_WORKER;
      let height = if HEIGHT % HEIGHT_PER_WORKER != 0 {
        std::cmp::min(HEIGHT_PER_WORKER, HEIGHT - start_y)
      } else {
        HEIGHT_PER_WORKER
      };
      let mut data_out = vec![0; WIDTH * 3 * height];
      for idx_y in 0..height {
        let y = start_y + idx_y;
        for x in 0..WIDTH {
          let pos = Pos::of(x as f32 / PIX_SZ, y as f32 / PIX_SZ) - center;
          let r = (red.get(pos) * sat_mul) as u8 + sat_add;
          let g = (green.get(pos) * sat_mul) as u8 + sat_add;
          let b = (blue.get(pos) * sat_mul) as u8 + sat_add;
          let idx = idx_y * WIDTH * 3 + x * 3;
          data_out[idx+0] = r;
          data_out[idx+1] = g;
          data_out[idx+2] = b;
        }
      }
      data_out
    }));
    let gen_time = Instant::now() - start;

    println!("Took {}ms to generate", gen_time.as_millis());

    let mut pixels = vec![0; WIDTH * HEIGHT * 3];
    for (i, row) in rows.into_iter().enumerate() {
      let start = i * WIDTH * 3 * HEIGHT_PER_WORKER;
      let end = start + row.len();
      pixels[start..end].copy_from_slice(&row);
    }
    let encoder = PngEncoder::new_with_quality(output, CompressionType::Fast, FilterType::Sub);
    encoder.encode(&pixels, WIDTH as u32, HEIGHT as u32, ColorType::Rgb8)?;
    Ok(())
  }
}
