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
mod worley;
pub use worley::WorleyGen;
mod mottler;
pub use mottler::Mottler;

#[allow(dead_code)]
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

/// Describes the type of generator the Gen implements.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Category {
  /// A generator just meant to be used to test other infrastructure
  Test,
  /// A single, simple generator of a standard type of noise
  Basic,
  /// An art piece, built out of other generators etc.
  Project,
}

impl Category {
  pub fn all() -> [Category; 3] {
    [ Category::Test, Category::Basic, Category::Project ]
  }

  pub fn name(&self) -> &'static str {
    match self {
      Category::Test => "test",
      Category::Basic => "basic",
      Category::Project => "project",
    }
  }

  pub fn description(&self) -> &'static str {
    match self {
      Category::Test => "A generator just meant to be used to test other infrastructure",
      Category::Basic => "A single, simple generator of noise, maybe with octaves or inversion applied",
      Category::Project => "An art piece, built out of other generators etc.",
    }
  }
}

// TODO: Add categories to generators, use them in main
/// A trait normalizing the interface across all generators
pub trait Gen: Sync {
  /// The category that the command is in, e.g. basic noise generation or tests
  fn category(&self) -> Category;
  /// The name of the subcommand to invoke to run this generator
  fn command(&self) -> &'static str;
  /// The human-friendly name of this subcommand
  fn about(&self) -> &'static str;
  /// Set up the subcommand for this generator, to fill out any needed extra command line options.
  /// Note you _should not_ add a subcommand for your gen: the parameter is the `SubCommand` which will be added for you.
  fn setup_cmd<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b>;
  /// Actually run the generator. Will be passed the subcommand's arguments only.
  fn run(&self, opts: &ArgMatches, seed: &[u8], output: &mut dyn Write) -> Result<()>;
}

impl dyn Gen {
  pub fn all() -> [&'static dyn Gen; 4] {
    [
      &TestAscii,
      &Test2D,
      &WorleyGen,
      &Mottler,
    ]
  }

  pub fn by_command(name: &str) -> Option<&'static dyn Gen> {
    let all = Self::all();
    for i in 0..all.len() {
      if all[i].command() == name {
        return Some(all[i]);
      }
    }
    None
  }
}