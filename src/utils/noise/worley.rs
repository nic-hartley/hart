use {
  crate::utils::ForeveRNG,
  rand::Rng as _,
};

const OFFSETS: [(isize, isize); 9] = [
  (-1,  1), (0,  1), (1,  1),
  (-1,  0), (0,  0), (1,  0),
  (-1, -1), (0, -1), (1, -1),
];
const ISIZE_SZ: usize = std::mem::size_of::<isize>();

pub struct Worley {
  rng: ForeveRNG,
}

impl Worley {
  pub fn new(seed: &[u8]) -> Worley {
    Worley {
      rng: ForeveRNG::with_seed(seed),
    }
  }
}

impl super::Noise2D for Worley {
  fn get(&self, p: super::Pos) -> f32 {
    let frac_x = p.x.rem_euclid(1.0);
    let int_x = p.x.floor() as isize;
    let frac_y = p.y.rem_euclid(1.0);
    let int_y = p.y.floor() as isize;

    let middle_pt = super::Pos::of(frac_x, frac_y);
    let mut min_dist_sq = 1.0;
    for (off_x, off_y) in OFFSETS.iter() {
      let mut subseed = [0; ISIZE_SZ * 2];
      subseed[..ISIZE_SZ].copy_from_slice(&(int_x + off_x).to_be_bytes());
      subseed[ISIZE_SZ..].copy_from_slice(&(int_y + off_y).to_be_bytes());
      let mut subrng = self.rng.reseed(&subseed);
      let pt_x = subrng.gen::<f32>() + (*off_x as f32);
      let pt_y = subrng.gen::<f32>() + (*off_y as f32);
      let other_pt = super::Pos::of(pt_x, pt_y);
      let dist_sq = (other_pt - middle_pt).len_sq();
      if dist_sq < min_dist_sq {
        min_dist_sq = dist_sq;
      }
    }
    min_dist_sq.sqrt()
  }
}
