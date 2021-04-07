//! A test/template generator for ensuring the chrome works

use {
  clap::{App, ArgMatches},
  std::io,
};

pub struct TestAscii;

impl super::Gen for TestAscii {
  fn command(&self) -> &'static str { "test-ascii" }
  fn about(&self) -> &'static str { "A test generator which outputs some ASCII" }
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> { app }
  fn run(&self, _: &ArgMatches, seed: &[u8], out: &mut dyn io::Write) -> super::Result<()> {
    out.write(format!("Seeded with {:?}\n", seed).as_bytes())?;
    Ok(())
  }
}
