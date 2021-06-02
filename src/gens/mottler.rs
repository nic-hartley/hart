
use {
  clap::{App, Arg, ArgMatches},
  std::{
    io,
    time::Instant,
  },
  crate::utils::noise::{Worley, Noise2D, Pos},
  image::{
    codecs::png::{PngEncoder, CompressionType, FilterType},
    ColorType,
    GenericImageView,
    Pixel,
  },
  rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator},
};

fn validate_pct(s: String) -> Result<(), String> {
  decode_pct(&s).map(|_| ())
}

fn decode_pct(mut s: &str) -> Result<f32, String> {
  if s.ends_with("%") {
    s = &s[..s.len()-1];
  }
  match s.parse::<f32>() {
    Ok(i) => Ok(i / 100.0),
    _ => Err(format!("{} is not a percent from 0 to 100", s))
  }
}

pub struct Mottler;

impl super::Gen for Mottler {
    fn command(&self) -> &'static str { "mottle" }
    fn category(&self) -> super::Category { super::Category::Project }
    fn about(&self) -> &'static str { "Blend two images together by picking pixels based on 2D noise" }
    fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
        app
          .arg(Arg::with_name("from")
            .short("f")
            .long("from")
            .takes_value(true)
            .help("The image being blended 'from' (visible on the left or top)")
            .required(true))
          .arg(Arg::with_name("to")
            .short("t")
            .long("to")
            .takes_value(true)
            .help("The image being blended 'to' (visible on the right or bottom)")
            .required(true))
          .arg(Arg::with_name("vertical")
            .short("U")
            .long("vertical")
            .help("Render the 'gradient' from top to bottom, instead of left to right."))
          .arg(Arg::with_name("start")
            .short("s")
            .long("start")
            .validator(validate_pct)
            .default_value("25%")
            .help("The starting point for the gradient, as a percentage of image size"))
          .arg(Arg::with_name("end")
            .short("e")
            .long("end")
            .validator(validate_pct)
            .default_value("75%")
            .help("The ending point for the gradient, as a percentage of image size"))
          .arg(Arg::with_name("algorithm")
            .short("a")
            .long("algorithm")
            .possible_values(&["perlin", "worley"])
            .default_value("worley")
            .help("The algorithm to generate noise with"))
          .arg(Arg::with_name("algo-scale")
            .long("algorithm-scale")
            .default_value("5%")
            .help("How much of the image's width should be covered by one unit in the noise sampling space"))
    }
    fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn io::Write) -> super::Result<()> {
      let img_from_path = opts.value_of("from").unwrap();
      let img_from = image::io::Reader::open(img_from_path)?.decode()?;
      println!("Opened {}", img_from_path);
      let img_to_path = opts.value_of("to").unwrap();
      let img_to = image::io::Reader::open(img_to_path)?.decode()?;
      println!("Opened {}", img_to_path);

      if img_from.dimensions() != img_to.dimensions() {
        return Err(super::GenFail::BadArg(format!(
          "{} {:?} and {} {:?} are not the same size",
          img_from_path, img_from.dimensions(),
          img_to_path, img_to.dimensions()
        )))
      }

      if opts.is_present("vertical") {
        return Err(super::GenFail::BadArg(format!(
          "--vertical not supported yet, rotate the images yourself"
        )));
      }

      let (width, height) = img_from.dimensions();
      let width = width as usize;
      let height = height as usize;

      let pix_sz = decode_pct(opts.value_of("algo-scale").unwrap()).unwrap();
      let start_pct = decode_pct(opts.value_of("start").unwrap()).unwrap();
      let start = width as f32 * start_pct;
      let end_pct = decode_pct(opts.value_of("end").unwrap()).unwrap();
      let end = width as f32 * end_pct;

      let noise = match opts.value_of("algorithm").unwrap() {
        "perlin" => unimplemented!("perlin noise"),
        "worley" => Worley::new(seed),
        _ => unreachable!("Option values set with clap"),
      };

      let mut rows = Vec::with_capacity(height);
      let start_time = Instant::now();
      rows.par_extend((0..rows.capacity()).into_par_iter().map(|row| {
        let mut data_out = vec![0; width * 3];
        for x in 0..width {
          let threshold = if (x as f32) < start {
            0.0
          } else if (x as f32) > end {
            1.0
          } else {
            (x as f32 - start) / (end - start)
          };
          let pos = Pos::of(x as f32 * pix_sz, row as f32 * pix_sz);
          let color = if noise.get(pos) > threshold {
            &img_from
          } else {
            &img_to
          }.get_pixel(x as u32, row as u32);
          let (r, g, b, _) = color.channels4();
          data_out[x*3 + 0] = r;
          data_out[x*3 + 1] = g;
          data_out[x*3 + 2] = b;
        }
        data_out
      }));
      let gen_time = Instant::now() - start_time;
      println!("Took {}ms to generate", gen_time.as_millis());

      let mut pixels = Vec::with_capacity(width * height * 3);
      for row in rows {
        pixels.extend(row);
      }

      let encoder = PngEncoder::new_with_quality(output, CompressionType::Fast, FilterType::Sub);
      encoder.encode(&pixels, width as u32, height as u32, ColorType::Rgb8)?;

      println!("Finished");

      Ok(())
    }
}