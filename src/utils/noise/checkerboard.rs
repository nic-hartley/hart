pub struct Checkerboard {
  bias: isize,
}

impl Checkerboard {
  pub fn new(seed: &[u8]) -> Checkerboard {
    Checkerboard {
      bias: seed.iter().fold(0, |a, i| *i as isize + a)
    }
  }
}

impl super::Noise2D for Checkerboard {
  fn get(&self, x: f32, y: f32) -> f32 {
    let x = x as isize;
    let y = y as isize;
    if (x + y + self.bias) % 2 == 0 {
      // println!("x={}, y={}, r={}", x, y, 1.0);
      1.0
    } else {
      // println!("x={}, y={}, r={}", x, y, 0.0);
      0.0
    }
  }
}
