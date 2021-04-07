//! The module where all of the actual art generators live.
//! The main module is mostly providing a clean UI.
//! The generators themselves are fairly simple; the (gross) workhorse code will likely live in crate::utils

use {
  clap::{App, ArgMatches},
  image::ImageError,
  std::io::{Error, Write},
};

mod test_ascii;
pub use test_ascii::TestAscii;
mod test_2d;
pub use test_2d::Test2D;

#[derive(Debug)]
pub enum GenFail {
  Io(Error),
  Image(ImageError),
  BadArg(String),
}

impl From<Error> for GenFail {
  fn from(e: Error) -> GenFail {
    GenFail::Io(e)
  }
}

impl From<ImageError> for GenFail {
  fn from(e: ImageError) -> GenFail {
    if let ImageError::IoError(inner) = e {
      GenFail::Io(inner)
    } else {
      GenFail::Image(e)
    }
  }
}

pub type Result<T> = std::result::Result<T, GenFail>;

/// A trait normalizing the interface across all generators
pub trait Gen: Sync {
  /// The name of the subcommand to invoke to run this generator
  fn command(&self, ) -> &'static str;
  /// The human-friendly name of this subcommand
  fn about(&self, ) -> &'static str;
  /// Set up the subcommand for this generator, to fill out any needed extra command line options.
  /// Note you _should not_ add a subcommand for your gen: the parameter is the `SubCommand` which will be added for you.
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b>;
  /// Actually run the generator. Will be passed the subcommand's arguments only.
  fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn Write) -> Result<()>;
}

impl dyn Gen {
  pub fn all() -> &'static [&'static dyn Gen] {
    &[
      &TestAscii,
      &Test2D,
    ]
  }

  pub fn by_command(name: &str) -> Option<&'static dyn Gen> {
    match name {
      "test-ascii" => Some(&TestAscii),
      "test-2d" => Some(&Test2D),
      _ => None
    }
  }
}