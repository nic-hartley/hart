//! A test/template generator for ensuring the chrome works

use {
  clap::{App, ArgMatches},
  std::io,
  crate::utils::ForeveRNG,
  rand::RngCore as _,
};

fn write_from(mut rng: ForeveRNG, name: &str, out: &mut dyn io::Write) -> super::Result<()> {
  out.write(b"Some random data from ")?;
  out.write(name.as_bytes())?;
  out.write(b"\n")?;
  let mut random_data = [0; 32];
  rng.fill_bytes(&mut random_data);
  for byte in random_data.iter() {
    out.write(format!(" {:02x}", byte).as_bytes())?;
  }
  out.write(b"\n")?;
  Ok(())
}

pub struct TestAscii;

impl super::Gen for TestAscii {
  fn category(&self) -> super::Category { super::Category::Test }
  fn command(&self) -> &'static str { "ascii" }
  fn about(&self) -> &'static str { "A test generator which outputs some ASCII" }
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> { app }
  fn run(&self, _: &ArgMatches, seed: &[u8], out: &mut dyn io::Write) -> super::Result<()> {
    out.write(format!("Seeded with {:?}\n", seed).as_bytes())?;
    let rng = ForeveRNG::with_seed(seed);
    let rng_c1 = rng.reseed(b"Hello");
    let rng_c2 = rng.reseed(b"World");
    let rng_c3 = rng.reseed(b"Hello");
    write_from(rng, "Parent", out)?;
    write_from(rng_c1, "Child 1-1", out)?;
    write_from(rng_c2, "Child 2", out)?;
    write_from(rng_c3, "Child 1-2", out)?;
    Ok(())
  }
}
