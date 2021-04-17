//! Generate Worley noise.

use {
  clap::{App, Arg, ArgMatches},
  std::{
    io,
    time::Instant,
  },
  crate::utils::noise::{Worley, Noise2D, Pos},
  image::{
    ColorType,
    codecs::png::{PngEncoder, CompressionType, FilterType},
  },
  rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator},
};

const HEIGHT_PER_WORKER: usize = 8;
fn validate_usize(s: String) -> Result<(), String> {
  match s.parse::<usize>() {
    Ok(_) => Ok(()),
    _ => Err(format!("{} is not a nonnegative integer in range", s))
  }
}
fn validate_pos_usize(s: String) -> Result<(), String> {
  match s.parse::<usize>() {
    Ok(i) if i > 0 => Ok(()),
    _ => Err(format!("{} is not a nonnegative integer in range", s))
  }
}

pub struct WorleyGen;

impl super::Gen for WorleyGen {
  fn command(&self) -> &'static str { "basic:worley" }
  fn about(&self) -> &'static str { "Generate Worley noise" }
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
    app
      .arg(Arg::with_name("size")
        .help("Output size in pixels")
        .short("D")
        .long("size")
        .value_names(&["width", "height"])
        .validator(validate_pos_usize)
        .required(true))
      .arg(Arg::with_name("pixels")
        .help("Number of pixels per in-noise unit")
        .short("p")
        .long("pixels")
        .validator(validate_pos_usize)
        .default_value("25"))
      .arg(Arg::with_name("octaves")
        .help("Number of layers of noise to add")
        .short("O")
        .long("octaves")
        .validator(validate_pos_usize)
        .default_value("1"))
  }
  fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn io::Write) -> super::Result<()> {
    let size = opts.values_of("size").unwrap().collect::<Vec<_>>();
    let img_width: usize = size[0].parse().unwrap();
    let img_height: usize = size[1].parse().unwrap();
    let pix_sz: f32 = opts.value_of("pixels").unwrap().parse().unwrap();
    let octaves: usize = opts.value_of("octaves").unwrap().parse().unwrap();

    let gen = Worley::new(seed).octaves().count(octaves).zoom(2.0).scale(0.5).offset(Pos::of(10.0, -4.83)).invert();

    let num_workers = if img_height % HEIGHT_PER_WORKER == 0 {
      img_height / HEIGHT_PER_WORKER
    } else {
      img_height / HEIGHT_PER_WORKER + 1
    };

    let mut rows = Vec::with_capacity(num_workers);
    let start = Instant::now();
    rows.par_extend((0..num_workers).into_par_iter().map(|row| {
      let start_y = row * HEIGHT_PER_WORKER;
      let block_height = if img_height % HEIGHT_PER_WORKER != 0 {
        std::cmp::min(HEIGHT_PER_WORKER, img_height - start_y)
      } else {
        HEIGHT_PER_WORKER
      };
      let mut data_out = vec![0; img_width * block_height];
      for idx_y in 0..block_height {
        let y = start_y + idx_y;
        for x in 0..img_width {
          let pos = Pos::of(x as f32 / pix_sz, y as f32 / pix_sz);
          let idx = idx_y * img_width + x;
          data_out[idx] = (gen.get(pos) * 255.0) as u8;
        }
      }
      data_out
    }));
    let gen_time = Instant::now() - start;

    println!("Took {}ms to generate", gen_time.as_millis());

    let mut pixels = vec![0; img_width * img_height];
    for (i, row) in rows.into_iter().enumerate() {
      let start = i * img_width * HEIGHT_PER_WORKER;
      let end = start + row.len();
      pixels[start..end].copy_from_slice(&row);
    }
    let encoder = PngEncoder::new_with_quality(output, CompressionType::Fast, FilterType::Sub);
    encoder.encode(&pixels, img_width as u32, img_height as u32, ColorType::L8)?;
    Ok(())
  }
}
