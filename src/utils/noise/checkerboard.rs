use {
  crate::utils::ForeveRNG,
  rand::RngCore as _,
};

// Cache is mostly so that as we query the _same_ square repeatedly, we don't have to recalculate it.
// It doesn't need to be huge, just big enough to reduce that weight.
// Ideally as they scan down multiple rows of identical squares that'll be sped up too, but we'll see.
pub struct Checkerboard {
  rng: ForeveRNG,
}

const ISIZE_SZ: usize = std::mem::size_of::<isize>();

impl Checkerboard {
  pub fn new(seed: &[u8]) -> Checkerboard {
    Checkerboard {
      rng: ForeveRNG::with_seed(seed),
    }
  }
}

impl super::Noise2D for Checkerboard {
  fn get(&self, x: f32, y: f32) -> f32 {
    let x = x as isize;
    let y = y as isize;
    let mut subseed = [0; ISIZE_SZ * 2];
    subseed[..ISIZE_SZ].copy_from_slice(&x.to_be_bytes());
    subseed[ISIZE_SZ..].copy_from_slice(&y.to_be_bytes());
    let mut subrng = self.rng.reseed(&subseed);
    if subrng.next_u32() % 2 == 1 {
      1.0
    } else {
      0.0
    }
  }
}
